//! # wiiu_swizzle
//! wiiu_swizzle is a CPU implementation of memory tiling
//! for texture surfaces for the Wii U GPU hardware.
use addrlib::AddrComputeSurfaceAddrFromCoordInput;
pub use addrlib::AddrTileMode;

mod addrlib;

/// Errors than can occur while tiling or untiling.
#[derive(Debug)]
pub enum SwizzleError {
    /// The source data does not contain enough bytes.
    NotEnoughData {
        expected_size: usize,
        actual_size: usize,
    },
}

impl std::fmt::Display for SwizzleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SwizzleError::NotEnoughData {
                expected_size,
                actual_size,
            } => write!(
                f,
                "Not enough data. Expected {} bytes but found {} bytes.",
                expected_size, actual_size
            ),
        }
    }
}

impl std::error::Error for SwizzleError {}

// TODO: Docs and examples.
/// Untile all the array layers and mipmaps in `source` to a combined vector.
///
/// For block compressed formats, `width` and `height` should be the dimensions in blocks
/// with `bytes_per_pixel` being the size of a block in bytes.
pub fn deswizzle_surface(
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    source: &[u8],
    swizzle: u32,
    pitch: u32,
    tile_mode: AddrTileMode,
    bytes_per_pixel: u32,
) -> Result<Vec<u8>, SwizzleError> {
    let output_size = width as usize
        * height as usize
        * depth_or_array_layers as usize
        * bytes_per_pixel as usize;
    if output_size == 0 {
        return Ok(Vec::new());
    }

    let expected_size = swizzled_surface_size(
        width,
        height,
        depth_or_array_layers,
        swizzle,
        pitch,
        tile_mode,
        bytes_per_pixel,
    );
    if source.len() < expected_size {
        return Err(SwizzleError::NotEnoughData {
            expected_size,
            actual_size: source.len(),
        });
    }

    let mut output = vec![0u8; output_size];

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
    )?;

    Ok(output)
}

/// Tile all the array layers and mipmaps in `source` to a combined vector.
///
/// For block compressed formats, `width` and `height` should be the dimensions in blocks
/// with `bytes_per_pixel` being the size of a block in bytes.
pub fn swizzle_surface(
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    source: &[u8],
    swizzle: u32,
    pitch: u32,
    tile_mode: AddrTileMode,
    bytes_per_pixel: u32,
) -> Result<Vec<u8>, SwizzleError> {
    // TODO: Is this the correct output size?
    let output_size = swizzled_surface_size(
        width,
        height,
        depth_or_array_layers,
        swizzle,
        pitch,
        tile_mode,
        bytes_per_pixel,
    );
    if output_size == 0 {
        return Ok(Vec::new());
    }

    let expected_size =
        deswizzled_surface_size(width, height, depth_or_array_layers, bytes_per_pixel);
    if source.len() < expected_size {
        return Err(SwizzleError::NotEnoughData {
            expected_size,
            actual_size: source.len(),
        });
    }

    let mut output = vec![0u8; output_size];

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
    )?;

    Ok(output)
}

fn deswizzled_surface_size(
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    bytes_per_pixel: u32,
) -> usize {
    width as usize * height as usize * depth_or_array_layers as usize * bytes_per_pixel as usize
}

fn swizzled_surface_size(
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    swizzle: u32,
    pitch: u32,
    tile_mode: AddrTileMode,
    bytes_per_pixel: u32,
) -> usize {
    // Addrlib code doesn't handle a bpp of 0.
    if bytes_per_pixel == 0 {
        return 0;
    }
    let bpp = bytes_per_pixel * u8::BITS;

    // TODO: name in gx2?
    let (pipe_swizzle, bank_swizzle) = addrlib::pipe_bank_swizzle(swizzle);

    // TODO: How to initialize these parameters?
    let sample = 0;
    let num_samples = 1; // TODO: is this right?
    let tile_base = 0; // TODO: only used for depth map textures?
    let comp_bits = 0; // TODO: only used for depth map textures?

    // TODO: How many of these fields are set from functions?
    // TODO: Find a way to get values used from cemu to create test cases?
    let p_in = AddrComputeSurfaceAddrFromCoordInput {
        x: width.saturating_sub(1),
        y: height.saturating_sub(1),
        slice: depth_or_array_layers.saturating_sub(1),
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

    // TODO: Will the corner always be the largest address?
    addrlib::dispatch_compute_surface_addrfrom_coord(&p_in) as usize
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
) -> Result<(), SwizzleError> {
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
    let tile_base = 0; // TODO: only used for depth map textures?
    let comp_bits = 0; // TODO: only used for depth map textures?

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
                // TODO: Fix size calculations to avoid needing these checks.
                // Assume the linear size estimation is accurate.
                if SWIZZLE {
                    let actual_size = output.len();
                    output
                        .get_mut(address..address + bytes_per_pixel as usize)
                        .ok_or(SwizzleError::NotEnoughData {
                            expected_size: address + bytes_per_pixel as usize,
                            actual_size,
                        })?
                        .copy_from_slice(
                            &source[linear_address..linear_address + bytes_per_pixel as usize],
                        );
                } else {
                    let actual_size = source.len();
                    output[linear_address..linear_address + bytes_per_pixel as usize]
                        .copy_from_slice(
                            source
                                .get(address..address + bytes_per_pixel as usize)
                                .ok_or(SwizzleError::NotEnoughData {
                                    expected_size: address + bytes_per_pixel as usize,
                                    actual_size,
                                })?,
                        );
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Add a test for micro tiling.
    // TODO: Test mipmaps
    #[test]
    fn deswizzle_empty() {
        assert!(deswizzle_surface(
            0,
            0,
            0,
            &[],
            853504,
            256,
            AddrTileMode::ADDR_TM_2D_TILED_THIN1,
            8
        )
        .unwrap()
        .is_empty());
    }

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
            )
            .unwrap()[..]
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
            )
            .unwrap()[..]
        );
    }

    // TODO: Test cube maps
}
