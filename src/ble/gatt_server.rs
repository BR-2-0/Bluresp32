/// NimBLE GATT server — exposes the Blure receipt service.
///
/// Replicates the exact GATT layout from blure-core so that the Android/iOS
/// apps connect without any changes.
///
/// Service: SERVICE_UUID
///   ├─ FILE_REQ  (Read | Notify)  — receipt chunks + heartbeat
///   └─ CLIENT_ID (Write)          — mobile writes its identifier

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use esp_idf_hal::delay::FreeRtos;
use esp_idf_svc::bt::ble::gap::{BleGapEvent, EspBleGap};
use esp_idf_svc::bt::ble::gatt::server::{EspGatts, GattsEvent};
use esp_idf_svc::bt::{BdAddr, BtDriver, BtStatus};
use log::{info, warn};

use crate::protocol::*;

/// Shared state between GATT callbacks and the main loop.
pub struct GattState {
    /// True when a mobile client has subscribed to FILE_REQ notifications.
    pub has_subscriber: AtomicBool,
    /// True when a receipt transfer is in progress (suppresses heartbeat).
    pub is_transferring: AtomicBool,
    /// FILE_REQ attribute handle — set during service registration.
    pub file_req_handle: std::sync::Mutex<Option<u16>>,
    /// Connection ID of the current subscriber.
    pub conn_id: std::sync::Mutex<Option<u16>>,
}

impl GattState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            has_subscriber: AtomicBool::new(false),
            is_transferring: AtomicBool::new(false),
            file_req_handle: std::sync::Mutex::new(None),
            conn_id: std::sync::Mutex::new(None),
        })
    }
}

/// Initialize the BLE GATT server and start advertising.
///
/// Returns the shared GattState so the main loop and heartbeat task can
/// check subscriber status and send notifications.
///
/// # Implementation Notes
///
/// The actual NimBLE initialization requires unsafe ESP-IDF calls via
/// `esp_idf_svc::bt`. The high-level flow is:
///
/// 1. Create BtDriver (enables BT controller + NimBLE host)
/// 2. Register GATT service with FILE_REQ + CLIENT_ID characteristics
/// 3. Set advertising data (flags + service UUID + local name)
/// 4. Start advertising (discoverable + connectable)
/// 5. Handle GATT events:
///    - Connect    → store conn_id
///    - Disconnect → clear subscriber, restart advertising
///    - Write      → CLIENT_ID: log identifier, reject if > MAX_CLIENT_ID_LEN
///    - CCCD Write → FILE_REQ subscription: set/clear has_subscriber
///
/// This is the scaffold — full NimBLE wiring depends on the esp-idf-svc
/// version's BLE API surface which is still evolving.
pub fn init(device_name: &str) -> Arc<GattState> {
    let state = GattState::new();

    info!("BLE GATT server initializing as {:?}", device_name);
    info!("Service UUID: {}", SERVICE_UUID_STR);
    info!("FILE_REQ UUID: {} (Read|Notify)", FILE_REQ_UUID_STR);
    info!("CLIENT_ID UUID: {} (Write)", CLIENT_ID_UUID_STR);

    // TODO: Full NimBLE initialization sequence:
    //
    // let bt = BtDriver::new(...)?;
    // let gap = EspBleGap::new(&bt)?;
    // let gatts = EspGatts::new(&bt)?;
    //
    // Register service:
    //   gatts.register_service(SERVICE_UUID, [
    //       Characteristic::new(FILE_REQ_UUID, Read | Notify),
    //       Characteristic::new(CLIENT_ID_UUID, Write, max_len=256),
    //   ]);
    //
    // Set advertising:
    //   gap.set_device_name(device_name);
    //   gap.set_adv_data(flags=GENERAL_DISCOVERABLE, service_uuids=[SERVICE_UUID]);
    //   gap.start_advertising(connectable=true);
    //
    // Event callbacks update GattState atomics.

    state
}

/// Send a single notification on FILE_REQ.
///
/// Used by both the receipt transfer loop and the heartbeat task.
pub fn notify(state: &GattState, data: &[u8]) -> Result<(), ()> {
    let handle = state.file_req_handle.lock().unwrap();
    let conn = state.conn_id.lock().unwrap();

    match (*handle, *conn) {
        (Some(_attr_handle), Some(_conn_id)) => {
            // TODO: esp_ble_gatts_send_indicate(conn_id, attr_handle, data.len(), data, false)
            Ok(())
        }
        _ => {
            warn!("notify() called with no subscriber");
            Err(())
        }
    }
}

/// Send a framed receipt as a sequence of BLE notifications.
///
/// Frames: [chunk_count_str, data_chunk_1, ..., data_chunk_N, EOF]
/// with CHUNK_INTERVAL_MS between each notification.
pub fn send_receipt(state: &Arc<GattState>, frames: &[Vec<u8>]) {
    if !state.has_subscriber.load(Ordering::Relaxed) {
        warn!("No subscriber — dropping receipt ({} frames)", frames.len());
        return;
    }

    state.is_transferring.store(true, Ordering::Relaxed);
    info!("Sending receipt: {} frames", frames.len());

    for (i, frame) in frames.iter().enumerate() {
        if notify(state, frame).is_err() {
            warn!("Notification failed at frame {}", i);
            break;
        }
        if i < frames.len() - 1 {
            FreeRtos::delay_ms(CHUNK_INTERVAL_MS);
        }
    }

    state.is_transferring.store(false, Ordering::Relaxed);
    info!("Receipt transfer complete");
}
