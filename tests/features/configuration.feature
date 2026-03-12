Feature: Device Configuration
  As a store operator deploying BlurE ESP32-S3 dongles
  I want persistent device configuration stored in flash
  So that each dongle remembers its identity across reboots

  Background:
    Given the ESP32-S3 dongle is powered on
    And the NVS partition is initialized

  # --- Default configuration ---

  Scenario: First boot uses default configuration
    Given no configuration exists in NVS
    When BlureConfig::load() is called
    Then the config uses defaults:
      | field          | value          |
      | store_name     | BlurE Store    |
      | register_number| 1              |
      | device_name    | BRPrinter      |

  Scenario: Default BLE name is "BRPrinter"
    Given default configuration is loaded
    When ble_name() is called
    Then the result is "BRPrinter"

  # --- Persistence ---

  Scenario: Configuration persists across reboots
    Given store_name is set to "Corner Bakery" and register_number to 3
    When BlureConfig::save() is called
    And the dongle reboots
    And BlureConfig::load() is called
    Then store_name is "Corner Bakery"
    And register_number is 3

  Scenario: BLE name reflects custom config
    Given store_name is "Corner Bakery" and register_number is 3
    When ble_name() is called
    Then the result is "BRPrinter-CornerBakery-3"

  # --- NVS storage ---

  Scenario: Config keys are stored in NVS namespace "blure_cfg"
    When configuration is saved
    Then NVS entries are written under namespace "blure_cfg"
    And keys include "store_name", "register_num", and "device_name"

  Scenario: Corrupted NVS falls back to defaults
    Given NVS contains corrupted data for "store_name"
    When BlureConfig::load() is called
    Then the corrupted field falls back to its default value
    And the other fields load normally
