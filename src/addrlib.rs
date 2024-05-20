use crate::SurfaceFormat;
use bilge::prelude::*;

// These are class member variables in addrlib.
// Values taken from Cemu.
// https://github.com/cemu-project/Cemu/blob/85141f17f977157b91b72883d879f50b27f17dda/src/Cafe/HW/Latte/LatteAddrLib/LatteAddrLib.h#L6-L15
// License: https://github.com/cemu-project/Cemu/blob/main/LICENSE.txt
const M_BANKS: u32 = 4;
const M_PIPES: u32 = 2;
const M_PIPE_INTERLEAVE_BYTES: u32 = 256;
const M_SPLIT_SIZE: u32 = 2048;
const M_ROW_SIZE: u32 = 2048;
const M_SWAP_SIZE: u32 = 256;

// Ported from c++:
// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/core/addrcommon.h
const MICRO_TILE_WIDTH: u32 = 8;
const MICRO_TILE_HEIGHT: u32 = 8;
const THICK_TILE_THICKNESS: u32 = 4;
const MICRO_TILE_PIXELS: u32 = MICRO_TILE_WIDTH * MICRO_TILE_HEIGHT;

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/include/addrlib/addrtypes.h#L83
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum TileMode {
    /// ADDR_TM_LINEAR_GENERAL
    LinearGeneral = 0x0,
    /// ADDR_TM_LINEAR_ALIGNED
    LinearAligned = 0x1,
    /// ADDR_TM_1D_TILED_THIN1
    D1TiledThin1 = 0x2,
    /// ADDR_TM_1D_TILED_THICK
    D1TiledThick = 0x3,
    /// ADDR_TM_2D_TILED_THIN1
    D2TiledThin1 = 0x4,
    /// ADDR_TM_2D_TILED_THIN2
    D2TiledThin2 = 0x5,
    /// ADDR_TM_2D_TILED_THIN4
    D2TiledThin4 = 0x6,
    /// ADDR_TM_2D_TILED_THICK
    D2TiledThick = 0x7,
    /// ADDR_TM_2B_TILED_THIN1
    B2TiledThin1 = 0x8,
    /// ADDR_TM_2B_TILED_THIN2
    B2TiledThin2 = 0x9,
    /// ADDR_TM_2B_TILED_THIN4
    B2TiledThin4 = 0xA,
    /// ADDR_TM_2B_TILED_THICK
    B2TiledThick = 0xB,
    /// ADDR_TM_3D_TILED_THIN1
    D3TiledThin1 = 0xC,
    /// ADDR_TM_3D_TILED_THICK
    D3TiledThick = 0xD,
    /// ADDR_TM_3B_TILED_THIN1
    B3TiledThin1 = 0xE,
    /// ADDR_TM_3B_TILED_THICK
    B3TiledThick = 0xF,
    /// ADDR_TM_2D_TILED_XTHICK or ADDR_TM_LINEAR_SPECIAL
    D2TiledXthick = 0x10,
    /// ADDR_TM_3D_TILED_XTHICK
    D3TiledXThick = 0x11,
    /// ADDR_TM_POWER_SAVE
    PowerSave = 0x12,
    /// ADDR_TM_COUNT
    Count = 0x13,
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/include/addrlib/addrtypes.h#L230C1-L237C1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    /// ADDR_DISPLAYABLE
    Displayable = 0x0,
    /// ADDR_NON_DISPLAYABLE
    NonDisplayable = 0x1,
    /// ADDR_DEPTH_SAMPLE_ORDER
    DepthSampleOrder = 0x2,
    /// ADDR_THICK_TILING
    ThickTiling = 0x3,
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/include/addrlib/addrinterface.h#L266
#[bitsize(32)]
#[derive(DebugBits, Clone, Copy, PartialEq, Eq, DefaultBits)]
pub struct SurfaceFlags {
    // TODO: bitfield?
    color: bool,
    depth: bool,
    stencil: bool,
    texture: bool,
    cube: bool,
    volume: bool,
    fmask: bool,
    cube_as_array: bool,
    compress_z: bool,
    linear_wa: bool,
    overlay: bool,
    no_stencil: bool,
    input_base_map: bool,
    display: bool,
    opt4_space: bool,
    prt: bool,
    qb_stereo: bool,
    pow2_pad: bool,
    reserved: bool,
    unused: u13,
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/include/addrlib/addrinterface.h#L311
#[derive(Default)]
pub struct TileInfo {
    banks: u32,
    bank_width: u32,
    bank_height: u32,
    macro_aspect_ratio: u32,
    tile_split_bytes: u32,
    // AddrPipeCfg pipeConfig;
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/include/addrlib/addrinterface.h#L343
pub struct ComputeSurfaceInfoInput {
    pub size: u32,
    pub tile_mode: TileMode,
    pub format: SurfaceFormat,
    pub bpp: u32,
    pub num_samples: u32,
    pub width: u32,
    pub height: u32,
    pub num_slices: u32,
    pub slice: u32,
    pub mip_level: u32,
    pub flags: SurfaceFlags,
    pub tile_info: TileInfo,
    pub tile_type: TileType,
    pub tile_index: i32,
}

// Modified to remove unused fields.
// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/include/addrlib/addrinterface.h#L374
pub struct ComputeSurfaceInfoOutput {
    pub size: u32,
    pub pitch: u32,
    pub height: u32,
    pub depth: u32,
    pub surf_size: u64,
    pub tile_mode: TileMode,
    pub base_align: u32,
    pub pitch_align: u32,
    pub height_align: u32,
    pub depth_align: u32,
    pub bpp: u32,
    pub pixel_pitch: u32,
    pub pixel_height: u32,
    pub pixel_bits: u32,
    pub slice_size: u32,
    pub pitch_tile_max: u32,
    pub height_tile_max: u32,
    pub slice_tile_max: u32,
    pub tile_info: TileInfo,
    pub tile_type: TileType,
    pub tile_index: i32, // ADDR_QBSTEREOINFO *pStereoInfo;
}

// Modified to remove unused fields.
// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/include/addrlib/addrinterface.h#L409
#[derive(Debug)]
pub struct ComputeSurfaceAddrFromCoordInput {
    pub x: u32,
    pub y: u32,
    pub slice: u32,
    pub sample: u32,
    pub bpp: u32,
    pub pitch: u32,
    pub height: u32,
    pub num_slices: u32,
    pub num_samples: u32,
    pub tile_mode: TileMode,
    pub is_depth: bool,
    pub tile_base: u32,
    pub comp_bits: u32,
    pub pipe_swizzle: u32,
    pub bank_swizzle: u32,
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/core/addrcommon.h#L50
fn bits_to_bytes(x: u32) -> u32 {
    // div_round_up
    (x + u8::BITS - 1) / u8::BITS
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/core/addrcommon.h#L52
fn bit(v: u32, b: u32) -> u32 {
    (v >> b) & 1
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/core/addrlib.cpp#L306
fn compute_surface_thickness(tile_mode: TileMode) -> u32 {
    match tile_mode {
        TileMode::D1TiledThick
        | TileMode::D2TiledThick
        | TileMode::B2TiledThick
        | TileMode::D3TiledThick
        | TileMode::B3TiledThick => 4,
        TileMode::D2TiledXthick | TileMode::D3TiledXThick => 8,
        _ => 1,
    }
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/core/addrlib.cpp#L337
fn adjust_pitch_alignment(flags: SurfaceFlags, pitch_align: &mut u32) {
    if flags.display() {
        *pitch_align = pitch_align.next_multiple_of(32);
    }
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/core/addrlib.cpp#L433
fn pad_dimensions(
    tile_mode: TileMode,
    flags: SurfaceFlags,
    pad_dims: u32,
    pitch: &mut u32,
    pitch_align: u32,
    height: &mut u32,
    height_align: u32,
    slices: &mut u32,
    slice_align: u32,
) {
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/core/addrlib.cpp#L643
fn compute_pixel_index_within_micro_tile(
    x: u32,
    y: u32,
    z: u32,
    bpp: u32,
    tile_mode: TileMode,
    tile_type: TileType,
) -> u32 {
    let pixel_bit0;
    let pixel_bit1;
    let pixel_bit2;
    let pixel_bit3;
    let pixel_bit4;
    let pixel_bit5;
    let mut pixel_bit6 = 0;
    let mut pixel_bit7 = 0;
    let mut pixel_bit8 = 0;

    let x0 = bit(x, 0);
    let x1 = bit(x, 1);
    let x2 = bit(x, 2);
    let y0 = bit(y, 0);
    let y1 = bit(y, 1);
    let y2 = bit(y, 2);
    let z0 = bit(z, 0);
    let z1 = bit(z, 1);
    let z2 = bit(z, 2);

    let thickness = compute_surface_thickness(tile_mode);

    if tile_type == TileType::ThickTiling {
        pixel_bit0 = x0;
        pixel_bit1 = y0;
        pixel_bit2 = z0;
        pixel_bit3 = x1;
        pixel_bit4 = y1;
        pixel_bit5 = z1;
        pixel_bit6 = x2;
        pixel_bit7 = y2;
    } else {
        if tile_type == TileType::NonDisplayable {
            pixel_bit0 = x0;
            pixel_bit1 = y0;
            pixel_bit2 = x1;
            pixel_bit3 = y1;
            pixel_bit4 = x2;
            pixel_bit5 = y2;
        } else {
            match bpp {
                8 => {
                    pixel_bit0 = x0;
                    pixel_bit1 = x1;
                    pixel_bit2 = x2;
                    pixel_bit3 = y1;
                    pixel_bit4 = y0;
                    pixel_bit5 = y2;
                }
                16 => {
                    pixel_bit0 = x0;
                    pixel_bit1 = x1;
                    pixel_bit2 = x2;
                    pixel_bit3 = y0;
                    pixel_bit4 = y1;
                    pixel_bit5 = y2;
                }
                64 => {
                    pixel_bit0 = x0;
                    pixel_bit1 = y0;
                    pixel_bit2 = x1;
                    pixel_bit3 = x2;
                    pixel_bit4 = y1;
                    pixel_bit5 = y2;
                }
                128 => {
                    pixel_bit0 = y0;
                    pixel_bit1 = x0;
                    pixel_bit2 = x1;
                    pixel_bit3 = x2;
                    pixel_bit4 = y1;
                    pixel_bit5 = y2;
                }
                _ => {
                    pixel_bit0 = x0;
                    pixel_bit1 = x1;
                    pixel_bit2 = y0;
                    pixel_bit3 = x2;
                    pixel_bit4 = y1;
                    pixel_bit5 = y2;
                }
            }
        }

        if thickness > 1 {
            pixel_bit6 = z0;
            pixel_bit7 = z1;
        }
    }

    if thickness == 8 {
        pixel_bit8 = z2;
    }

    pixel_bit0
        | (pixel_bit1 << 1)
        | (pixel_bit2 << 2)
        | (pixel_bit3 << 3)
        | (pixel_bit4 << 4)
        | (pixel_bit5 << 5)
        | (pixel_bit6 << 6)
        | (pixel_bit7 << 7)
        | (pixel_bit8 << 8)
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L355
fn convert_to_non_bank_swapped_mode(tile_mode: TileMode) -> TileMode {
    match tile_mode {
        TileMode::B2TiledThin1 => TileMode::D2TiledThin1,
        TileMode::B2TiledThin2 => TileMode::D2TiledThin2,
        TileMode::B2TiledThin4 => TileMode::D2TiledThin4,
        TileMode::B2TiledThick => TileMode::D2TiledThick,
        TileMode::B3TiledThin1 => TileMode::D3TiledThin1,
        TileMode::B3TiledThick => TileMode::D3TiledThick,
        _ => tile_mode,
    }
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L387
fn compute_surface_tile_slices(tile_mode: TileMode, bpp: u32, num_samples: u32) -> u32 {
    let mut num_samples = num_samples;
    let bytes_per_sample = bits_to_bytes(bpp * 64);
    let mut tile_slices = 1;

    if compute_surface_thickness(tile_mode) > 1 {
        num_samples = 4;
    }

    if bytes_per_sample != 0 {
        let sample_per_tile = M_SPLIT_SIZE / bytes_per_sample;

        if sample_per_tile != 0 {
            tile_slices = (num_samples / sample_per_tile).max(1);
        }
    }

    tile_slices
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L421
fn compute_surface_rotation_from_tile_mode(tile_mode: TileMode) -> u32 {
    match tile_mode {
        TileMode::D2TiledThin1
        | TileMode::D2TiledThin2
        | TileMode::D2TiledThin4
        | TileMode::D2TiledThick
        | TileMode::B2TiledThin1
        | TileMode::B2TiledThin2
        | TileMode::B2TiledThin4
        | TileMode::B2TiledThick => M_PIPES * ((M_BANKS >> 1) - 1),
        TileMode::D3TiledThin1
        | TileMode::D3TiledThick
        | TileMode::B3TiledThin1
        | TileMode::B3TiledThick => {
            if M_PIPES >= 4 {
                (M_PIPES >> 1) - 1
            } else {
                1
            }
        }
        _ => 0,
    }
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L466
fn hwl_degrade_thick_tile_mode(
    tile_mode: TileMode,
    num_samples: u32,
    tile_slices: u32,
    is_depth: bool,
) -> TileMode {
    let mut tile_mode = tile_mode;
    match tile_mode {
        TileMode::D1TiledThin1 => {
            // TODO: Is this case used?
            // if (num_samples > 1 && mConfigFlags.no1DTiledMSAA) {
            //     tile_mode = TileMode::ADDR_TM_2D_TILED_THIN1;
            // }
        }
        TileMode::D1TiledThick => {
            if num_samples > 1 || is_depth {
                tile_mode = TileMode::D1TiledThin1;
            }

            if num_samples == 2 || num_samples == 4 {
                tile_mode = TileMode::D2TiledThick;
            }
        }
        TileMode::D2TiledThin2 => {
            if 2 * M_PIPE_INTERLEAVE_BYTES > M_SPLIT_SIZE {
                tile_mode = TileMode::D2TiledThin1;
            }
        }
        TileMode::D2TiledThin4 => {
            if 4 * M_PIPE_INTERLEAVE_BYTES > M_SPLIT_SIZE {
                tile_mode = TileMode::D2TiledThin2;
            }
        }
        TileMode::D2TiledThick => {
            if num_samples > 1 || tile_slices > 1 || is_depth {
                tile_mode = TileMode::D2TiledThin1;
            }
        }
        TileMode::B2TiledThin2 => {
            if 2 * M_PIPE_INTERLEAVE_BYTES > M_SPLIT_SIZE {
                tile_mode = TileMode::B2TiledThin1;
            }
        }
        TileMode::B2TiledThin4 => {
            if 4 * M_PIPE_INTERLEAVE_BYTES > M_SPLIT_SIZE {
                tile_mode = TileMode::B2TiledThin2;
            }
        }
        TileMode::B2TiledThick => {
            if num_samples > 1 || tile_slices > 1 || is_depth {
                tile_mode = TileMode::B2TiledThin1;
            }
        }
        TileMode::D3TiledThick => {
            if num_samples > 1 || tile_slices > 1 || is_depth {
                tile_mode = TileMode::D3TiledThin1;
            }
        }
        TileMode::B3TiledThick => {
            if num_samples > 1 || tile_slices > 1 || is_depth {
                tile_mode = TileMode::B3TiledThin1;
            }
        }
        _ => (),
    }

    tile_mode
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L544
#[allow(clippy::too_many_arguments)]
pub fn compute_surface_mip_level_tile_mode(
    base_tile_mode: TileMode,
    bpp: u32,
    level: u32,
    width: u32,
    height: u32,
    num_slices: u32,
    num_samples: u32,
    is_depth: bool,
    no_recursive: bool,
) -> TileMode {
    let tile_slices = compute_surface_tile_slices(base_tile_mode, bpp, num_samples);
    let mut tile_mode =
        hwl_degrade_thick_tile_mode(base_tile_mode, num_samples, tile_slices, is_depth);
    let rotation = compute_surface_rotation_from_tile_mode(tile_mode);

    if (rotation % M_PIPES) == 0 {
        match tile_mode {
            TileMode::D3TiledThin1 => {
                tile_mode = TileMode::D2TiledThin1;
            }
            TileMode::D3TiledThick => {
                tile_mode = TileMode::D2TiledThick;
            }
            TileMode::B3TiledThin1 => {
                tile_mode = TileMode::B2TiledThin1;
            }
            TileMode::B3TiledThick => {
                tile_mode = TileMode::B2TiledThick;
            }
            _ => (),
        }
    }

    if no_recursive || level == 0 {
        return tile_mode;
    }

    let mut bpp = bpp;
    if bpp == 96 || bpp == 48 || bpp == 24 {
        bpp /= 3;
    }

    let width = width.next_power_of_two();
    let height = height.next_power_of_two();
    let num_slices = num_slices.next_power_of_two();

    tile_mode = convert_to_non_bank_swapped_mode(tile_mode);

    let thickness = compute_surface_thickness(tile_mode);
    let micro_tile_bytes = bits_to_bytes(num_samples * bpp * thickness * 64);
    let mut width_align_factor = 1;

    if micro_tile_bytes <= M_PIPE_INTERLEAVE_BYTES {
        width_align_factor = M_PIPE_INTERLEAVE_BYTES / micro_tile_bytes;
    }

    let mut macro_tile_width = 8 * M_BANKS;
    let mut macro_tile_height = 8 * M_PIPES;

    // Reduce the tile mode from 2D/3D to 1D in following conditions
    match tile_mode {
        TileMode::D2TiledThin1 | TileMode::D3TiledThin1 => {
            if width < width_align_factor * macro_tile_width || height < macro_tile_height {
                tile_mode = TileMode::D1TiledThin1;
            }
        }
        TileMode::D2TiledThin2 => {
            macro_tile_width >>= 1;
            macro_tile_height *= 2;

            if width < width_align_factor * macro_tile_width || height < macro_tile_height {
                tile_mode = TileMode::D1TiledThin1;
            }
        }
        TileMode::D2TiledThin4 => {
            macro_tile_width >>= 2;
            macro_tile_height *= 4;

            if width < width_align_factor * macro_tile_width || height < macro_tile_height {
                tile_mode = TileMode::D1TiledThin1;
            }
        }
        TileMode::D2TiledThick | TileMode::D3TiledThick => {
            if width < width_align_factor * macro_tile_width || height < macro_tile_height {
                tile_mode = TileMode::D1TiledThick;
            }
        }
        _ => (),
    }

    if tile_mode == TileMode::D1TiledThick && num_slices < 4 {
        tile_mode = TileMode::D1TiledThin1;
    } else if tile_mode == TileMode::D2TiledThick && num_slices < 4 {
        tile_mode = TileMode::D2TiledThin1;
    } else if tile_mode == TileMode::D3TiledThick && num_slices < 4 {
        tile_mode = TileMode::D3TiledThin1;
    }

    compute_surface_mip_level_tile_mode(
        tile_mode,
        bpp,
        level,
        width,
        height,
        num_slices,
        num_samples,
        is_depth,
        true,
    )
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/core/addrlib.cpp#L606
#[allow(clippy::too_many_arguments)]
fn compute_surface_addr_from_coord_linear(
    x: u32,
    y: u32,
    slice: u32,
    sample: u32,
    bpp: u32,
    pitch: u32,
    height: u32,
    num_slices: u32,
) -> u32 {
    let slice_size = pitch * height;

    let slice_offset = slice_size * (slice + sample * num_slices);
    let row_offset = y * pitch;
    let pix_offset = x;

    (slice_offset + row_offset + pix_offset) * bpp / 8
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L670
fn compute_surface_alignments_linear(
    tile_mode: TileMode,
    bpp: u32,
    flags: SurfaceFlags,
    base_align: &mut u32,
    pitch_align: &mut u32,
    height_align: &mut u32,
) -> bool {
    let mut valid = true;

    match tile_mode {
        TileMode::LinearGeneral => {
            *base_align = 1;
            *pitch_align = if bpp != 1 { 1 } else { 8 };
            *height_align = 1;
        }
        TileMode::LinearAligned => {
            *base_align = M_PIPE_INTERLEAVE_BYTES;
            *pitch_align = ((8 * M_PIPE_INTERLEAVE_BYTES) / bpp).max(64);
            *height_align = 1;
        }
        _ => {
            *base_align = 1;
            *pitch_align = 1;
            *height_align = 1;
        }
    }

    adjust_pitch_alignment(flags, pitch_align);
    return valid;
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L714
fn compute_surface_alignments_micro_tiled(
    tile_mode: TileMode,
    bpp: u32,
    flags: SurfaceFlags,
    num_samples: u32,
    base_align: &mut u32,
    pitch_align: &mut u32,
    height_align: &mut u32,
) -> bool {
    let mut bpp = bpp;
    if bpp == 96 || bpp == 48 || bpp == 24 {
        bpp /= 3;
    }

    let micro_tile_thickness = compute_surface_thickness(tile_mode);
    let pitch_alignment = M_PIPE_INTERLEAVE_BYTES / bpp / num_samples / micro_tile_thickness;

    *base_align = M_PIPE_INTERLEAVE_BYTES;
    *pitch_align = pitch_alignment.max(8);
    *height_align = 8;

    adjust_pitch_alignment(flags, pitch_align);
    return true;
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L750
fn compute_macro_tile_aspect_ratio(tile_mode: TileMode) -> u32 {
    match tile_mode {
        TileMode::B2TiledThin1 | TileMode::D3TiledThin1 | TileMode::B3TiledThin1 => 1,
        TileMode::D2TiledThin2 | TileMode::B2TiledThin2 => 2,
        TileMode::D2TiledThin4 | TileMode::B2TiledThin4 => 4,
        _ => 1,
    }
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L781
fn is_dual_base_align_needed(tile_mode: TileMode) -> bool {
    if tile_mode > TileMode::D1TiledThick {
        true
    } else {
        false
    }
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L805
fn compute_surface_alignments_macrotiled(
    tile_mode: TileMode,
    bpp: u32,
    flags: SurfaceFlags,
    num_samples: u32,
    base_align: &mut u32,
    pitch_align: &mut u32,
    height_align: &mut u32,
    macro_width: &mut u32,
    macro_height: &mut u32,
) -> bool {
    let aspect_ratio = compute_macro_tile_aspect_ratio(tile_mode);
    let thickness = compute_surface_thickness(tile_mode);

    let mut bpp = bpp;
    if bpp == 96 || bpp == 48 || bpp == 24 {
        bpp /= 3;
    }

    if bpp == 3 {
        bpp = 1;
    }

    let num_banks = M_BANKS;
    let num_pipes = M_PIPES;
    let group_bytes = M_PIPE_INTERLEAVE_BYTES;
    let split_bytes = M_SPLIT_SIZE;
    *base_align = 0;

    let macro_tile_width = 8 * num_banks / aspect_ratio;
    let macro_tile_height = aspect_ratio * 8 * num_pipes;
    *pitch_align = std::cmp::max(
        macro_tile_width,
        macro_tile_width * (group_bytes / bpp / (8 * thickness) / num_samples),
    );

    *height_align = macro_tile_height;
    let mut macro_tile_bytes =
        num_samples * bits_to_bytes(bpp * macro_tile_height * macro_tile_width);

    if num_samples == 1 {
        macro_tile_bytes *= 2;
    }

    if thickness == 1 {
        *base_align = std::cmp::max(
            macro_tile_bytes,
            bits_to_bytes(num_samples * *height_align * bpp * *pitch_align),
        );
    } else {
        *base_align = std::cmp::max(
            group_bytes,
            bits_to_bytes(4 * *height_align * bpp * *pitch_align),
        );
    }

    let micro_tile_bytes = bits_to_bytes(thickness * num_samples * bpp * 64);
    let mut num_slices_per_micro_tile = 1;

    if micro_tile_bytes >= split_bytes {
        num_slices_per_micro_tile = micro_tile_bytes / split_bytes;
    }

    *base_align /= num_slices_per_micro_tile;

    if is_dual_base_align_needed(tile_mode) {
        let macro_bytes = bits_to_bytes(bpp * macro_tile_height * macro_tile_width);

        if *base_align / macro_bytes % 2 == 0 {
            *base_align += macro_bytes;
        }
    }

    *macro_width = macro_tile_width;
    *macro_height = macro_tile_height;
    return true;
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L889
fn compute_surface_info_linear(
    p_in: &ComputeSurfaceInfoInput,
    p_out: &mut ComputeSurfaceInfoOutput,
    pad_dims: u32,
    tile_mode: TileMode,
) {
    let micro_tile_thickness = compute_surface_thickness(tile_mode);
    let mut pitch = p_in.width;
    let mut height = p_in.height;
    let mut num_slices = p_in.num_slices;
    let num_samples = p_in.num_samples;
    let mip_level = p_in.mip_level;
    let bpp = p_in.bpp;
    let mut pad_dims = pad_dims;

    compute_surface_alignments_linear(
        p_in.tile_mode,
        p_in.bpp,
        p_in.flags,
        &mut p_out.base_align,
        &mut p_out.pitch_align,
        &mut p_out.height_align,
    );

    if p_in.flags.linear_wa() && mip_level == 0 {
        pitch = (pitch / 3).next_power_of_two();
    }

    if mip_level != 0 {
        pitch = pitch.next_power_of_two();
        height = height.next_power_of_two();

        if p_in.flags.cube() {
            if num_slices <= 1 {
                pad_dims = 2;
            } else {
                pad_dims = 0;
            }
        } else {
            num_slices = num_slices.next_power_of_two();
        }
    }

    pad_dimensions(
        tile_mode,
        p_in.flags,
        pad_dims,
        &mut pitch,
        p_out.pitch_align,
        &mut height,
        p_out.height_align,
        &mut num_slices,
        micro_tile_thickness,
    );

    if p_in.flags.linear_wa() && mip_level == 0 {
        pitch *= 3;
    }

    let slices = (num_slices * num_samples) / micro_tile_thickness;
    let surface_size = bits_to_bytes(height * pitch * slices * bpp * num_samples);

    p_out.pitch = pitch;
    p_out.height = height;
    p_out.depth = num_slices;
    p_out.surf_size = surface_size as u64;
    p_out.depth_align = micro_tile_thickness;
    p_out.tile_mode = tile_mode;
    //    return ADDR_OK; // TODO: return type?
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L969
fn compute_surface_info_micro_tiled(
    p_in: &ComputeSurfaceInfoInput,
    p_out: &mut ComputeSurfaceInfoOutput,
    pad_dims: u32,
    tile_mode: TileMode,
) {
    let mut micro_tile_thickness = compute_surface_thickness(tile_mode);
    let mut pitch = p_in.width;
    let mut height = p_in.height;
    let mut num_slices = p_in.num_slices;
    let num_samples = p_in.num_samples;
    let mip_level = p_in.mip_level;
    let bpp = p_in.bpp;
    let mut pad_dims = pad_dims;
    let mut tile_mode = tile_mode;

    if mip_level != 0 {
        pitch = pitch.next_power_of_two();
        height = height.next_power_of_two();

        if p_in.flags.cube() {
            if num_slices <= 1 {
                pad_dims = 2;
            } else {
                pad_dims = 0;
            }
        } else {
            num_slices = num_slices.next_power_of_two();
        }

        if tile_mode == TileMode::D1TiledThick && num_slices < 4 {
            tile_mode = TileMode::D1TiledThin1;
            micro_tile_thickness = 1;
        }
    }

    compute_surface_alignments_micro_tiled(
        tile_mode,
        p_in.bpp,
        p_in.flags,
        p_in.num_samples,
        &mut p_out.base_align,
        &mut p_out.pitch_align,
        &mut p_out.height_align,
    );

    pad_dimensions(
        tile_mode,
        p_in.flags,
        pad_dims,
        &mut pitch,
        p_out.pitch_align,
        &mut height,
        p_out.height_align,
        &mut num_slices,
        micro_tile_thickness,
    );

    let surface_size = bits_to_bytes(height * pitch * num_slices * bpp * num_samples);

    p_out.pitch = pitch;
    p_out.height = height;
    p_out.depth = num_slices;
    p_out.surf_size = surface_size as u64;
    p_out.tile_mode = tile_mode;
    p_out.depth_align = micro_tile_thickness;
    //    return ADDR_OK; TODO: return type?
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L1044
fn is_thick_macro_tiled(tile_mode: TileMode) -> bool {
    matches!(
        tile_mode,
        TileMode::D2TiledThick
            | TileMode::B2TiledThick
            | TileMode::D3TiledThick
            | TileMode::B3TiledThick
    )
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L1070
fn is_bank_swapped_tile_mode(tile_mode: TileMode) -> bool {
    matches!(
        tile_mode,
        TileMode::B2TiledThin1
            | TileMode::B2TiledThin2
            | TileMode::B2TiledThin4
            | TileMode::B2TiledThick
            | TileMode::B3TiledThin1
            | TileMode::B3TiledThick
    )
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L1098
fn is_dual_pitch_align_needed(tile_mode: TileMode, is_depth: bool, mip_level: u32) -> bool {
    if is_depth || mip_level != 0 {
        return false;
    }

    match tile_mode {
        TileMode::LinearGeneral
        | TileMode::LinearAligned
        | TileMode::D1TiledThin1
        | TileMode::D1TiledThick
        | TileMode::D2TiledThick
        | TileMode::B2TiledThick
        | TileMode::D3TiledThick
        | TileMode::B3TiledThick => false,
        _ => true,
    }
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L1134
fn compute_surface_bank_swapped_width(
    tile_mode: TileMode,
    bpp: u32,
    num_samples: u32,
    pitch: u32,
) -> u32 {
    let mut bank_swap_width = 0;
    let num_banks = M_BANKS;
    let num_pipes = M_PIPES;
    let swap_size = M_SWAP_SIZE;
    let row_size = M_ROW_SIZE;
    let split_size = M_SPLIT_SIZE;
    let group_size = M_PIPE_INTERLEAVE_BYTES;
    let mut slices_per_tile = 1;
    let bytes_per_sample = 8 * bpp;
    let samples_per_tile = split_size / bytes_per_sample;

    if (split_size / bytes_per_sample) != 0 {
        slices_per_tile = std::cmp::max(1, num_samples / samples_per_tile);
    }

    let mut num_samples = num_samples;
    if is_thick_macro_tiled(tile_mode) {
        num_samples = 4;
    }

    let bytes_per_tile_slice = num_samples * bytes_per_sample / slices_per_tile;

    if is_bank_swapped_tile_mode(tile_mode) {
        let factor = compute_macro_tile_aspect_ratio(tile_mode);
        let swap_tiles = std::cmp::max(1, (swap_size >> 1) / bpp);
        let swap_width = swap_tiles * 8 * num_banks;
        let height_bytes = num_samples * factor * num_pipes * bpp / slices_per_tile;
        let swap_max = num_pipes * num_banks * row_size / height_bytes;
        let swap_min = group_size * 8 * num_banks / bytes_per_tile_slice;

        bank_swap_width = std::cmp::min(swap_max, std::cmp::max(swap_min, swap_width));

        while bank_swap_width >= 2 * pitch {
            bank_swap_width >>= 1;
        }
    }

    bank_swap_width
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L1198
fn compute_surface_info_macro_tiled(
    p_in: &ComputeSurfaceInfoInput,
    p_out: &mut ComputeSurfaceInfoOutput,
    pad_dims: u32,
    tile_mode: TileMode,
    base_tile_mode: TileMode,
) {
    let mut macro_width = 0;
    let mut macro_height = 0;
    let mut micro_tile_thickness = compute_surface_thickness(tile_mode);
    let mut pitch = p_in.width;
    let mut height = p_in.height;
    let mut num_slices = p_in.num_slices;
    let num_samples = p_in.num_samples;
    let mip_level = p_in.mip_level;
    let bpp = p_in.bpp;
    let mut pitch_align = p_out.pitch_align;
    let mut pad_dims = pad_dims;
    let mut tile_mode = tile_mode;

    if mip_level != 0 {
        pitch = pitch.next_power_of_two();
        height = height.next_power_of_two();

        if p_in.flags.cube() {
            if num_slices <= 1 {
                pad_dims = 2;
            } else {
                pad_dims = 0;
            }
        } else {
            num_slices = num_slices.next_power_of_two();
        }

        if tile_mode == TileMode::D2TiledThick && num_slices < 4 {
            tile_mode = TileMode::D2TiledThin1;
            micro_tile_thickness = 1;
        }
    }

    if tile_mode != base_tile_mode
        && mip_level != 0
        && is_thick_macro_tiled(base_tile_mode)
        && !is_thick_macro_tiled(tile_mode)
    {
        compute_surface_alignments_macrotiled(
            base_tile_mode,
            p_in.bpp,
            p_in.flags,
            p_in.num_samples,
            &mut p_out.base_align,
            &mut p_out.pitch_align,
            &mut p_out.height_align,
            &mut macro_width,
            &mut macro_height,
        );

        let pitch_align_factor = ((M_PIPE_INTERLEAVE_BYTES >> 3) / bpp).max(1);

        if pitch < (p_out.pitch_align * pitch_align_factor) || height < p_out.height_align {
            return compute_surface_info_micro_tiled(p_in, p_out, pad_dims, TileMode::D1TiledThin1);
        }
    }

    compute_surface_alignments_macrotiled(
        tile_mode,
        p_in.bpp,
        p_in.flags,
        p_in.num_samples,
        &mut p_out.base_align,
        &mut pitch_align,
        &mut p_out.height_align,
        &mut macro_width,
        &mut macro_height,
    );

    let bank_swapped_width = compute_surface_bank_swapped_width(tile_mode, bpp, num_samples, pitch);
    pitch_align = pitch_align.max(bank_swapped_width);

    if is_dual_pitch_align_needed(tile_mode, p_in.flags.depth(), mip_level) {
        let mut tile_per_group = (M_PIPE_INTERLEAVE_BYTES >> 3) / bpp / num_samples;
        tile_per_group = (tile_per_group / compute_surface_thickness(tile_mode)).max(1);

        let even_width = ((pitch - 1) / macro_width) & 1;
        let even_height = ((height - 1) / macro_height) & 1;

        if num_samples == 1 && tile_per_group == 1 && even_width == 0 {
            if pitch > macro_width || (even_height == 0 && height > macro_height) {
                pitch += macro_width;
            }
        }
    }

    pad_dimensions(
        tile_mode,
        p_in.flags,
        pad_dims,
        &mut pitch,
        pitch_align,
        &mut height,
        p_out.height_align,
        &mut num_slices,
        micro_tile_thickness,
    );

    let surface_size = bits_to_bytes(height * pitch * num_slices * bpp * num_samples);

    p_out.pitch = pitch;
    p_out.height = height;
    p_out.depth = num_slices;
    p_out.surf_size = surface_size as u64;
    p_out.tile_mode = tile_mode;
    p_out.pitch_align = pitch_align;
    p_out.depth_align = micro_tile_thickness;
    // return ADDR_OK; // TODO: return type?
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L1315
pub fn hwl_compute_surface_info(
    p_in: &ComputeSurfaceInfoInput,
    p_out: &mut ComputeSurfaceInfoOutput,
) {
    let mut result;
    let num_samples = p_in.num_samples.max(1);
    let mut tile_mode = p_in.tile_mode;
    let mut pad_dims = 0;

    if p_in.flags.cube() && p_in.mip_level == 0 {
        pad_dims = 2;
    }

    if p_in.flags.fmask() {
        tile_mode = convert_to_non_bank_swapped_mode(tile_mode);
    } else {
        tile_mode = compute_surface_mip_level_tile_mode(
            tile_mode,
            p_in.bpp,
            p_in.mip_level,
            p_in.width,
            p_in.height,
            p_in.num_slices,
            num_samples,
            p_in.flags.depth(),
            false,
        );
    }

    match tile_mode {
        TileMode::LinearGeneral | TileMode::LinearAligned => {
            result = compute_surface_info_linear(p_in, p_out, pad_dims, tile_mode);
        }
        TileMode::D1TiledThin1 | TileMode::D1TiledThick => {
            result = compute_surface_info_micro_tiled(p_in, p_out, pad_dims, tile_mode);
        }
        TileMode::D2TiledThin1
        | TileMode::D2TiledThin2
        | TileMode::D2TiledThin4
        | TileMode::D2TiledThick
        | TileMode::B2TiledThin1
        | TileMode::B2TiledThin2
        | TileMode::B2TiledThin4
        | TileMode::B2TiledThick
        | TileMode::D3TiledThin1
        | TileMode::D3TiledThick
        | TileMode::B3TiledThin1
        | TileMode::B3TiledThick => {
            result =
                compute_surface_info_macro_tiled(p_in, p_out, pad_dims, tile_mode, p_in.tile_mode);
        }
        _ => {
            // TODO: return type?
            // result = ADDR_INVALIDPARAMS;
        }
    }

    // return result;
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L1384
fn get_tile_type(is_depth: bool) -> TileType {
    if is_depth {
        TileType::NonDisplayable
    } else {
        TileType::Displayable
    }
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L1407
#[allow(clippy::too_many_arguments)]
fn compute_surface_addr_from_coord_micro_tiled(
    x: u32,
    y: u32,
    slice: u32,
    bpp: u32,
    pitch: u32,
    height: u32,
    tile_mode: TileMode,
    is_depth: bool,
    tile_base: u32,
    comp_bits: u32,
) -> u32 {
    let mut micro_tile_thickness = 1;

    if tile_mode == TileMode::D1TiledThick {
        micro_tile_thickness = 4;
    }

    let micro_tile_bytes = bits_to_bytes(MICRO_TILE_PIXELS * micro_tile_thickness * bpp);
    let micro_tiles_per_row = pitch / MICRO_TILE_WIDTH;
    let micro_tile_index_x = x / MICRO_TILE_WIDTH;
    let micro_tile_index_y = y / MICRO_TILE_HEIGHT;
    let micro_tile_index_z = slice / micro_tile_thickness;

    let micro_tile_offset =
        micro_tile_bytes * (micro_tile_index_x + micro_tile_index_y * micro_tiles_per_row);

    let slice_bytes = bits_to_bytes(pitch * height * micro_tile_thickness * bpp);
    let slice_offset = micro_tile_index_z * slice_bytes;

    let pixel_index =
        compute_pixel_index_within_micro_tile(x, y, slice, bpp, tile_mode, get_tile_type(is_depth));
    let mut pixel_offset;

    if comp_bits != 0 && comp_bits != bpp && is_depth {
        pixel_offset = tile_base + comp_bits * pixel_index;
    } else {
        pixel_offset = bpp * pixel_index;
    }

    pixel_offset /= 8;

    pixel_offset + micro_tile_offset + slice_offset
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L1464
fn compute_pipe_from_coord_wo_rotation(x: u32, y: u32) -> u32 {
    let mut pipe_bit0 = 0;
    let mut pipe_bit1 = 0;
    let mut pipe_bit2 = 0;

    let x3 = bit(x, 3);
    let x4 = bit(x, 4);
    let x5 = bit(x, 5);
    let y3 = bit(y, 3);
    let y4 = bit(y, 4);
    let y5 = bit(y, 5);

    match M_PIPES {
        1 => {
            pipe_bit0 = 0;
        }
        2 => {
            pipe_bit0 = y3 ^ x3;
        }
        4 => {
            pipe_bit0 = y3 ^ x4;
            pipe_bit1 = y4 ^ x3;
        }
        8 => {
            pipe_bit0 = y3 ^ x5;
            pipe_bit1 = y4 ^ x5 ^ x4;
            pipe_bit2 = y5 ^ x3;
        }
        _ => (),
    }

    pipe_bit0 | (pipe_bit1 << 1) | (pipe_bit2 << 2)
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L1517
fn compute_bank_from_coord_wo_rotation(x: u32, y: u32) -> u32 {
    let num_pipes = M_PIPES;
    let num_banks = M_BANKS;
    let bank_opt = 1; // mConfigFlags.optimalBankSwap is always 1?

    let tx = x / num_banks;
    let ty = y / num_pipes;

    let mut bank_bit0 = 0;
    let mut bank_bit1 = 0;
    let mut bank_bit2 = 0;

    let x3 = bit(x, 3);
    let x4 = bit(x, 4);
    let x5 = bit(x, 5);

    let tx3 = bit(tx, 3);

    let ty3 = bit(ty, 3);
    let ty4 = bit(ty, 4);
    let ty5 = bit(ty, 5);

    match M_BANKS {
        4 => {
            bank_bit0 = ty4 ^ x3;

            if bank_opt == 1 && num_pipes == 8 {
                bank_bit0 ^= x5;
            }

            bank_bit1 = ty3 ^ x4;
        }
        8 => {
            bank_bit0 = ty5 ^ x3;

            if bank_opt == 1 && num_pipes == 8 {
                bank_bit0 ^= tx3;
            }

            bank_bit1 = ty5 ^ ty4 ^ x4;
            bank_bit2 = ty3 ^ x5;
        }
        _ => (),
    }

    bank_bit0 | (bank_bit1 << 1) | (bank_bit2 << 2)
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L1582
#[allow(clippy::too_many_arguments)]
fn compute_surface_addr_from_coord_macro_tiled(
    x: u32,
    y: u32,
    slice: u32,
    sample: u32,
    bpp: u32,
    pitch: u32,
    height: u32,
    num_samples: u32,
    tile_mode: TileMode,
    is_depth: bool,
    tile_base: u32,
    comp_bits: u32,
    pipe_swizzle: u32,
    bank_swizzle: u32,
) -> u32 {
    let num_pipes = M_PIPES;
    let num_banks = M_BANKS;
    let num_group_bits = M_PIPE_INTERLEAVE_BYTES.ilog2();
    let num_pipe_bits = M_PIPES.ilog2();
    let num_bank_bits = M_BANKS.ilog2();

    let micro_tile_thickness = compute_surface_thickness(tile_mode);
    let micro_tile_bits = MICRO_TILE_PIXELS * micro_tile_thickness * bpp * num_samples;
    let micro_tile_bytes = micro_tile_bits / 8;

    let pixel_index =
        compute_pixel_index_within_micro_tile(x, y, slice, bpp, tile_mode, get_tile_type(is_depth));

    let sample_offset;
    let pixel_offset;

    if is_depth {
        if comp_bits != 0 && comp_bits != bpp {
            sample_offset = tile_base + comp_bits * sample;
            pixel_offset = num_samples * comp_bits * pixel_index;
        } else {
            sample_offset = bpp * sample;
            pixel_offset = num_samples * bpp * pixel_index;
        }
    } else {
        sample_offset = sample * (micro_tile_bits / num_samples);
        pixel_offset = bpp * pixel_index;
    }

    let mut elem_offset = pixel_offset + sample_offset;

    let bytes_per_sample = micro_tile_bytes / num_samples;
    let samples_per_slice;
    let num_sample_splits;
    let sample_slice;
    let tile_slice_bits;

    let mut num_samples = num_samples;
    if num_samples > 1 && micro_tile_bytes > M_SPLIT_SIZE {
        samples_per_slice = M_SPLIT_SIZE / bytes_per_sample;
        num_sample_splits = num_samples / samples_per_slice;
        num_samples = samples_per_slice;

        tile_slice_bits = micro_tile_bits / num_sample_splits;
        sample_slice = elem_offset / tile_slice_bits;
        elem_offset %= tile_slice_bits;
    } else {
        num_sample_splits = 1;
        sample_slice = 0;
    }

    elem_offset /= 8;

    let mut pipe = compute_pipe_from_coord_wo_rotation(x, y);
    let mut bank = compute_bank_from_coord_wo_rotation(x, y);

    let mut bank_pipe = pipe + num_pipes * bank;
    let rotation = compute_surface_rotation_from_tile_mode(tile_mode);
    let swizzle = pipe_swizzle + num_pipes * bank_swizzle;
    let mut slice_in = slice;

    if is_thick_macro_tiled(tile_mode) {
        slice_in /= THICK_TILE_THICKNESS;
    }

    bank_pipe ^=
        (num_pipes * sample_slice * ((num_banks >> 1) + 1)) ^ (swizzle + slice_in * rotation);
    bank_pipe %= num_pipes * num_banks;
    pipe = bank_pipe % num_pipes;
    bank = bank_pipe / num_pipes;

    let slice_bytes = bits_to_bytes(pitch * height * micro_tile_thickness * bpp * num_samples);
    let slice_offset =
        slice_bytes * ((sample_slice + num_sample_splits * slice) / micro_tile_thickness);

    let mut macro_tile_pitch = 8 * num_banks;
    let mut macro_tile_height = 8 * num_pipes;

    match tile_mode {
        TileMode::D2TiledThin2 | TileMode::B2TiledThin2 => {
            macro_tile_pitch /= 2;
            macro_tile_height *= 2;
        }
        TileMode::D2TiledThin4 | TileMode::B2TiledThin4 => {
            macro_tile_pitch /= 4;
            macro_tile_height *= 4;
        }
        _ => (),
    }

    let macro_tiles_per_row = pitch / macro_tile_pitch;
    let macro_tile_bytes = bits_to_bytes(
        num_samples * micro_tile_thickness * bpp * macro_tile_height * macro_tile_pitch,
    );
    let macro_tile_index_x = x / macro_tile_pitch;
    let macro_tile_index_y = y / macro_tile_height;
    let macro_tile_offset =
        macro_tile_bytes * (macro_tile_index_x + macro_tiles_per_row * macro_tile_index_y);

    // Do bank swapping if needed
    if matches!(
        tile_mode,
        TileMode::B2TiledThin1
            | TileMode::B2TiledThin2
            | TileMode::B2TiledThin4
            | TileMode::B2TiledThick
            | TileMode::B3TiledThin1
            | TileMode::B3TiledThick
    ) {
        let bank_swap_order = [0, 1, 3, 2, 6, 7, 5, 4, 0, 0];
        let bank_swap_width =
            compute_surface_bank_swapped_width(tile_mode, bpp, num_samples, pitch);
        let swap_index = macro_tile_pitch * macro_tile_index_x / bank_swap_width;
        bank ^= bank_swap_order[(swap_index & (M_BANKS - 1)) as usize];
    }

    // Calculate final offset
    let group_mask = (1 << num_group_bits) - 1;
    let total_offset =
        elem_offset + ((macro_tile_offset + slice_offset) >> (num_bank_bits + num_pipe_bits));

    // TODO: offset_high is causing indexing out of bounds?
    let offset_high = (total_offset & !group_mask) << (num_bank_bits + num_pipe_bits);
    let offset_low = total_offset & group_mask;
    let bank_bits = bank << (num_pipe_bits + num_group_bits);
    let pipe_bits = pipe << num_group_bits;
    bank_bits | pipe_bits | offset_low | offset_high
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L1736
pub fn dispatch_compute_surface_addrfrom_coord(p_in: &ComputeSurfaceAddrFromCoordInput) -> u32 {
    let num_samples = std::cmp::max(1, p_in.num_samples);

    match p_in.tile_mode {
        TileMode::LinearGeneral | TileMode::LinearAligned => {
            compute_surface_addr_from_coord_linear(
                p_in.x,
                p_in.y,
                p_in.slice,
                p_in.sample,
                p_in.bpp,
                p_in.pitch,
                p_in.height,
                p_in.num_slices,
            )
        }
        TileMode::D1TiledThin1 | TileMode::D1TiledThick => {
            compute_surface_addr_from_coord_micro_tiled(
                p_in.x,
                p_in.y,
                p_in.slice,
                p_in.bpp,
                p_in.pitch,
                p_in.height,
                p_in.tile_mode,
                p_in.is_depth,
                p_in.tile_base,
                p_in.comp_bits,
            )
        }
        TileMode::D2TiledThin1
        | TileMode::D2TiledThin2
        | TileMode::D2TiledThin4
        | TileMode::D2TiledThick
        | TileMode::B2TiledThin1
        | TileMode::B2TiledThin2
        | TileMode::B2TiledThin4
        | TileMode::B2TiledThick
        | TileMode::D3TiledThin1
        | TileMode::D3TiledThick
        | TileMode::B3TiledThin1
        | TileMode::B3TiledThick => compute_surface_addr_from_coord_macro_tiled(
            p_in.x,
            p_in.y,
            p_in.slice,
            p_in.sample,
            p_in.bpp,
            p_in.pitch,
            p_in.height,
            num_samples,
            p_in.tile_mode,
            p_in.is_depth,
            p_in.tile_base,
            p_in.comp_bits,
            p_in.pipe_swizzle,
            p_in.bank_swizzle,
        ),
        _ => 0,
    }
}

// Pipe and bank swizzle values from Cemu.
// License: https://github.com/cemu-project/Cemu/blob/main/LICENSE.txt
// https://github.com/cemu-project/Cemu/blob/85141f17f977157b91b72883d879f50b27f17dda/src/Cafe/HW/Latte/Core/LatteTextureLoader.cpp#L30-L31
pub fn pipe_bank_swizzle(swizzle: u32) -> (u32, u32) {
    ((swizzle >> 8) & 1, (swizzle >> 9) & 3)
}
