#![no_main]
use libfuzzer_sys::fuzz_target;

extern crate arbitrary;
use arbitrary::{Arbitrary, Result, Unstructured};

#[derive(Debug)]
struct Input {
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    swizzle: u32, // TODO: only set bits in 0x700
    pitch: u32,
    tile_mode: wiiu_swizzle::TileMode,
    bytes_per_pixel: u32,
    aa: wiiu_swizzle::AaMode,
}

impl<'a> Arbitrary<'a> for Input {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        Ok(Input {
            width: u.int_in_range(0..=256)?,
            height: u.int_in_range(0..=256)?,
            depth_or_array_layers: u.int_in_range(0..=32)?,
            swizzle: u.int_in_range(0..=7)? << 8, // only 0x700 is used
            pitch: u.int_in_range(0..=256)?,
            tile_mode: u.arbitrary()?,
            bytes_per_pixel: u.int_in_range(0..=32)?,
            aa: u.arbitrary()?,
        })
    }
}

fuzz_target!(|input: Input| {
    let size = input.width as usize
        * input.height as usize
        * input.depth_or_array_layers as usize
        * input.bytes_per_pixel as usize;

    let swizzled = vec![0u8; size];

    let _ = wiiu_swizzle::deswizzle_mipmap(
        input.width,
        input.height,
        input.depth_or_array_layers,
        &swizzled,
        input.swizzle,
        input.pitch,
        input.tile_mode,
        input.bytes_per_pixel,
        input.aa,
    );
});
