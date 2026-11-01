use crate::Garment;

pub fn next_style_id(garment: &Garment) -> u32 {
    garment.styles.iter().map(|x| x.id).max().unwrap_or(0) + 1
}

pub fn next_hardware_id(garment: &Garment) -> u32 {
    garment.hardware.iter().map(|x| x.id).max().unwrap_or(0) + 1
}

pub fn next_zipper_id(garment: &Garment) -> u32 {
    garment.zippers.iter().map(|x| x.id).max().unwrap_or(0) + 1
}

pub fn next_stitch_id(garment: &Garment) -> u32 {
    garment.stitches.iter().map(|x| x.id).max().unwrap_or(0) + 1
}

pub fn next_finish_id(garment: &Garment) -> u32 {
    garment.finishes.iter().map(|x| x.id).max().unwrap_or(0) + 1
}

pub fn next_print_id(garment: &Garment) -> u32 {
    garment.panels.iter().flat_map(|p| p.prints.iter().map(|x| x.id)).max().unwrap_or(0) + 1
}
