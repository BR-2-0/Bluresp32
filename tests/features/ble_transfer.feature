Feature: BLE Receipt Transfer
  As a BlueReceipt mobile client
  I want the ESP32-S3 dongle to deliver receipts over BLE
  So that I receive paperless receipts on my phone

  Background:
    Given the ESP32-S3 dongle is powered on
    And NimBLE is initialized
    And the GATT server is advertising as "BRPrinter"

  # --- Advertising ---

  Scenario: Dongle advertises with correct service UUID
    When a BLE scan is performed
    Then the dongle appears with service UUID 0016FF85-E965-4B40-8412-5E6215C87D29
    And the device name is "BRPrinter"

  Scenario: Dongle advertises continuously until connected
    Given no mobile client is connected
    Then the dongle continues advertising indefinitely
    And the advertising interval is within BLE specification

  # --- Connection ---

  Scenario: Mobile client connects and discovers services
    When a mobile client connects via GATT
    Then the FILE_REQ characteristic is discoverable
    And the CLIENT_ID characteristic is discoverable

  Scenario: Mobile client subscribes to FILE_REQ notifications
    Given a mobile client is connected
    When the client enables notifications on FILE_REQ (writes CCC descriptor)
    Then has_subscriber is set to true
    And the dongle is ready to send receipts

  Scenario: Subscribe timeout disconnects idle client
    Given a mobile client is connected
    And the client has not subscribed to FILE_REQ
    When 15 seconds elapse without CCC descriptor write
    Then the dongle disconnects the client
    And resumes advertising

  # --- Transfer ---

  Scenario: Single-chunk receipt is transferred correctly
    Given a mobile client is subscribed to FILE_REQ
    And a 100-byte receipt is in the buffer
    When the main loop dispatches the receipt
    Then notification 1 contains UTF-8 string "1"
    And notification 2 contains 100 bytes of receipt data
    And notification 3 contains a single byte 0x04

  Scenario: Multi-chunk receipt is transferred correctly
    Given a mobile client is subscribed to FILE_REQ
    And a 500-byte receipt is in the buffer
    When the main loop dispatches the receipt
    Then notification 1 contains UTF-8 string "3"
    And notifications 2-4 contain 182, 182, and 136 bytes respectively
    And the final notification contains 0x04

  Scenario: Notifications are paced at 100ms intervals
    Given a multi-chunk receipt is being transferred
    Then each notification is sent at least 100ms after the previous one

  Scenario: is_transferring suppresses heartbeat during transfer
    Given a receipt transfer is in progress
    Then is_transferring is set to true
    And no heartbeat bytes are sent during the transfer
    When the transfer completes
    Then is_transferring is set to false

  # --- Persistent connection ---

  Scenario: Connection stays open after transfer
    Given a receipt transfer has completed
    Then the GATT connection remains active
    And the dongle is ready for the next receipt
    And heartbeat resumes

  Scenario: Multiple receipts transfer sequentially
    Given a mobile client is subscribed
    When 3 receipts arrive in the buffer
    Then all 3 are transferred sequentially
    And each transfer is separated by the chunk count → data → EOF sequence
