#![no_main]
use libfuzzer_sys::fuzz_target;

extern crate arbitrary;
use arbitrary::{Arbitrary, Result, Unstructured};

extern crate rand;
use rand::{rngs::StdRng, Rng, SeedableRng};

#[derive(Debug)]
struct Input {
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    swizzle: u32,
    pitch: u32,
    tile_mode: wiiu_swizzle::AddrTileMode,
    bytes_per_pixel: u32,
}

impl<'a> Arbitrary<'a> for Input {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        Ok(Input {
            width: u.int_in_range(0..=256)?,
            height: u.int_in_range(0..=256)?,
            depth_or_array_layers: u.int_in_range(0..=256)?,
            swizzle: u.arbitrary()?,
            pitch: u.int_in_range(0..=256)?,
            tile_mode: u.arbitrary()?,
            bytes_per_pixel: u.int_in_range(0..=32)?,
        })
    }
}

fuzz_target!(|input: Input| {
    let deswizzled_size =
        input.width * input.height * input.depth_or_array_layers * input.bytes_per_pixel;

    let seed = [13u8; 32];
    let mut rng: StdRng = SeedableRng::from_seed(seed);
    let deswizzled: Vec<_> = (0..deswizzled_size)
        .map(|_| rng.gen_range::<u8, _>(0..=255))
        .collect();

    let swizzled = wiiu_swizzle::swizzle_surface(
        input.width,
        input.height,
        input.depth_or_array_layers,
        &deswizzled,
        input.swizzle,
        input.pitch,
        input.tile_mode,
        input.bytes_per_pixel,
    );

    let new_deswizzled = wiiu_swizzle::deswizzle_surface(
        input.width,
        input.height,
        input.depth_or_array_layers,
        &swizzled,
        input.swizzle,
        input.pitch,
        input.tile_mode,
        input.bytes_per_pixel,
    );

    if deswizzled != new_deswizzled {
        panic!("Swizzle deswizzle is not 1:1");
    }
});
