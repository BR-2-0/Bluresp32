Feature: USB Printer Class Ingestion
  As a POS system operator
  I want the ESP32-S3 dongle to appear as a USB receipt printer
  So that the POS system sends receipts to it without custom drivers

  Background:
    Given the ESP32-S3 dongle is connected to a POS system via USB
    And TinyUSB is initialized as a Printer Class device

  # --- USB enumeration ---

  Scenario: Dongle enumerates as a USB printer
    When the POS system enumerates USB devices
    Then the dongle appears with VID 0x1209 and PID 0xB10E
    And USB class is 0x07 (Printer)
    And the product string is "Blure Receipt Printer"

  Scenario: No custom driver required
    Given the POS system has standard USB printer support
    Then the dongle is recognized without additional drivers
    And the POS system can print to it like any receipt printer

  # --- Receipt detection ---

  Scenario: USB bulk OUT data is accumulated
    When the POS system sends ESC/POS data in multiple USB packets
    Then all packets are appended to the staging buffer

  Scenario: Idle timeout detects receipt boundary
    Given ESC/POS data is being received
    When 250ms elapse with no new USB data
    Then the staging buffer is treated as a complete receipt
    And the receipt is pushed to the ReceiptBuffer

  Scenario: Back-to-back receipts are separated
    Given the POS system sends receipt A
    And 250ms of idle time passes
    And the POS system sends receipt B
    Then receipt A and receipt B are pushed as separate entries in the buffer

  # --- Buffer pressure ---

  Scenario: Buffer accepts receipts up to capacity
    Given the ReceiptBuffer has capacity 8
    When 8 receipts are pushed
    Then all 8 are stored in the buffer
    And none are dropped

  Scenario: Buffer rejects when full
    Given the ReceiptBuffer is at capacity (8 entries)
    When a new receipt arrives from USB
    Then the push returns false
    And the receipt is logged as dropped

  # --- Edge cases ---

  Scenario: Empty USB transfer is ignored
    When the POS system sends zero bytes followed by idle timeout
    Then no receipt is pushed to the buffer

  Scenario: Large receipt within limits is accepted
    When the POS system sends a receipt of 500 KB
    Then the receipt is pushed to the buffer successfully
    And it will be framed into multiple BLE chunks
