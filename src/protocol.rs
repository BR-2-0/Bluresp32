/// BLE protocol constants — mirrors blure-core/src/protocol.rs exactly.
/// Any change here MUST be synced with the canonical source in blure-core.

// ── UUIDs ───────────────────────────────────────────────────────────────

/// Primary BLE service UUID — Android and iOS scan exclusively for this.
pub const SERVICE_UUID_STR: &str = "0016FF85-E965-4B40-8412-5E6215C87D29";

/// FILE_REQ characteristic — carries receipt data via notifications.
/// Properties: Read, Notify.
pub const FILE_REQ_UUID_STR: &str = "012EA2BD-3B37-4BE3-AA32-9334D7C6CFFA";

/// CLIENT_ID characteristic — mobile writes its device identifier here.
/// Properties: Write.
pub const CLIENT_ID_UUID_STR: &str = "011EA2BD-3B37-4BE3-AA32-9334D7C6CFFA";

/// Standard BLE CCCD UUID.
pub const CCC_UUID_STR: &str = "00002902-0000-1000-8000-00805F9B34FB";

// ── Framing ─────────────────────────────────────────────────────────────

/// Chunk size in bytes — conservative for BLE 4.0 (ATT_MTU 23).
pub const CHUNK_SIZE: usize = 182;

/// Interval between chunk notifications in milliseconds.
pub const CHUNK_INTERVAL_MS: u32 = 100;

/// ASCII End-Of-Transmission — signals receipt transfer complete.
pub const EOF_MARKER: u8 = 0x04;

/// Maximum receipt size (1 MB).
pub const MAX_RECEIPT_BYTES: usize = 1_048_576;

/// Maximum chunks per receipt.
pub const MAX_CHUNKS: usize = 10_000;

// ── Timing ──────────────────────────────────────────────────────────────

/// Timeout waiting for mobile to subscribe after connect.
pub const SUBSCRIBE_TIMEOUT_SECS: u64 = 15;

/// Heartbeat byte — sent when idle to keep connection alive.
pub const HEARTBEAT_BYTE: u8 = 0x00;

/// Interval between heartbeat pings.
pub const HEARTBEAT_INTERVAL_SECS: u64 = 4;

/// Mobile-side idle timeout — disconnect if nothing arrives in this window.
pub const CONNECTION_IDLE_TIMEOUT_SECS: u64 = 300;

// ── USB ─────────────────────────────────────────────────────────────────

/// Idle time (ms) after last USB byte before treating the buffer as a complete receipt.
/// Matches the Rust printer's TCP behavior.
pub const USB_IDLE_TIMEOUT_MS: u32 = 250;

/// Maximum CLIENT_ID write length.
pub const MAX_CLIENT_ID_LEN: usize = 256;
