use std::ops::Range;

use crate::M4AType;

pub(crate) fn construct_adts_header(
    object_type: M4AType,
    sample_freq_index: u8,
    channel_config: u8,
    num_bytes: usize,
) -> Vec<u8> {
    // ADTS header wiki reference: https://wiki.multimedia.cx/index.php/ADTS#:~:text=Audio%20Data%20Transport%20Stream%20(ADTS,to%20stream%20audio%2C%20usually%20AAC.

    // byte7 and byte9 not included without CRC
    let adts_header_length = 7;

    // AAAA_AAAA
    let byte0 = 0b1111_1111;

    // AAAA_BCCD
    // D: Only support 1 (without CRC)
    let byte1 = 0b1111_0001;

    // EEFF_FFGH
    let mut byte2 = 0b0000_0000;

    let adts_object_type = object_type as u8 - 1;
    byte2 = (byte2 << 2) | adts_object_type; // EE

    byte2 = (byte2 << 4) | sample_freq_index; // FFFF
    byte2 = (byte2 << 1) | 0b1; // G
    byte2 = (byte2 << 1) | get_bits_u8(channel_config, 6..6); // H

    // HHIJ_KLMM
    let mut byte3 = 0b0000_0000;
    byte3 = (byte3 << 2) | get_bits_u8(channel_config, 7..8); // HH
    byte3 = (byte3 << 4) | 0b1111; // IJKL

    let frame_length = adts_header_length + num_bytes as u16;
    byte3 = (byte3 << 2) | get_bits_u16(frame_length, 3..5) as u8; // MM

    // MMMM_MMMM
    let byte4 = get_bits_u16(frame_length, 6..13) as u8;

    // MMMO_OOOO
    let mut byte5 = 0b0000_0000;
    byte5 = (byte5 << 3) | get_bits_u16(frame_length, 14..16) as u8;
    byte5 = (byte5 << 5) | 0b11111; // OOOOO

    // OOOO_OOPP
    let mut byte6 = 0b0000_0000;
    byte6 = (byte6 << 6) | 0b111111; // OOOOOO
    byte6 <<= 2; // PP

    vec![byte0, byte1, byte2, byte3, byte4, byte5, byte6]
}

fn get_bits_u16(byte: u16, range: Range<u16>) -> u16 {
    let shaved_left = byte << (range.start - 1);
    let moved_back = shaved_left >> (range.start - 1);
    moved_back >> (16 - range.end)
}

fn get_bits_u8(byte: u8, range: Range<u8>) -> u8 {
    let shaved_left = byte << (range.start - 1);
    let moved_back = shaved_left >> (range.start - 1);
    moved_back >> (8 - range.end)
}
