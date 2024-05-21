//! # wiiu_swizzle
//! wiiu_swizzle is a CPU implementation of memory tiling
//! for texture surfaces for the Wii U GPU hardware.
//!
//! Most applications should construct a [Gx2Surface] and use [Gx2Surface::deswizzle]
//! to correctly handle offsets and parameter changes for different mip levels.
pub use addrlib::TileMode;
use addrlib::{
    hwl_compute_surface_info, ComputeSurfaceAddrFromCoordInput, ComputeSurfaceInfoInput,
    ComputeSurfaceInfoOutput,
};

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

// TODO: Use try into and avoid panic.
#[macro_export]
macro_rules! c_enum {
    (#[$attr1:meta] $name:ident, $($(#[$attr2:meta])* $variant:ident=$value:expr),*,) => {
        #[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub enum $name {
            $(
                $(#[$attr2])*
                $variant = $value
            ),*
        }

        impl From<u32> for $name {
            fn from(value: u32) -> Self {
                match value {
                    $(
                        $value => Self::$variant,
                    )*
                    _ => panic!("No variant found for {value}")
                }
            }
        }
    };
}

// TODO: Include all gx2 enum variants?
c_enum! {
    /// GX2AAMode for the number of samples
    AaMode,
    X1 = 0,
    X2 = 1,
    X4 = 2,
    X8 = 3,
}

// TODO: link to addrlib
// TODO: include all variants.
// TODO: Parse the list with Python to generate the Rust enum
// The GX2 and addrlib enums are the same.
c_enum! {
    /// GX2SurfaceFormat
    SurfaceFormat,
    /// GX2_SURFACE_FORMAT_TCS_R8_G8_B8_A8_UNORM
    R8G8B8A8Unorm = 26,
    /// GX2_SURFACE_FORMAT_T_BC1_UNORM
    BC1Unorm = 49,
    /// GX2_SURFACE_FORMAT_T_BC2_UNORM
    BC2Unorm = 50,
    /// GX2_SURFACE_FORMAT_T_BC3_UNORM
    BC3Unorm = 51,
    /// GX2_SURFACE_FORMAT_T_BC4_UNORM
    BC4Unorm = 52,
    /// GX2_SURFACE_FORMAT_T_BC5_UNORM
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

c_enum! {
    /// GX2SurfaceDim
    SurfaceDim,
    D1 = 0,
    D2 = 1,
    D3 = 2,
    Cube = 3,
}

// TODO: How to handle array layers?
// TODO: additional enums?
// TODO: Show how to split a combined image buffer in docs
/// A view over Wii U GX2 texture surface to simplify swizzling operations.
///
/// Most of these parameters are likely stored
/// in a texture binary file format in exactly the expected format.
/// If a value is not present in the texture file
/// like [usage](#structfield.usage) or [aa](#structfield.aa),
/// using the recommended default should produce the intended result.
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
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
    /// The number of mipmaps or 1 if there are no additional mipmaps.
    pub mipmap_count: u32,
    /// The format for the image data.
    /// Many texture binary file formats store the GX2 values directly.
    pub format: SurfaceFormat,
    pub aa: AaMode,
    pub usage: u32,
    /// The image data for the base mipmap.
    pub image_data: &'a [u8],
    /// The image data for the mipmaps past the base level starting with mip 1.
    /// If there are no mipmaps, simply set this to an empty slice.
    pub mipmap_data: &'a [u8],
    pub tile_mode: TileMode,
    pub swizzle: u32,
    /// Usually `512 * bytes_per_pixel`.
    pub alignment: u32,
    /// Horizontal pitch that typically depends on [width](#structfield.width).
    pub pitch: u32,
    /// The offsets for each mipmap base the base level starting with mip 1.
    /// Mipmap offsets after mip 1 are relative to the mip 1 offset.
    ///
    /// Many texture binary file formats use this method of encoding offsets.
    pub mipmap_offsets: [u32; 13],
}

// TODO: Also define a swizzle surface?
impl<'a> Gx2Surface<'a> {
    /// Convert all layers and mipmaps from tiled to a combined linear vector.
    pub fn deswizzle(&self) -> Result<Vec<u8>, SwizzleError> {
        let (block_width, block_height) = self.format.block_dim();
        let bytes_per_pixel = self.format.bytes_per_pixel();

        let div_round_up = |x, d| (x + d - 1) / d;

        let mut data = Vec::new();
        for mip in 0..self.mipmap_count {
            let source = if mip == 0 {
                // The mip 0 data is at the start of the image data.
                self.image_data
            } else if mip == 1 {
                // The slice already accounts for the mip 1 offset.
                &self.mipmap_data[..self.mipmap_offsets[1] as usize]
            } else {
                // Remaining mip levels are relative to the start of the mipmap data.
                let offset = self.mipmap_offsets[mip as usize - 1] as usize;
                let next_offset = self.mipmap_offsets[mip as usize] as usize;
                if next_offset != 0 {
                    &self.mipmap_data[offset..next_offset]
                } else {
                    &self.mipmap_data[offset..]
                }
            };

            // TODO: How to handle dimensions not divisible by block dimensions?
            // TODO: cemu uses mipPtr & 0x700 for swizzle for mipmaps?
            let width = div_round_up(self.width >> mip, block_width);
            let height = div_round_up(self.height >> mip, block_height);

            // Some parameters change based on dimensions or mip level.
            // Small mips may use micro instead of macro tiling.
            // TODO: how to set these parameters?
            let input = ComputeSurfaceInfoInput {
                size: source.len() as u32,
                tile_mode: self.tile_mode,
                format: self.format,
                bpp: bytes_per_pixel * u8::BITS,
                num_samples: 1 << self.aa as u32,
                width,
                height,
                num_slices: self.depth,
                slice: 0,
                mip_level: mip,
                flags: Default::default(),
                tile_info: Default::default(),
                tile_type: addrlib::TileType::Displayable,
                tile_index: 0,
            };
            // TODO: Can this use defaults?
            let mut output = ComputeSurfaceInfoOutput {
                size: 0,
                pitch: 0,
                height: 0,
                depth: self.depth,
                surf_size: 0,
                tile_mode: self.tile_mode,
                base_align: 0,
                pitch_align: 0,
                height_align: 0,
                depth_align: 0,
                bpp: 0,
                pixel_pitch: 0,
                pixel_height: 0,
                pixel_bits: 0,
                slice_size: 0,
                pitch_tile_max: 0,
                height_tile_max: 0,
                slice_tile_max: 0,
                tile_info: Default::default(),
                tile_type: addrlib::TileType::Displayable,
                tile_index: 0,
            };
            hwl_compute_surface_info(&input, &mut output);

            // TODO: Why does output.pitch not work?
            let pitch = self.pitch >> mip;

            let mip = deswizzle_mipmap(
                width,
                height,
                self.depth,
                source,
                self.swizzle,
                pitch,
                output.tile_mode,
                bytes_per_pixel,
                self.aa,
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
#[allow(clippy::too_many_arguments)]
pub fn deswizzle_mipmap(
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    source: &[u8],
    swizzle: u32,
    pitch: u32,
    tile_mode: TileMode,
    bytes_per_pixel: u32,
    aa: AaMode,
) -> Result<Vec<u8>, SwizzleError> {
    let output_size = width as usize
        * height as usize
        * depth_or_array_layers as usize
        * bytes_per_pixel as usize;
    if output_size == 0 {
        return Ok(Vec::new());
    }

    let expected_size = swizzled_mipmap_size(
        width,
        height,
        depth_or_array_layers,
        swizzle,
        pitch,
        tile_mode,
        bytes_per_pixel,
        aa,
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
        aa,
    )?;

    Ok(output)
}

/// Convert the linear data in `source` to a combined tiled vector.
///
/// For block compressed formats, `width` and `height` should be the dimensions in blocks
/// with `bytes_per_pixel` being the size of a block in bytes.
#[allow(clippy::too_many_arguments)]
pub fn swizzle_mipmap(
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    source: &[u8],
    swizzle: u32,
    pitch: u32,
    tile_mode: TileMode,
    bytes_per_pixel: u32,
    aa: AaMode,
) -> Result<Vec<u8>, SwizzleError> {
    // TODO: Is this the correct output size?
    let output_size = swizzled_mipmap_size(
        width,
        height,
        depth_or_array_layers,
        swizzle,
        pitch,
        tile_mode,
        bytes_per_pixel,
        aa,
    );
    if output_size == 0 {
        return Ok(Vec::new());
    }

    let expected_size =
        deswizzled_mipmap_size(width, height, depth_or_array_layers, bytes_per_pixel);
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
        aa,
    )?;

    Ok(output)
}

fn deswizzled_mipmap_size(
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    bytes_per_pixel: u32,
) -> usize {
    width as usize * height as usize * depth_or_array_layers as usize * bytes_per_pixel as usize
}

// TODO: Should this use ComputeSurfaceInfo functions from addrlib?
#[allow(clippy::too_many_arguments)]
fn swizzled_mipmap_size(
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    swizzle: u32,
    pitch: u32,
    tile_mode: TileMode,
    bytes_per_pixel: u32,
    aa: AaMode,
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
    let num_samples = 1 << aa as u32;
    let tile_base = 0; // TODO: only used for depth map textures?
    let comp_bits = 0; // TODO: only used for depth map textures?

    // TODO: How many of these fields are set from functions?
    // TODO: Find a way to get values used from cemu to create test cases?
    let p_in = ComputeSurfaceAddrFromCoordInput {
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

#[allow(clippy::too_many_arguments)]
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
    aa: AaMode,
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
    let num_samples = 1 << aa as u32; // TODO: is this based on self.aa?
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
                let p_in = ComputeSurfaceAddrFromCoordInput {
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
    #[test]
    fn deswizzle_empty() {
        assert!(deswizzle_mipmap(
            0,
            0,
            0,
            &[],
            853504,
            256,
            TileMode::D2TiledThin1,
            8,
            AaMode::X1
        )
        .unwrap()
        .is_empty());
    }

    #[test]
    fn deswizzle_macro_tiled_1024x1024_bc1() {
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
                TileMode::D2TiledThin1,
                8,
                AaMode::X1
            )
            .unwrap()[..]
        );
    }

    #[test]
    fn deswizzle_macro_tiled_16x16x16_rgba8() {
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
                TileMode::D2TiledThick,
                4,
                AaMode::X1
            )
            .unwrap()[..]
        );
    }

    #[test]
    fn deswizzle_surface_256x256_bc1_mipmaps() {
        let expected = include_bytes!("data/256x256_bc1_tm4_p64_s132352_mips8_deswizzled.bin");
        let swizzled = include_bytes!("data/256x256_bc1_tm4_p64_s132352_mips8_swizzled.bin");

        let surface = Gx2Surface {
            dim: SurfaceDim::D2,
            width: 256,
            height: 256,
            depth: 1,
            mipmap_count: 8,
            format: SurfaceFormat::BC1Unorm,
            aa: AaMode::X1,
            usage: 1,
            image_data: swizzled,
            mipmap_data: &swizzled[32768..],
            tile_mode: TileMode::D2TiledThin1,
            swizzle: 132352,
            alignment: 4096,
            pitch: 64,
            mipmap_offsets: [
                32768, 9472, 11520, 12032, 12544, 13056, 13568, 0, 0, 0, 0, 0, 0,
            ],
        };
        assert_eq!(expected, &surface.deswizzle().unwrap()[..]);
    }

    // TODO: Test cube maps
}
