/// Heartbeat task — sends a single 0x00 byte on FILE_REQ every 4 seconds
/// to keep the BLE connection alive when idle.
///
/// Suppressed during active receipt transfers.

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use log::debug;

use crate::ble::gatt_server::{self, GattState};
use crate::protocol::*;

/// Spawn the heartbeat task on a FreeRTOS thread.
///
/// Runs forever. Sends HEARTBEAT_BYTE on FILE_REQ when:
///   - has_subscriber == true
///   - is_transferring == false
pub fn start(state: Arc<GattState>) {
    thread::Builder::new()
        .name("heartbeat".into())
        .stack_size(2048)
        .spawn(move || loop {
            thread::sleep(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));

            if !state.has_subscriber.load(Ordering::Relaxed) {
                continue;
            }
            if state.is_transferring.load(Ordering::Relaxed) {
                continue;
            }

            debug!("Heartbeat ping");
            let _ = gatt_server::notify(&state, &[HEARTBEAT_BYTE]);
        })
        .expect("Failed to spawn heartbeat task");
}
