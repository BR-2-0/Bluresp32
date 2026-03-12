Feature: BLE Heartbeat
  As a mobile client connected to the ESP32-S3 dongle
  I want the dongle to send periodic heartbeat bytes
  So that the BLE connection stays alive between receipts

  Background:
    Given the ESP32-S3 dongle is powered on
    And a mobile client is connected and subscribed to FILE_REQ

  # --- Heartbeat sending ---

  Scenario: Heartbeat sends 0x00 every 4 seconds
    Given the dongle is idle (no receipt transfer in progress)
    When 4 seconds elapse
    Then a single byte 0x00 is sent as a FILE_REQ notification
    And this repeats every 4 seconds

  Scenario: Heartbeat suppressed during transfer
    Given a receipt transfer is in progress (is_transferring == true)
    Then no heartbeat bytes are sent
    When the transfer completes and is_transferring resets to false
    Then heartbeat resumes on the next 4-second tick

  Scenario: Heartbeat only sent when subscriber exists
    Given no mobile client is subscribed (has_subscriber == false)
    Then no heartbeat bytes are sent
    When a client subscribes to FILE_REQ
    Then heartbeat begins on the next 4-second tick

  # --- Connection keep-alive ---

  Scenario: Heartbeat prevents idle disconnect
    Given the mobile client expects activity within 5 minutes
    And the dongle sends heartbeat every 4 seconds
    Then the connection remains active indefinitely

  Scenario: Missing heartbeat causes mobile-side timeout
    Given the heartbeat task is not running
    And no receipt transfer occurs
    When 5 minutes elapse with no notifications
    Then the mobile client's idle timer expires
    And the mobile client disconnects

  # --- Thread safety ---

  Scenario: Heartbeat task runs on dedicated FreeRTOS thread
    When the heartbeat is started
    Then it runs on a thread named "heartbeat"
    And the thread stack is 2048 bytes
    And it does not block the main dispatch loop
