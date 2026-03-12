Feature: Receipt Framing
  As the BLE transport layer
  I want receipts split into BLE-safe frames
  So that they can be delivered reliably over 182-byte notifications

  # --- Framing ---

  Scenario: Single-chunk receipt framing
    Given a receipt payload of 100 bytes
    When the receipt is framed for BLE transfer
    Then 3 frames are produced
    And frame 0 is the UTF-8 string "1"
    And frame 1 is 100 bytes of raw data
    And frame 2 is a single byte 0x04

  Scenario: Multi-chunk receipt framing
    Given a receipt payload of 500 bytes
    When the receipt is framed for BLE transfer
    Then 5 frames are produced
    And frame 0 is the UTF-8 string "3"
    And frame 1 is 182 bytes
    And frame 2 is 182 bytes
    And frame 3 is 136 bytes
    And frame 4 is 0x04

  Scenario: Exact boundary receipt framing
    Given a receipt payload of exactly 182 bytes
    When the receipt is framed for BLE transfer
    Then 3 frames are produced
    And frame 0 is "1"
    And frame 1 is exactly 182 bytes
    And frame 2 is 0x04

  Scenario: Two-chunk boundary receipt framing
    Given a receipt payload of exactly 364 bytes
    When the receipt is framed for BLE transfer
    Then 4 frames are produced
    And frame 0 is "2"
    And frames 1-2 are each 182 bytes
    And frame 3 is 0x04

  # --- Reassembly verification ---

  Scenario: Reassembled data matches original
    Given a receipt payload of 1000 bytes
    When the receipt is framed and the data frames are concatenated
    Then the concatenated result is byte-for-byte identical to the original

  # --- Validation ---

  Scenario: Empty receipt is rejected
    Given a receipt payload of 0 bytes
    When framing is attempted
    Then a FrameError::EmptyReceipt error is returned

  Scenario: Oversized receipt is rejected
    Given a receipt payload exceeding 1 MB
    When framing is attempted
    Then a FrameError::ReceiptTooLarge error is returned

  Scenario: Receipt producing too many chunks is rejected
    Given a receipt that would produce more than 10000 chunks
    When framing is attempted
    Then a FrameError::TooManyChunks error is returned
