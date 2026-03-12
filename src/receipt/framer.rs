/// Receipt framing — port of blure-core::protocol::frame_receipt().
///
/// Splits raw ESC/POS receipt data into the notification sequence
/// expected by Android/iOS clients:
///
///   Frame 0:     UTF-8 chunk count string (e.g. "3")
///   Frames 1-N:  Raw data in 182-byte chunks (last may be shorter)
///   Frame N+1:   EOF marker [0x04]

use crate::protocol::*;

#[derive(Debug)]
pub enum FrameError {
    EmptyReceipt,
    ReceiptTooLarge,
    TooManyChunks,
}

/// Split receipt data into BLE notification frames.
///
/// Mirrors `blure_core::protocol::frame_receipt()` exactly.
pub fn frame_receipt(data: &[u8]) -> Result<Vec<Vec<u8>>, FrameError> {
    if data.is_empty() {
        return Err(FrameError::EmptyReceipt);
    }
    if data.len() > MAX_RECEIPT_BYTES {
        return Err(FrameError::ReceiptTooLarge);
    }

    let chunk_count = (data.len() + CHUNK_SIZE - 1) / CHUNK_SIZE;
    if chunk_count > MAX_CHUNKS {
        return Err(FrameError::TooManyChunks);
    }

    let mut frames = Vec::with_capacity(chunk_count + 2);

    // Frame 0: chunk count as UTF-8 string.
    frames.push(chunk_count.to_string().into_bytes());

    // Frames 1..N: raw data chunks.
    for chunk in data.chunks(CHUNK_SIZE) {
        frames.push(chunk.to_vec());
    }

    // Final frame: EOF marker.
    frames.push(vec![EOF_MARKER]);

    Ok(frames)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_chunk() {
        let data = vec![0xAA; 100];
        let frames = frame_receipt(&data).unwrap();
        assert_eq!(frames.len(), 3);
        assert_eq!(frames[0], b"1");
        assert_eq!(frames[1], data);
        assert_eq!(frames[2], vec![EOF_MARKER]);
    }

    #[test]
    fn multi_chunk() {
        let data = vec![0xBB; 500];
        let frames = frame_receipt(&data).unwrap();
        assert_eq!(frames.len(), 5); // count + 3 chunks + eof
        assert_eq!(frames[0], b"3");
        assert_eq!(frames[1].len(), CHUNK_SIZE);
        assert_eq!(frames[2].len(), CHUNK_SIZE);
        assert_eq!(frames[3].len(), 500 - 2 * CHUNK_SIZE);
        assert_eq!(frames[4], vec![EOF_MARKER]);
    }

    #[test]
    fn exact_boundary() {
        let data = vec![0xCC; CHUNK_SIZE];
        let frames = frame_receipt(&data).unwrap();
        assert_eq!(frames.len(), 3);
        assert_eq!(frames[0], b"1");
    }

    #[test]
    fn empty_rejected() {
        assert!(frame_receipt(&[]).is_err());
    }

    #[test]
    fn oversized_rejected() {
        let data = vec![0; MAX_RECEIPT_BYTES + 1];
        assert!(frame_receipt(&data).is_err());
    }

    #[test]
    fn reassembly_matches_original() {
        let data = vec![0xDD; 1000];
        let frames = frame_receipt(&data).unwrap();
        let reassembled: Vec<u8> = frames[1..frames.len() - 1]
            .iter()
            .flat_map(|f| f.iter().copied())
            .collect();
        assert_eq!(reassembled, data);
    }
}
