# Bluresp32

ESP32-S3 BLE receipt dongle firmware. Receives ESC/POS data from a POS system over USB and delivers it to Android and iOS BlueReceipt clients over Bluetooth Low Energy.

## Architecture

```
POS System ──USB Printer Class──► ┌──────────────────┐
                                  │     ESP32-S3      │
                                  │                    │──BLE Notify──► Android Central
                                  │  USB Bulk OUT →    │──BLE Notify──► iOS Central
                                  │  ReceiptBuffer →   │
                                  │  Framer → GATT     │
                                  └──────────────────┘
```

### Data Flow

1. **USB Printer Class** — POS system sends raw ESC/POS data to the dongle as a standard USB printer (Class 0x07). No drivers needed.
2. **Receipt Buffer** — Thread-safe ring buffer (capacity 8) decouples USB read task from BLE send loop.
3. **Framer** — Splits receipts into 182-byte BLE-safe chunks with count header and EOF marker. Same protocol as [Blure-Printer](https://github.com/BR-2-0/Blure-Printer).
4. **GATT Server** — NimBLE-based BLE peripheral advertising as "BRPrinter". Sends chunked notifications on FILE_REQ characteristic.
5. **Heartbeat** — 0x00 byte every 4 seconds keeps the BLE connection alive when idle.

### Modules

| Module | File | Purpose |
|--------|------|---------|
| **Entry** | `src/main.rs` | Init USB + BLE, main dispatch loop |
| **Config** | `src/config.rs` | NVS-backed persistent config (store name, register #) |
| **Protocol** | `src/protocol.rs` | BLE UUIDs, chunk size, timing constants |
| **GATT Server** | `src/ble/gatt_server.rs` | NimBLE GATT service, notifications, subscriber tracking |
| **Heartbeat** | `src/ble/heartbeat.rs` | FreeRTOS thread sending keepalive bytes |
| **Receipt Buffer** | `src/receipt/buffer.rs` | Thread-safe Mutex+Condvar ring buffer |
| **Framer** | `src/receipt/framer.rs` | Receipt → chunked BLE frames (tested) |
| **USB Printer** | `src/usb/printer_class.rs` | TinyUSB Printer Class device, idle detection |

## BLE Protocol

Same protocol as [Blure-Printer](https://github.com/BR-2-0/Blure-Printer) — Android and iOS clients connect identically to both.

| Entity | UUID | Properties |
|--------|------|------------|
| Service | `0016FF85-E965-4B40-8412-5E6215C87D29` | Primary |
| FILE_REQ | `012EA2BD-3B37-4BE3-AA32-9334D7C6CFFA` | Read, Notify |
| CLIENT_ID | `011EA2BD-3B37-4BE3-AA32-9334D7C6CFFA` | Write |

**Transfer sequence:**

```
Notification 1:   UTF-8 chunk count (e.g. "3")
Notification 2:   Raw bytes (≤182 bytes)
Notification 3:   Raw bytes (≤182 bytes)
Notification 4:   Raw bytes (remaining)
Notification 5:   0x04 (EOF marker)
```

- Chunk size: **182 bytes**
- Pacing: **100ms** between notifications
- Heartbeat: **0x00** every 4 seconds (suppressed during transfer)
- Idle timeout: **5 minutes**
- Max receipt: **1 MB** / **10,000 chunks**

## USB Printer Class

The dongle enumerates as a standard USB printer — no custom drivers required.

| Field | Value |
|-------|-------|
| VID | `0x1209` (pid.codes open-source) |
| PID | `0xB10E` |
| Class | `0x07` (Printer) |
| Subclass | `0x01` (Printers) |
| Protocol | `0x02` (Bidirectional) |
| Device Name | "Blure Receipt Printer" |

The POS system prints to the dongle like any receipt printer. USB bulk OUT data accumulates until 250ms of idle time, then the complete receipt is pushed to the buffer.

## Hardware

- **MCU**: ESP32-S3 Mini (Xtensa LX7 dual-core, 240 MHz)
- **Flash**: 4 MB
- **SRAM**: 512 KB
- **PSRAM**: 2 MB
- **BLE**: Bluetooth 5.0 (BLE 4.2 compatible)
- **USB**: Native USB-OTG (device mode)

### Estimated Flash Usage

| Component | Size |
|-----------|------|
| Firmware | ~400 KB |
| NimBLE stack | ~200 KB |
| TinyUSB stack | ~50 KB |
| NVS partition | ~24 KB |
| ESP-IDF runtime | ~900 KB |
| **Total** | **~1.6 MB** |
| **Available** | **4 MB** |

## Build

### Prerequisites

- [espup](https://github.com/esp-rs/espup) (installs Xtensa Rust toolchain)
- [ldproxy](https://github.com/esp-rs/embuild)
- [espflash](https://github.com/esp-rs/espflash)

```bash
# Install toolchain
espup install --targets esp32s3
cargo install ldproxy espflash

# Source environment (Windows)
. $HOME/export-esp.ps1
```

### Compile

```bash
# Debug
cargo build

# Release (size-optimized, LTO)
cargo build --release

# Flash and monitor
cargo run --release
```

### Run Tests

```bash
cargo test
```

## Configuration

Device config is stored in NVS (non-volatile storage) and persists across reboots:

| Key | Default | Purpose |
|-----|---------|---------|
| `store_name` | `"BlurE Store"` | Store name for BLE advertising |
| `register_num` | `1` | Register number |
| `device_name` | `"BRPrinter"` | BLE device name |

## Project Structure

```
Bluresp32/
├── .cargo/
│   └── config.toml                # Xtensa target, ldproxy linker
├── build.rs                       # ESP-IDF build script
├── Cargo.toml                     # Dependencies
├── sdkconfig.defaults             # NimBLE + TinyUSB enabled
├── src/
│   ├── main.rs                    # Entry point, dispatch loop
│   ├── config.rs                  # NVS-backed device config
│   ├── protocol.rs                # BLE UUIDs, framing constants
│   ├── ble/
│   │   ├── mod.rs
│   │   ├── gatt_server.rs         # NimBLE GATT server
│   │   └── heartbeat.rs           # Keepalive task
│   ├── receipt/
│   │   ├── mod.rs
│   │   ├── buffer.rs              # Thread-safe ring buffer
│   │   └── framer.rs              # Receipt → BLE frames (tested)
│   └── usb/
│       ├── mod.rs
│       └── printer_class.rs       # USB Printer Class device
├── tests/
│   └── features/                  # BDD feature files (Gherkin)
│       ├── ble_transfer.feature
│       ├── usb_ingestion.feature
│       ├── receipt_framing.feature
│       ├── heartbeat.feature
│       └── configuration.feature
├── LICENSE
└── README.md
```

## Related Repos

| Repo | Description |
|------|-------------|
| [Blure-Printer](https://github.com/BR-2-0/Blure-Printer) | Windows BLE peripheral (same protocol) |
| [Blure-Rust-Android](https://github.com/BR-2-0/Blure-Rust-Android) | Android Rust bindings |
| [Blure-Rust-iOS](https://github.com/BR-2-0/Blure-Rust-iOS) | iOS client |

## License

MIT License. See [LICENSE](LICENSE).
