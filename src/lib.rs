// TODO: module docs
use addrlib::AddrComputeSurfaceAddrFromCoordInput;
pub use addrlib::AddrTileMode;

mod addrlib;

// TODO: Docs and examples.
/// Untile all the array layers and mipmaps in `source` to a combined vector.
pub fn deswizzle_surface(
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    source: &[u8],
    swizzle: u32,
    pitch: u32,
    tile_mode: AddrTileMode,
    bytes_per_pixel: u32,
) -> Vec<u8> {
    let mut output = vec![
        0u8;
        width as usize
            * height as usize
            * depth_or_array_layers as usize
            * bytes_per_pixel as usize
    ];

    swizzle_surface_inner::<false>(
        width,
        height,
        depth_or_array_layers,
        source,
        &mut output,
        swizzle,
        pitch,
        tile_mode,
        bytes_per_pixel,
    );

    output
}

/// Tile all the array layers and mipmaps in `source` to a combined vector.
pub fn swizzle_surface(
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    source: &[u8],
    swizzle: u32,
    pitch: u32,
    tile_mode: AddrTileMode,
    bytes_per_pixel: u32,
) -> Vec<u8> {
    // TODO: Is this the correct output size?
    let mut output = vec![
        0u8;
        width as usize
            * height as usize
            * depth_or_array_layers as usize
            * bytes_per_pixel as usize
    ];

    swizzle_surface_inner::<true>(
        width,
        height,
        depth_or_array_layers,
        source,
        &mut output,
        swizzle,
        pitch,
        tile_mode,
        bytes_per_pixel,
    );

    output
}

fn swizzle_surface_inner<const SWIZZLE: bool>(
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    source: &[u8],
    output: &mut [u8],
    swizzle: u32,
    pitch: u32,
    tile_mode: AddrTileMode,
    bytes_per_pixel: u32,
) {
    // TODO: validate dimensions?
    // TODO: surface info to fill in these params?
    // TODO: rounding or padding of dimensions?
    // TODO: handle div round up based on block dimensions?

    // TODO: always bytes per pixel * 8?
    let bpp = bytes_per_pixel * u8::BITS;

    // TODO: name in gx2?
    let (pipe_swizzle, bank_swizzle) = addrlib::pipe_bank_swizzle(swizzle);

    // TODO: How to initialize these parameters?
    let sample = 0;
    let num_samples = 1; // TODO: is this right?
    let tile_base = 0; // TODO: only used for depth textures?
    let comp_bits = 0; // TODO: only used for depth textures?

    // TODO: addrlib uses input and output structs to "dispatch" swizzling?
    // TODO: only the input pin values matter?
    // TODO: cemu uses this structure as well?
    // TODO: should these define the public API?

    // TODO: array layers?
    // TODO: Is it correct to use depth as slices?
    for z in 0..depth_or_array_layers {
        for y in 0..height {
            for x in 0..width {
                // TODO: How many of these fields are set from functions?
                // TODO: Find a way to get values used from cemu to create test cases?
                let p_in = AddrComputeSurfaceAddrFromCoordInput {
                    x,
                    y,
                    slice: z,
                    sample,
                    bpp,
                    pitch,
                    height,
                    num_slices: depth_or_array_layers,
                    num_samples,
                    tile_mode,
                    is_depth: false,
                    tile_base,
                    comp_bits,
                    pipe_swizzle,
                    bank_swizzle,
                };

                let address = addrlib::dispatch_compute_surface_addrfrom_coord(&p_in) as usize;
                let linear_address =
                    ((z * width * height + y * width + x) * bytes_per_pixel) as usize;
                if SWIZZLE {
                    output[address..address + bytes_per_pixel as usize].copy_from_slice(
                        &source[linear_address..linear_address + bytes_per_pixel as usize],
                    );
                } else {
                    output[linear_address..linear_address + bytes_per_pixel as usize]
                        .copy_from_slice(&source[address..address + bytes_per_pixel as usize]);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Add a test for micro tiling.
    // TODO: Test mipmaps
    #[test]
    fn deswizzled_macro_tiled_1024x1024_bc1() {
        let expected = include_bytes!("data/1024x1024_bc1_tm4_p256_s853504_deswizzled.bin");
        let swizzled = include_bytes!("data/1024x1024_bc1_tm4_p256_s853504_swizzled.bin");

        assert_eq!(
            expected,
            &deswizzle_surface(
                1024 / 4,
                1024 / 4,
                1,
                swizzled,
                853504,
                256,
                AddrTileMode::ADDR_TM_2D_TILED_THIN1,
                8
            )[..]
        );
    }

    #[test]
    fn deswizzled_macro_tiled_16x16x16_rgba8() {
        let expected = include_bytes!("data/16x16x16_rgba8_tm7_p32_s852224_deswizzled.bin");
        let swizzled = include_bytes!("data/16x16x16_rgba8_tm7_p32_s852224_swizzled.bin");

        assert_eq!(
            expected,
            &deswizzle_surface(
                16,
                16,
                16,
                swizzled,
                852224,
                32,
                AddrTileMode::ADDR_TM_2D_TILED_THICK,
                4
            )[..]
        );
    }

    // TODO: Test cube maps
}
