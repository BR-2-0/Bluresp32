/// Bluresp32 — ESP32-S3 BLE Receipt Dongle Firmware
///
/// Architecture:
///   POS System ──USB──▶ [ESP32-S3] ──BLE──▶ Android/iOS
///
/// The dongle plugs into a POS via USB and enumerates as a USB Printer
/// Class device. When the POS prints a receipt, the ESC/POS data arrives
/// over USB, gets framed into BLE notification chunks (matching the
/// blure-core protocol), and is sent to the connected mobile app.
///
/// This replaces the Windows BLE dependency entirely — the ESP32 has its
/// own dedicated BLE radio, so no adapter selection issues.

mod ble;
mod config;
mod protocol;
mod receipt;
mod usb;

use std::sync::Arc;

use esp_idf_svc::nvs::EspDefaultNvsPartition;
use log::info;

fn main() {
    // 1. Initialize ESP-IDF runtime (logging, event loop, etc.)
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("=== Bluresp32 starting ===");

    // 2. Load config from NVS (store name, register number, device name)
    let nvs_partition = EspDefaultNvsPartition::take().expect("Failed to take NVS partition");
    let cfg = config::BlureConfig::load(nvs_partition);
    let ble_name = cfg.ble_name();
    info!("BLE name: {:?}", ble_name);

    // 3. Initialize USB Printer Class — returns receipt buffer
    let receipt_buffer = usb::printer_class::init();

    // 4. Initialize BLE GATT server + start advertising
    let gatt_state = ble::gatt_server::init(&ble_name);

    // 5. Spawn heartbeat task
    ble::heartbeat::start(Arc::clone(&gatt_state));

    // 6. Main loop: USB receipts → frame → BLE notifications
    info!("Entering main receipt loop");
    loop {
        let receipt_data = receipt_buffer.recv(); // blocks until a receipt arrives
        info!("Receipt received from USB: {} bytes", receipt_data.len());

        match receipt::framer::frame_receipt(&receipt_data) {
            Ok(frames) => {
                ble::gatt_server::send_receipt(&gatt_state, &frames);
            }
            Err(e) => {
                log::warn!("Failed to frame receipt: {:?}", e);
            }
        }
    }
}
