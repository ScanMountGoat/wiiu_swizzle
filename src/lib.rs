// TODO: module docs
use addrlib::AddrComputeSurfaceAddrFromCoordInput;
pub use addrlib::AddrTileMode;

mod addrlib;

// TODO: Docs and examples.
pub fn deswizzle_surface(
    width: u32,
    height: u32,
    depth: u32,
    source: &[u8],
    swizzle: u32,
    pitch: u32,
    tile_mode: AddrTileMode,
    bytes_per_pixel: u32,
) -> Vec<u8> {
    // TODO: validate dimensions?
    // TODO: surface info to fill in these params?
    // TODO: rounding or padding of dimensions?
    // TODO: handle div round up based on block dimensions?

    // TODO: always bytes per pixel * 8?
    let bpp = bytes_per_pixel * u8::BITS;

    // TODO: name in gx2?
    let (pipe_swizzle, bank_swizzle) = addrlib::pipe_bank_swizzle(swizzle);

    // TODO: How to initialize these parameters?
    let slice = 0;
    let sample = 0;
    let num_samples = 0; // TODO: is this right?
    let tile_base = 0; // TODO: only used for depth?
    let comp_bits = 0; // TODO: only used for depth?

    // TODO: addrlib uses input and output structs to "dispatch" swizzling?
    // TODO: only the input pin values matter?
    // TODO: cemu uses this structure as well?
    // TODO: should these define the public API?

    let mut output =
        vec![0u8; width as usize * height as usize * depth as usize * bytes_per_pixel as usize];

    // TODO: depth and array layers?
    for y in 0..height {
        for x in 0..width {
            // TODO: How many of these fields are set from functions?
            // TODO: Find a way to get values used from cemu to create test cases?
            let p_in = AddrComputeSurfaceAddrFromCoordInput {
                size: 0, // TODO: is this important?
                x,
                y,
                slice,
                sample,
                bpp,
                pitch,
                height,
                num_slices: 1,
                num_samples,
                tile_mode,
                is_depth: false,
                tile_base,
                comp_bits,
                pipe_swizzle,
                bank_swizzle,
                num_frags: 0, // TODO: unused?
            };

            let address = addrlib::dispatch_compute_surface_addrfrom_coord(&p_in) as usize;
            let linear_address = ((y * width + x) * bytes_per_pixel) as usize;
            output[linear_address..linear_address + bytes_per_pixel as usize]
                .copy_from_slice(&source[address..address + bytes_per_pixel as usize]);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Add a test for micro tiling.
    #[test]
    fn deswizzled_macro_tiled_1024x1024_bc1() {
        let expected = include_bytes!("data/1024_bc1_tm_4_p_256_s_853504_deswizzled.bin");
        let swizzled = include_bytes!("data/1024_bc1_tm_4_p_256_s_853504_swizzled.bin");

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
}
