/// USB Printer Class device — makes the ESP32-S3 enumerate as a USB printer
/// when plugged into a POS system.
///
/// Uses the ESP32-S3's native USB OTG peripheral via TinyUSB (bundled in ESP-IDF).
///
/// USB Class:    0x07 (Printer)
/// Subclass:     0x01 (Printers)
/// Protocol:     0x02 (Bidirectional)
///
/// Data flow:
///   POS sends ESC/POS print job → USB bulk OUT → accumulate in buffer
///   → 250ms idle timeout → push complete receipt to ReceiptBuffer

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use log::{info, warn};

use crate::protocol::USB_IDLE_TIMEOUT_MS;
use crate::receipt::buffer::ReceiptBuffer;

/// USB Printer Class descriptors.
///
/// These are the TinyUSB descriptor constants that will be passed to
/// `tud_descriptor_device_cb` and the configuration descriptor.
pub const USB_VID: u16 = 0x1209; // pid.codes open-source VID
pub const USB_PID: u16 = 0xB10E; // placeholder — register at pid.codes
pub const USB_CLASS_PRINTER: u8 = 0x07;
pub const USB_SUBCLASS_PRINTER: u8 = 0x01;
pub const USB_PROTOCOL_BIDIRECTIONAL: u8 = 0x02;

/// Device string descriptors.
pub const MANUFACTURER: &str = "Blure";
pub const PRODUCT: &str = "Blure Receipt Printer";
pub const SERIAL_NUMBER: &str = "BLURE-ESP32-001";

/// Initialize the USB Printer Class device and spawn the read task.
///
/// Returns an Arc<ReceiptBuffer> that the main loop reads from.
///
/// # Implementation Notes
///
/// TinyUSB on ESP-IDF requires:
/// 1. Configure USB descriptors (device, config, string, interface)
/// 2. Call `tud_init()` to start the USB device stack
/// 3. Poll `tud_task()` in a loop or dedicated FreeRTOS task
/// 4. Implement `tud_printer_rx_cb()` to receive bulk OUT data
///
/// The idle-timeout accumulation pattern:
/// - Each USB bulk OUT transfer appends bytes to a staging buffer
/// - A timer resets on every new data arrival
/// - When USB_IDLE_TIMEOUT_MS elapses with no new data, the staging
///   buffer is treated as a complete receipt and pushed to ReceiptBuffer
pub fn init() -> Arc<ReceiptBuffer> {
    let buffer = Arc::new(ReceiptBuffer::new(8));

    info!("USB Printer Class initializing");
    info!(
        "  VID={:#06x} PID={:#06x} Class={:#04x}/{:#04x}/{:#04x}",
        USB_VID, USB_PID, USB_CLASS_PRINTER, USB_SUBCLASS_PRINTER, USB_PROTOCOL_BIDIRECTIONAL
    );

    // TODO: Full TinyUSB initialization:
    //
    // unsafe {
    //     // Configure descriptors
    //     tinyusb_driver_install(&tusb_cfg);
    //
    //     // The TinyUSB task runs in the background, calling our callbacks:
    //     //   tud_printer_rx_cb(itf, data, len) → append to staging buffer
    //     //   tud_mount_cb()  → log "USB mounted"
    //     //   tud_umount_cb() → log "USB unmounted"
    // }

    let buf_clone = Arc::clone(&buffer);
    thread::Builder::new()
        .name("usb_reader".into())
        .stack_size(4096)
        .spawn(move || usb_read_loop(buf_clone))
        .expect("Failed to spawn USB reader task");

    buffer
}

/// USB read loop — accumulates bulk OUT data and pushes complete receipts.
///
/// This is a placeholder that will be wired to TinyUSB's rx callback.
fn usb_read_loop(buffer: Arc<ReceiptBuffer>) {
    let mut staging: Vec<u8> = Vec::with_capacity(4096);
    let mut last_data_time = Instant::now();
    let idle_timeout = Duration::from_millis(USB_IDLE_TIMEOUT_MS as u64);

    loop {
        // TODO: Replace with actual TinyUSB bulk OUT polling.
        // For now, this is the structural skeleton:
        //
        // if let Some(chunk) = tinyusb_read_bulk_out() {
        //     staging.extend_from_slice(&chunk);
        //     last_data_time = Instant::now();
        // }

        // Check idle timeout — if we have data and enough time has passed,
        // treat it as a complete receipt.
        if !staging.is_empty() && last_data_time.elapsed() >= idle_timeout {
            let receipt = std::mem::take(&mut staging);
            info!("USB receipt complete: {} bytes", receipt.len());

            if !buffer.push(receipt) {
                warn!("Receipt buffer full — dropping receipt");
            }
        }

        // Yield to other tasks
        thread::sleep(Duration::from_millis(10));
    }
}
