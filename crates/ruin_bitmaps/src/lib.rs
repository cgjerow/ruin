pub fn vecbool_to_u8(bits: [bool; 8]) -> u8 {
    bits.iter()
        .enumerate()
        .fold(0u8, |acc, (i, bit)| if *bit { acc | (1 << i) } else { acc })
}

pub type MaskLayerBitmap = u8;

pub fn masks_overlap_layers(a: MaskLayerBitmap, b: MaskLayerBitmap) -> bool {
    a & b > 0
}
