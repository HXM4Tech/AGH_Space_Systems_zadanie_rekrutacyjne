use crc::{Crc, CRC_32_MPEG_2};

pub const MPEG2_CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_MPEG_2);

pub fn calculate_crc32(payload_10_bytes: &[u8]) -> [u8; 4] {
    let mut padded = [0u8; 12];
    padded[..10].copy_from_slice(&payload_10_bytes[..10]);

    let mut swapped = [0u8; 12];
    for i in 0..3 {
        let chunk = u32::from_le_bytes(padded[i * 4..(i + 1) * 4].try_into().unwrap());
        swapped[i * 4..(i + 1) * 4].copy_from_slice(&chunk.swap_bytes().to_le_bytes());
    }

    let checksum = MPEG2_CRC.checksum(&swapped);
    checksum.to_le_bytes()
}
