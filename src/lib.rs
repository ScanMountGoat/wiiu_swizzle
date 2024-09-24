//! # wiiu_swizzle
//! wiiu_swizzle is a CPU implementation of memory tiling
//! for texture surfaces for the Wii U GPU hardware.
//!
//! Most applications should construct a [Gx2Surface] and use [Gx2Surface::deswizzle]
//! to correctly handle offsets and parameter changes for different mip levels.
#![no_std]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub use addrlib::TileMode;
use addrlib::{
    hwl_compute_surface_info, ComputeSurfaceAddrFromCoordInput, ComputeSurfaceInfoInput,
    ComputeSurfaceInfoOutput, SurfaceFlags,
};
use alloc::{vec, vec::Vec};

mod addrlib;

/// Errors than can occur while converting between tiled and linear memory layouts.
#[derive(Debug, PartialEq, Eq)]
pub enum SwizzleError {
    /// The source data does not contain enough bytes.
    NotEnoughData {
        expected_size: usize,
        actual_size: usize,
    },

    /// The surface dimensions would overflow in size calculations.
    InvalidSurface {
        width: u32,
        height: u32,
        depth: u32,
        format: SurfaceFormat,
        mipmap_count: u32,
    },

    /// The mipmap offsets are out of bounds.
    InvalidMipmapOffsets {
        mipmap_offsets: [u32; 13],
        image_data_len: usize,
        mipmap_data_len: usize,
    },
}

#[cfg(feature = "std")]
impl std::fmt::Display for SwizzleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SwizzleError::NotEnoughData {
                expected_size,
                actual_size,
            } => write!(
                f,
                "Expected at least {expected_size} bytes but found {actual_size} bytes"
            ),
            SwizzleError::InvalidSurface {
                width,
                height,
                depth,
                format,
                mipmap_count,
            } => write!(f, "Invalid surface dimensions {width}x{height}x{depth} format {format:?} and {mipmap_count} mipmaps"),
            SwizzleError::InvalidMipmapOffsets {
                mipmap_offsets,
                image_data_len,
                mipmap_data_len,
            } => write!(f, "Mipmap offsets {mipmap_offsets:?} out of range for {image_data_len} bytes and {mipmap_data_len} mipmap bytes"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SwizzleError {}

// TODO: Use try into and avoid panic.
macro_rules! c_enum {
    (#[$attr1:meta] $name:ident, $($(#[$attr2:meta])* $variant:ident=$value:expr),*,) => {
        #[$attr1]
        #[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub enum $name {
            $(
                $(#[$attr2])*
                $variant = $value
            ),*
        }

        impl $name {
            /// Returns the variant with the given value or `None` if invalid.
            pub fn from_repr(value: u32) -> Option<Self> {
                match value {
                    $(
                        $value => Some(Self::$variant),
                    )*
                    _ => None
                }
            }
        }
    };
}
pub(crate) use c_enum;

// TODO: Include all gx2 enum variants?
c_enum! {
    /// GX2AAMode for the number of samples
    AaMode,
    X1 = 0,
    X2 = 1,
    X4 = 2,
    X8 = 3,
}

// The GX2 and addrlib enums are the same.
// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/include/addrlib/addrtypes.h#L118
c_enum! {
    /// GX2SurfaceFormat for the format of the image data
    SurfaceFormat,
    /// GX2_SURFACE_FORMAT_TC_R8_UNORM
    R8Unorm = 0x00000001,
    /// GX2_SURFACE_FORMAT_TC_R8_UINT
    R8Uint = 0x00000101,
    /// GX2_SURFACE_FORMAT_TC_R8_SNORM
    R8Snorm = 0x00000201,
    /// GX2_SURFACE_FORMAT_TC_R8_SINT
    R8Sint = 0x00000301,
    /// GX2_SURFACE_FORMAT_T_R4_G4_UNORM
    R4G4Unorm = 0x00000002,
    /// GX2_SURFACE_FORMAT_TCD_R16_UNORM
    R16Unorm = 0x00000005,
    /// GX2_SURFACE_FORMAT_TC_R16_UINT
    R16Uint = 0x00000105,
    /// GX2_SURFACE_FORMAT_TC_R16_SNORM
    R16Snorm = 0x00000205,
    /// GX2_SURFACE_FORMAT_TC_R16_SINT
    R16Sint = 0x00000305,
    /// GX2_SURFACE_FORMAT_TC_R16_FLOAT
    R16Float = 0x00000806,
    /// GX2_SURFACE_FORMAT_TC_R8_G8_UNORM
    R8G8Unorm = 0x00000007,
    /// GX2_SURFACE_FORMAT_TC_R8_G8_UINT
    R8G8Uint = 0x00000107,
    /// GX2_SURFACE_FORMAT_TC_R8_G8_SNORM
    R8G8Snorm = 0x00000207,
    /// GX2_SURFACE_FORMAT_TC_R8_G8_SINT
    R8G8Sint = 0x00000307,
    /// GX2_SURFACE_FORMAT_TCS_R5_G6_B5_UNORM
    R5G6B5Unorm = 0x00000008,
    /// GX2_SURFACE_FORMAT_TC_R5_G5_B5_A1_UNORM
    R5G5B5A1Unorm = 0x0000000a,
    /// GX2_SURFACE_FORMAT_TC_R4_G4_B4_A4_UNORM
    R4G4B4A4Unorm = 0x0000000b,
    /// GX2_SURFACE_FORMAT_TC_R32_UINT
    R32Uint = 0x0000010d,
    /// GX2_SURFACE_FORMAT_TC_R32_SINT
    R32Sint = 0x0000030d,
    /// GX2_SURFACE_FORMAT_TCD_R32_FLOAT
    R32Float = 0x0000080e,
    /// GX2_SURFACE_FORMAT_TC_R16_G16_UNORM
    R16G16Unorm = 0x0000000f,
    /// GX2_SURFACE_FORMAT_TC_R16_G16_UINT
    R16G16Uint = 0x0000010f,
    /// GX2_SURFACE_FORMAT_TC_R16_G16_SNORM
    R16G16Snorm = 0x0000020f,
    /// GX2_SURFACE_FORMAT_TC_R16_G16_SINT
    R16G16Sint = 0x0000030f,
    /// GX2_SURFACE_FORMAT_TC_R16_G16_FLOAT
    R16G16Float = 0x00000810,
    /// GX2_SURFACE_FORMAT_D_D24_S8_FLOAT
    D24S8Float = 0x00000811,
    /// GX2_SURFACE_FORMAT_TC_R11_G11_B10_FLOAT
    R11G11B10Float = 0x00000816,
    /// GX2_SURFACE_FORMAT_TCS_R10_G10_B10_A2_UNORM
    R10G10B10A2Unorm = 0x00000019,
    /// GX2_SURFACE_FORMAT_TC_R10_G10_B10_A2_UINT
    R10G10B10A2Uint = 0x00000119,
    /// GX2_SURFACE_FORMAT_TC_R10_G10_B10_A2_SINT
    R10G10B10A2Sint = 0x00000319,
    /// GX2_SURFACE_FORMAT_TCS_R8_G8_B8_A8_UNORM
    R8G8B8A8Unorm = 0x0000001a,
    /// GX2_SURFACE_FORMAT_TC_R8_G8_B8_A8_UINT
    R8G8B8A8Uint = 0x0000011a,
    /// GX2_SURFACE_FORMAT_TC_R8_G8_B8_A8_SNORM
    R8G8B8A8Snorm = 0x0000021a,
    /// GX2_SURFACE_FORMAT_TC_R8_G8_B8_A8_SINT
    R8G8B8A8Sint = 0x0000031a,
    /// GX2_SURFACE_FORMAT_TCS_R8_G8_B8_A8_SRGB
    R8G8B8A8Srgb = 0x0000041a,
    /// GX2_SURFACE_FORMAT_TC_R32_G32_UINT
    R32G32Uint = 0x0000011d,
    /// GX2_SURFACE_FORMAT_TC_R32_G32_SINT
    R32G32Sint = 0x0000031d,
    /// GX2_SURFACE_FORMAT_TC_R32_G32_FLOAT
    R32G32Float = 0x0000081e,
    /// GX2_SURFACE_FORMAT_TC_R16_G16_B16_A16_UNORM
    R16G16B16A16Unorm = 0x0000001f,
    /// GX2_SURFACE_FORMAT_TC_R16_G16_B16_A16_UINT
    R16G16B16A16Uint = 0x0000011f,
    /// GX2_SURFACE_FORMAT_TC_R16_G16_B16_A16_SNORM
    R16G16B16A16Snorm = 0x0000021f,
    /// GX2_SURFACE_FORMAT_TC_R16_G16_B16_A16_SINT
    R16G16B16A16Sint = 0x0000031f,
    /// GX2_SURFACE_FORMAT_TC_R16_G16_B16_A16_FLOAT
    R16G16B16A16Float = 0x00000820,
    /// GX2_SURFACE_FORMAT_TC_R32_G32_B32_A32_UINT
    R32G32B32A32Uint = 0x00000122,
    /// GX2_SURFACE_FORMAT_TC_R32_G32_B32_A32_SINT
    R32G32B32A32Sint = 0x00000322,
    /// GX2_SURFACE_FORMAT_TC_R32_G32_B32_A32_FLOAT
    R32G32B32A32Float = 0x00000823,
    /// GX2_SURFACE_FORMAT_T_BC1_UNORM
    Bc1Unorm = 0x00000031,
    /// GX2_SURFACE_FORMAT_T_BC1_SRGB
    Bc1Srgb = 0x00000431,
    /// GX2_SURFACE_FORMAT_T_BC2_UNORM
    Bc2Unorm = 0x00000032,
    /// GX2_SURFACE_FORMAT_T_BC2_SRGB
    Bc2Srgb = 0x00000432,
    /// GX2_SURFACE_FORMAT_T_BC3_UNORM
    Bc3Unorm = 0x00000033,
    /// GX2_SURFACE_FORMAT_T_BC3_SRGB
    Bc3Srgb = 0x00000433,
    /// GX2_SURFACE_FORMAT_T_BC4_UNORM
    Bc4Unorm = 0x00000034,
    /// GX2_SURFACE_FORMAT_T_BC4_SNORM
    Bc4Snorm = 0x00000234,
    /// GX2_SURFACE_FORMAT_T_BC5_UNORM
    Bc5Unorm = 0x00000035,
    /// GX2_SURFACE_FORMAT_T_BC5_SNORM
    Bc5Snorm = 0x00000235,
}

impl SurfaceFormat {
    pub fn block_dim(&self) -> (u32, u32) {
        match self {
            SurfaceFormat::Bc1Unorm => (4, 4),
            SurfaceFormat::Bc1Srgb => (4, 4),
            SurfaceFormat::Bc2Unorm => (4, 4),
            SurfaceFormat::Bc2Srgb => (4, 4),
            SurfaceFormat::Bc3Unorm => (4, 4),
            SurfaceFormat::Bc3Srgb => (4, 4),
            SurfaceFormat::Bc4Unorm => (4, 4),
            SurfaceFormat::Bc4Snorm => (4, 4),
            SurfaceFormat::Bc5Unorm => (4, 4),
            SurfaceFormat::Bc5Snorm => (4, 4),
            _ => (1, 1),
        }
    }

    // https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/core/addrelemlib.cpp#L139
    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            SurfaceFormat::R8Unorm => 1,
            SurfaceFormat::R8Uint => 1,
            SurfaceFormat::R8Snorm => 1,
            SurfaceFormat::R8Sint => 1,
            SurfaceFormat::R4G4Unorm => 1,
            SurfaceFormat::R16Unorm => 2,
            SurfaceFormat::R16Uint => 2,
            SurfaceFormat::R16Snorm => 2,
            SurfaceFormat::R16Sint => 2,
            SurfaceFormat::R16Float => 2,
            SurfaceFormat::R8G8Unorm => 2,
            SurfaceFormat::R8G8Uint => 2,
            SurfaceFormat::R8G8Snorm => 2,
            SurfaceFormat::R8G8Sint => 2,
            SurfaceFormat::R5G6B5Unorm => 2,
            SurfaceFormat::R5G5B5A1Unorm => 2,
            SurfaceFormat::R4G4B4A4Unorm => 2,
            SurfaceFormat::R32Uint => 4,
            SurfaceFormat::R32Sint => 4,
            SurfaceFormat::R32Float => 4,
            SurfaceFormat::R16G16Unorm => 4,
            SurfaceFormat::R16G16Uint => 4,
            SurfaceFormat::R16G16Snorm => 4,
            SurfaceFormat::R16G16Sint => 4,
            SurfaceFormat::R16G16Float => 4,
            SurfaceFormat::D24S8Float => 4,
            SurfaceFormat::R11G11B10Float => 4,
            SurfaceFormat::R10G10B10A2Unorm => 4,
            SurfaceFormat::R10G10B10A2Uint => 4,
            SurfaceFormat::R10G10B10A2Sint => 4,
            SurfaceFormat::R8G8B8A8Unorm => 4,
            SurfaceFormat::R8G8B8A8Uint => 4,
            SurfaceFormat::R8G8B8A8Snorm => 4,
            SurfaceFormat::R8G8B8A8Sint => 4,
            SurfaceFormat::R8G8B8A8Srgb => 4,
            SurfaceFormat::R32G32Uint => 8,
            SurfaceFormat::R32G32Sint => 8,
            SurfaceFormat::R32G32Float => 8,
            SurfaceFormat::R16G16B16A16Unorm => 8,
            SurfaceFormat::R16G16B16A16Uint => 8,
            SurfaceFormat::R16G16B16A16Snorm => 8,
            SurfaceFormat::R16G16B16A16Sint => 8,
            SurfaceFormat::R16G16B16A16Float => 8,
            SurfaceFormat::R32G32B32A32Uint => 16,
            SurfaceFormat::R32G32B32A32Sint => 16,
            SurfaceFormat::R32G32B32A32Float => 16,
            SurfaceFormat::Bc1Unorm => 8,
            SurfaceFormat::Bc1Srgb => 8,
            SurfaceFormat::Bc2Unorm => 16,
            SurfaceFormat::Bc2Srgb => 16,
            SurfaceFormat::Bc3Unorm => 16,
            SurfaceFormat::Bc3Srgb => 16,
            SurfaceFormat::Bc4Unorm => 8,
            SurfaceFormat::Bc4Snorm => 8,
            SurfaceFormat::Bc5Unorm => 16,
            SurfaceFormat::Bc5Snorm => 16,
        }
    }
}

c_enum! {
    /// GX2SurfaceDim for the dimensionality of the texture surface
    SurfaceDim,
    D1 = 0,
    D2 = 1,
    D3 = 2,
    Cube = 3,
}

// TODO: impl Default?
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
    /// The depth of the base mip level in pixels or the number of array layers.
    /// Cube maps will have a value of 6.
    /// 2D surfaces without any layers should use a value of 1.
    pub depth_or_array_layers: u32,
    /// The number of mipmaps or 1 if there are no additional mipmaps.
    pub mipmap_count: u32,
    /// The format for the image data.
    /// Many texture binary file formats store the GX2 values directly.
    pub format: SurfaceFormat,
    /// Used to calculate the sample count. Often set to [AaMode::X1].
    pub aa: AaMode,
    pub usage: u32,
    /// The image data for the base mipmap.
    pub image_data: &'a [u8],
    /// The image data for the mipmaps past the base level starting with mip 1.
    /// If there are no mipmaps, simply set this to an empty slice.
    pub mipmap_data: &'a [u8],
    pub tile_mode: TileMode,
    /// Affects the upper bits of the swizzled address. Often set to 0.
    pub swizzle: u32,
    /// Usually `512 * bytes_per_pixel`.
    pub alignment: u32,
    /// Horizontal pitch that typically depends on [width](#structfield.width).
    pub pitch: u32,
    /// The offsets for each mipmap past the base level starting with mip 1.
    /// Mipmap offsets after mip 1 are relative to the mip 1 offset.
    ///
    /// Many texture binary file formats use this method of encoding offsets.
    pub mipmap_offsets: [u32; 13],
}

// TODO: Also define a swizzle surface?
impl<'a> Gx2Surface<'a> {
    /// Convert all layers and mipmaps from tiled to a combined linear vector.
    pub fn deswizzle(&self) -> Result<Vec<u8>, SwizzleError> {
        // TODO: The compute info functions can also validate?
        self.validate()?;
        // TODO: how to handle empty surfaces?
        if self.width == 0 || self.height == 0 || self.depth_or_array_layers == 0 || self.pitch == 0
        {
            return Ok(Vec::new());
        }

        let (block_width, block_height) = self.format.block_dim();
        let bytes_per_pixel = self.format.bytes_per_pixel();

        let mut data = Vec::new();
        for mip in 0..self.mipmap_count {
            let source = if mip == 0 {
                // The mip 0 data is at the start of the image data.
                self.image_data
            } else if mip == 1 {
                // The slice already accounts for the mip 1 offset.
                let next_offset = self.mipmap_offsets[mip as usize] as usize;
                if next_offset != 0 {
                    &self.mipmap_data[..next_offset]
                } else {
                    self.mipmap_data
                }
            } else {
                // Remaining mip levels are relative to the start of the mipmap data.
                let offset = if mip == 1 {
                    0
                } else {
                    self.mipmap_offsets[mip as usize - 1] as usize
                };
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
                num_slices: self.depth_or_array_layers,
                slice: 0,
                mip_level: mip,
                flags: match self.dim {
                    SurfaceDim::D1 => SurfaceFlags::default(),
                    SurfaceDim::D2 => SurfaceFlags::default(),
                    SurfaceDim::D3 => SurfaceFlags::volume,
                    SurfaceDim::Cube => SurfaceFlags::cube,
                },
                tile_info: Default::default(),
                tile_type: addrlib::TileType::Displayable,
                tile_index: 0,
            };
            // TODO: Can this use defaults?
            let mut output = ComputeSurfaceInfoOutput {
                size: 0,
                pitch: 0,
                height: 0,
                depth: 0,
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

            // TODO: Why is output.pitch sometimes too large?
            let pitch = output.pitch.min(self.pitch);

            // TODO: is this data all layers for each mip?
            // TODO: Store this as layer major instead of mip major to match dds?
            // TODO: Should this be depth and also layers?
            let mip = deswizzle_mipmap(
                width,
                height,
                self.depth_or_array_layers,
                source,
                self.swizzle,
                pitch,
                output.tile_mode,
                bytes_per_pixel,
                self.aa,
            )?;
            data.extend_from_slice(&mip);
        }

        if self.dim == SurfaceDim::Cube {
            Ok(self.mip_major_to_layer_major(&data, block_width, block_height, bytes_per_pixel))
        } else {
            Ok(data)
        }
    }

    fn mip_major_to_layer_major(
        &self,
        data: &[u8],
        block_width: u32,
        block_height: u32,
        bytes_per_pixel: u32,
    ) -> Vec<u8> {
        // Convert from [mip][layer] to [layer][mip] ordering.
        // TODO: Is there a better way of doing this?
        let mut new_data = vec![0u8; data.len()];

        let mut mip_offsets: Vec<_> = (0..self.mipmap_count.saturating_sub(1))
            .map(|mip| {
                let width = div_round_up(self.width >> mip, block_width);
                let height = div_round_up(self.height >> mip, block_height);
                width * height * bytes_per_pixel
            })
            .scan(0, |state, x| Some(*state + x))
            .collect();
        mip_offsets.insert(0, 0);

        let output_layer_size = (0..self.mipmap_count)
            .map(|mip| {
                let width = div_round_up(self.width >> mip, block_width);
                let height = div_round_up(self.height >> mip, block_height);
                width * height * bytes_per_pixel
            })
            .sum::<u32>();

        let mut mip_offset = 0;
        for mip in 0..self.mipmap_count {
            let width = div_round_up(self.width >> mip, block_width);
            let height = div_round_up(self.height >> mip, block_height);
            let mip_size = (width * height * bytes_per_pixel) as usize;

            for layer in 0..6 {
                let layer_offset = width * height * layer * bytes_per_pixel;
                let input_offset = layer_offset as usize + mip_offset;
                let mip_data = &data[input_offset..input_offset + mip_size];

                let output_layer_offset = output_layer_size * layer;
                let output_mip_offset = mip_offsets[mip as usize];
                let output_offset = (output_layer_offset + output_mip_offset) as usize;

                new_data[output_offset..output_offset + mip_size].copy_from_slice(mip_data);
            }

            mip_offset += mip_size * 6;
        }
        new_data
    }

    fn validate(&self) -> Result<(), SwizzleError> {
        if self.mipmap_offsets[0] > self.image_data.len() as u32
            || self.mipmap_offsets[1..]
                .iter()
                .any(|o| *o > self.mipmap_data.len() as u32)
        {
            Err(SwizzleError::InvalidMipmapOffsets {
                mipmap_offsets: self.mipmap_offsets,
                image_data_len: self.image_data.len(),
                mipmap_data_len: self.mipmap_data.len(),
            })
        } else if self
            .width
            .checked_mul(self.height)
            .and_then(|u| u.checked_mul(self.depth_or_array_layers))
            .and_then(|u| u.checked_mul(self.format.bytes_per_pixel() * u8::BITS))
            .and_then(|u| u.checked_mul(self.pitch))
            .and_then(|u| u.checked_mul(1 << self.aa as u32))
            .is_none()
            || self.mipmap_count > self.mipmap_offsets.len() as u32
        {
            // Check dimensions to prevent overflow.
            Err(SwizzleError::InvalidSurface {
                width: self.width,
                height: self.height,
                depth: self.depth_or_array_layers,
                format: self.format,
                mipmap_count: self.mipmap_count,
            })
        } else {
            Ok(())
        }
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
    // TODO: How to handle zero sized inputs?
    if output_size == 0 {
        return Ok(Vec::new());
    }

    // TODO: Why is this sometimes 0?
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

fn div_round_up(x: u32, d: u32) -> u32 {
    (x + d - 1) / d
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
            depth_or_array_layers: 1,
            mipmap_count: 8,
            format: SurfaceFormat::Bc1Unorm,
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

    #[test]
    fn deswizzle_surface_64x64_cube_bc1_mipmaps() {
        let expected = include_bytes!("data/64x64_cube_bc1_tm4_p32_s67328_deswizzled.bin");
        let swizzled = include_bytes!("data/64x64_cube_bc1_tm4_p32_s67328_swizzled.bin");

        let surface = Gx2Surface {
            dim: SurfaceDim::Cube,
            width: 64,
            height: 64,
            depth_or_array_layers: 6,
            mipmap_count: 2,
            format: SurfaceFormat::Bc1Unorm,
            aa: AaMode::X1,
            usage: 1,
            image_data: swizzled,
            mipmap_data: &swizzled[24576..],
            tile_mode: TileMode::D2TiledThin1,
            swizzle: 67328,
            alignment: 4096,
            pitch: 32,
            mipmap_offsets: [24576, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        };
        assert_eq!(expected, &surface.deswizzle().unwrap()[..]);
    }

    #[test]
    fn deswizzle_surface_overflow() {
        let surface = Gx2Surface {
            dim: SurfaceDim::Cube,
            width: 65535,
            height: 65535,
            depth_or_array_layers: 65535,
            mipmap_count: 1,
            format: SurfaceFormat::Bc1Unorm,
            aa: AaMode::X1,
            usage: 1,
            image_data: &[],
            mipmap_data: &[],
            tile_mode: TileMode::D2TiledThin1,
            swizzle: 67328,
            alignment: 4096,
            pitch: 32,
            mipmap_offsets: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        };
        assert_eq!(
            Err(SwizzleError::InvalidSurface {
                width: 65535,
                height: 65535,
                depth: 65535,
                format: SurfaceFormat::Bc1Unorm,
                mipmap_count: 1
            }),
            surface.deswizzle()
        );
    }

    #[test]
    fn aa_mode_from_repr() {
        assert_eq!(Some(AaMode::X2), AaMode::from_repr(1));
        assert_eq!(None, AaMode::from_repr(0xff));
    }

    #[test]
    fn surface_dim_from_repr() {
        assert_eq!(Some(SurfaceDim::D2), SurfaceDim::from_repr(1));
        assert_eq!(None, SurfaceDim::from_repr(0xff));
    }

    #[test]
    fn surface_format_from_repr() {
        assert_eq!(Some(SurfaceFormat::Bc5Unorm), SurfaceFormat::from_repr(53));
        assert_eq!(None, SurfaceFormat::from_repr(0xff));
    }

    #[test]
    fn tile_mode_from_repr() {
        assert_eq!(Some(TileMode::D2TiledThin1), TileMode::from_repr(4));
        assert_eq!(None, TileMode::from_repr(0xff));
    }
}
