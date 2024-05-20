//! # wiiu_swizzle
//! wiiu_swizzle is a CPU implementation of memory tiling
//! for texture surfaces for the Wii U GPU hardware.
use addrlib::AddrComputeSurfaceAddrFromCoordInput;
pub use addrlib::TileMode;

// TODO: Don't make this public.
pub use addrlib::compute_surface_mip_level_tile_mode;

mod addrlib;

/// Errors than can occur while converting between tiled and linear memory layouts.
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

// TODO: Include all gx2 enum variants?
#[derive(Debug)]
pub enum AaMode {
    X1 = 0,
    X2 = 1,
    X4 = 2,
    X8 = 3,
}

#[derive(Debug)]
pub enum SurfaceFormat {
    R8G8B8A8Unorm = 26,
    BC1Unorm = 49,
    BC2Unorm = 50,
    BC3Unorm = 51,
    BC4Unorm = 52,
    BC5Unorm = 53,
}

impl SurfaceFormat {
    pub fn block_dim(&self) -> (u32, u32) {
        match self {
            SurfaceFormat::R8G8B8A8Unorm => (1, 1),
            SurfaceFormat::BC1Unorm => (4, 4),
            SurfaceFormat::BC2Unorm => (4, 4),
            SurfaceFormat::BC3Unorm => (4, 4),
            SurfaceFormat::BC4Unorm => (4, 4),
            SurfaceFormat::BC5Unorm => (4, 4),
        }
    }

    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            SurfaceFormat::R8G8B8A8Unorm => 4,
            SurfaceFormat::BC1Unorm => 8,
            SurfaceFormat::BC2Unorm => 16,
            SurfaceFormat::BC3Unorm => 16,
            SurfaceFormat::BC4Unorm => 8,
            SurfaceFormat::BC5Unorm => 16,
        }
    }
}

#[derive(Debug)]
pub enum SurfaceDim {
    D1 = 0,
    D2 = 1,
    D3 = 2,
    Cube = 3,
}

// TODO: How to handle array layers?
// TODO: additional enums?
#[derive(Debug)]
pub struct Gx2Surface<'a> {
    // TODO: Is this even used?
    pub dim: SurfaceDim,
    /// The width of the base mip level in pixels.
    pub width: u32,
    /// The height of the base mip level in pixels.
    pub height: u32,
    /// The depth of the base mip level in pixels.
    pub depth: u32,
    pub mipmap_count: u32,
    pub format: SurfaceFormat,
    pub aa: AaMode,
    pub usage: u32,
    pub image_data: &'a [u8],
    pub mipmap_data: &'a [u8],
    pub tile_mode: TileMode,
    pub swizzle: u32,
    pub alignment: u32,
    pub pitch: u32,
    pub mipmap_offsets: [u32; 13],
}

// TODO: Also define a swizzle surface?
impl<'a> Gx2Surface<'a> {
    /// Convert all layers and mipmaps from tiled to a combined linear vector.
    pub fn deswizzle(&self) -> Result<Vec<u8>, SwizzleError> {
        let (block_width, block_height) = self.format.block_dim();
        let bytes_per_pixel = self.format.bytes_per_pixel();

        let div_round_up = |x, d| (x + d - 1) / d;

        // TODO: Add tests cases for mipmap offsets?
        let mut data = Vec::new();
        for mip in 0..self.mipmap_count {
            let source = if mip == 0 {
                // The mip 0 data is at the start of the image data.
                self.image_data
            } else if mip == 1 {
                // The slice already accounts for the mip 1 offset.
                // TODO: Set the end of the range?
                self.mipmap_data
            } else {
                // Remaining mip levels are relative to the start of the mipmap data.
                // TODO: Set the end of the range?
                let offset = self.mipmap_offsets[mip as usize - 1] as usize;
                &self.mipmap_data[offset..]
            };

            // TODO: How to handle dimensions not divisible by block dimensions?
            // TODO: cemu uses mipPtr & 0x700 for swizzle for mipmaps?
            let width = div_round_up(self.width, block_width) >> mip;
            let height = div_round_up(self.height, block_height) >> mip;
            let pitch = self.pitch >> mip;

            // Some mipmaps need to be micro tiled instead of macro tiled.
            let tile_mode = compute_surface_mip_level_tile_mode(
                self.tile_mode,
                bytes_per_pixel * u8::BITS,
                mip,
                width,
                height,
                1,
                1,
                false,
                false,
            );

            // TODO: Handle mipmaps smaller than the block dimensions?
            let mip = deswizzle_mipmap(
                width,
                height,
                1,
                source,
                self.swizzle,
                pitch,
                tile_mode,
                bytes_per_pixel,
            )?;
            data.extend_from_slice(&mip);
        }

        Ok(data)
    }
}

// TODO: Docs and examples.
/// Convert the tiled data in `source` to a combined linear vector.
///
/// For block compressed formats, `width` and `height` should be the dimensions in blocks
/// with `bytes_per_pixel` being the size of a block in bytes.
pub fn deswizzle_mipmap(
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    source: &[u8],
    swizzle: u32,
    pitch: u32,
    tile_mode: TileMode,
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

/// Convert the linear data in `source` to a combined tiled vector.
///
/// For block compressed formats, `width` and `height` should be the dimensions in blocks
/// with `bytes_per_pixel` being the size of a block in bytes.
pub fn swizzle_mipmap(
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    source: &[u8],
    swizzle: u32,
    pitch: u32,
    tile_mode: TileMode,
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

// TODO: Should this use ComputeSurfaceInfo functions from addrlib?
//https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L1198
fn swizzled_surface_size(
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    swizzle: u32,
    pitch: u32,
    tile_mode: TileMode,
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
    tile_mode: TileMode,
    bytes_per_pixel: u32,
) -> Result<(), SwizzleError> {
    // TODO: validate dimensions?
    // TODO: compute surface info to fill in these params?
    // TODO: rounding or padding of dimensions?
    // TODO: handle div round up based on block dimensions?

    // TODO: always bytes per pixel * 8?
    let bpp = bytes_per_pixel * u8::BITS;

    // TODO: name in gx2?
    let (pipe_swizzle, bank_swizzle) = addrlib::pipe_bank_swizzle(swizzle);

    // TODO: How to initialize these parameters?
    let sample = 0;
    let num_samples = 1; // TODO: is this based on self.aa?
    let tile_base = 0; // TODO: only used for depth map textures?
    let comp_bits = 0; // TODO: only used for depth map textures?

    // TODO: addrlib uses input and output structs to "dispatch" swizzling?
    // TODO: only the input pin values matter?
    // TODO: cemu uses this structure as well?
    // TODO: should these define the public API?

    // TODO: Is it correct to use depth and layers as slices?
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
                    // TODO: This should never be out of bounds on valid inputs?
                    output[address..address + bytes_per_pixel as usize].copy_from_slice(
                        &source[linear_address..linear_address + bytes_per_pixel as usize],
                    );
                } else {
                    // TODO: This should never be out of bounds on valid inputs?
                    output[linear_address..linear_address + bytes_per_pixel as usize]
                        .copy_from_slice(&source[address..address + bytes_per_pixel as usize]);
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
        assert!(deswizzle_mipmap(
            0,
            0,
            0,
            &[],
            853504,
            256,
            TileMode::ADDR_TM_2D_TILED_THIN1,
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
            &deswizzle_mipmap(
                1024 / 4,
                1024 / 4,
                1,
                swizzled,
                853504,
                256,
                TileMode::ADDR_TM_2D_TILED_THIN1,
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
            &deswizzle_mipmap(
                16,
                16,
                16,
                swizzled,
                852224,
                32,
                TileMode::ADDR_TM_2D_TILED_THICK,
                4
            )
            .unwrap()[..]
        );
    }

    // TODO: Test cube maps
}
