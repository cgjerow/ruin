pub fn vecbool_to_u8(bits: [bool; 8]) -> u8 {
    bits.into_iter()
        .enumerate()
        .fold(0u8, |acc, (i, bit)| if bit { acc | (1 << i) } else { acc })
}
