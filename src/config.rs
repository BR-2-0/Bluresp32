/// NVS-backed configuration for the Bluresp32 dongle.
///
/// Stores device identity in ESP32 flash so it persists across reboots.
/// Configurable via USB serial console on first boot.

use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};
use log::{info, warn};

const NVS_NAMESPACE: &str = "blure_cfg";
const KEY_STORE_NAME: &str = "store_name";
const KEY_REGISTER_NUM: &str = "register_num";
const KEY_DEVICE_NAME: &str = "device_name";

const DEFAULT_DEVICE_NAME: &str = "BRPrinter";

pub struct BlureConfig {
    pub store_name: String,
    pub register_number: String,
    pub device_name: String,
}

impl BlureConfig {
    /// Load config from NVS. Returns defaults if keys are missing.
    pub fn load(nvs_partition: EspDefaultNvsPartition) -> Self {
        let nvs = match EspNvs::<NvsDefault>::new(nvs_partition, NVS_NAMESPACE, true) {
            Ok(nvs) => nvs,
            Err(e) => {
                warn!("Failed to open NVS namespace: {:?}, using defaults", e);
                return Self::defaults();
            }
        };

        let store_name = read_string(&nvs, KEY_STORE_NAME).unwrap_or_default();
        let register_number = read_string(&nvs, KEY_REGISTER_NUM).unwrap_or_default();
        let device_name =
            read_string(&nvs, KEY_DEVICE_NAME).unwrap_or_else(|| DEFAULT_DEVICE_NAME.to_string());

        info!(
            "Config loaded: store={:?}, register={:?}, device={:?}",
            store_name, register_number, device_name
        );

        Self {
            store_name,
            register_number,
            device_name,
        }
    }

    /// Save current config to NVS.
    pub fn save(&self, nvs_partition: EspDefaultNvsPartition) -> Result<(), esp_idf_sys::EspError> {
        let mut nvs = EspNvs::<NvsDefault>::new(nvs_partition, NVS_NAMESPACE, true)?;

        nvs.set_str(KEY_STORE_NAME, &self.store_name)?;
        nvs.set_str(KEY_REGISTER_NUM, &self.register_number)?;
        nvs.set_str(KEY_DEVICE_NAME, &self.device_name)?;

        info!("Config saved to NVS");
        Ok(())
    }

    /// Build the BLE advertising name from store + register, or fall back to device_name.
    pub fn ble_name(&self) -> String {
        if !self.store_name.is_empty() && !self.register_number.is_empty() {
            format!("{} / Register {}", self.store_name, self.register_number)
        } else if !self.store_name.is_empty() {
            self.store_name.clone()
        } else {
            self.device_name.clone()
        }
    }

    fn defaults() -> Self {
        Self {
            store_name: String::new(),
            register_number: String::new(),
            device_name: DEFAULT_DEVICE_NAME.to_string(),
        }
    }
}

/// Read a string value from NVS. Returns None if key doesn't exist.
fn read_string(nvs: &EspNvs<NvsDefault>, key: &str) -> Option<String> {
    // First call to get required buffer size
    let len = match nvs.str_len(key) {
        Ok(Some(len)) => len,
        _ => return None,
    };

    let mut buf = vec![0u8; len];
    match nvs.get_str(key, &mut buf) {
        Ok(Some(val)) => Some(val.to_string()),
        _ => None,
    }
}
