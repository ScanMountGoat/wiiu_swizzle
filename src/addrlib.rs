// TODO: Fix names.
// TODO: Preserve enum variant names?

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

// TODO: Add original name in doc comments from gx2 enums?
// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/include/addrlib/addrtypes.h
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum TileMode {
    ADDR_TM_LINEAR_GENERAL = 0x0,
    ADDR_TM_LINEAR_ALIGNED = 0x1,
    ADDR_TM_1D_TILED_THIN1 = 0x2,
    ADDR_TM_1D_TILED_THICK = 0x3,
    ADDR_TM_2D_TILED_THIN1 = 0x4,
    ADDR_TM_2D_TILED_THIN2 = 0x5,
    ADDR_TM_2D_TILED_THIN4 = 0x6,
    ADDR_TM_2D_TILED_THICK = 0x7,
    ADDR_TM_2B_TILED_THIN1 = 0x8,
    ADDR_TM_2B_TILED_THIN2 = 0x9,
    ADDR_TM_2B_TILED_THIN4 = 0xA,
    ADDR_TM_2B_TILED_THICK = 0xB,
    ADDR_TM_3D_TILED_THIN1 = 0xC,
    ADDR_TM_3D_TILED_THICK = 0xD,
    ADDR_TM_3B_TILED_THIN1 = 0xE,
    ADDR_TM_3B_TILED_THICK = 0xF,
    // ADDR_TM_LINEAR_SPECIAL = 0x10,
    ADDR_TM_2D_TILED_XTHICK = 0x10,
    ADDR_TM_3D_TILED_XTHICK = 0x11,
    ADDR_TM_POWER_SAVE = 0x12,
    ADDR_TM_COUNT = 0x13,
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/include/addrlib/addrtypes.h#L230C1-L237C1
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddrTileType {
    ADDR_DISPLAYABLE = 0x0,
    ADDR_NON_DISPLAYABLE = 0x1,
    // ADDR_DEPTH_SAMPLE_ORDER = 0x2,
    ADDR_THICK_TILING = 0x3,
}

// Modified to remove unused fields.
// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/include/addrlib/addrinterface.h#L343
#[derive(Debug)]
pub struct AddrComputeSurfaceAddrFromCoordInput {
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
        TileMode::ADDR_TM_1D_TILED_THICK
        | TileMode::ADDR_TM_2D_TILED_THICK
        | TileMode::ADDR_TM_2B_TILED_THICK
        | TileMode::ADDR_TM_3D_TILED_THICK
        | TileMode::ADDR_TM_3B_TILED_THICK => 4,
        TileMode::ADDR_TM_2D_TILED_XTHICK | TileMode::ADDR_TM_3D_TILED_XTHICK => 8,
        _ => 1,
    }
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/core/addrlib.cpp#L643
fn compute_pixel_index_within_micro_tile(
    x: u32,
    y: u32,
    z: u32,
    bpp: u32,
    tile_mode: TileMode,
    tile_type: AddrTileType,
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

    if tile_type == AddrTileType::ADDR_THICK_TILING {
        pixel_bit0 = x0;
        pixel_bit1 = y0;
        pixel_bit2 = z0;
        pixel_bit3 = x1;
        pixel_bit4 = y1;
        pixel_bit5 = z1;
        pixel_bit6 = x2;
        pixel_bit7 = y2;
    } else {
        if tile_type == AddrTileType::ADDR_NON_DISPLAYABLE {
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
fn convert_to_non_bank_swapped_model(tile_mode: TileMode) -> TileMode {
    match tile_mode {
        TileMode::ADDR_TM_2B_TILED_THIN1 => TileMode::ADDR_TM_2D_TILED_THIN1,
        TileMode::ADDR_TM_2B_TILED_THIN2 => TileMode::ADDR_TM_2D_TILED_THIN2,
        TileMode::ADDR_TM_2B_TILED_THIN4 => TileMode::ADDR_TM_2D_TILED_THIN4,
        TileMode::ADDR_TM_2B_TILED_THICK => TileMode::ADDR_TM_2D_TILED_THICK,
        TileMode::ADDR_TM_3B_TILED_THIN1 => TileMode::ADDR_TM_3D_TILED_THIN1,
        TileMode::ADDR_TM_3B_TILED_THICK => TileMode::ADDR_TM_3D_TILED_THICK,
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
        TileMode::ADDR_TM_2D_TILED_THIN1
        | TileMode::ADDR_TM_2D_TILED_THIN2
        | TileMode::ADDR_TM_2D_TILED_THIN4
        | TileMode::ADDR_TM_2D_TILED_THICK
        | TileMode::ADDR_TM_2B_TILED_THIN1
        | TileMode::ADDR_TM_2B_TILED_THIN2
        | TileMode::ADDR_TM_2B_TILED_THIN4
        | TileMode::ADDR_TM_2B_TILED_THICK => M_PIPES * ((M_BANKS >> 1) - 1),
        TileMode::ADDR_TM_3D_TILED_THIN1
        | TileMode::ADDR_TM_3D_TILED_THICK
        | TileMode::ADDR_TM_3B_TILED_THIN1
        | TileMode::ADDR_TM_3B_TILED_THICK => {
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
        TileMode::ADDR_TM_1D_TILED_THIN1 => {
            // TODO: Is this case used?
            // if (num_samples > 1 && mConfigFlags.no1DTiledMSAA) {
            //     tile_mode = TileMode::ADDR_TM_2D_TILED_THIN1;
            // }
        }
        TileMode::ADDR_TM_1D_TILED_THICK => {
            if num_samples > 1 || is_depth {
                tile_mode = TileMode::ADDR_TM_1D_TILED_THIN1;
            }

            if num_samples == 2 || num_samples == 4 {
                tile_mode = TileMode::ADDR_TM_2D_TILED_THICK;
            }
        }
        TileMode::ADDR_TM_2D_TILED_THIN2 => {
            if 2 * M_PIPE_INTERLEAVE_BYTES > M_SPLIT_SIZE {
                tile_mode = TileMode::ADDR_TM_2D_TILED_THIN1;
            }
        }
        TileMode::ADDR_TM_2D_TILED_THIN4 => {
            if 4 * M_PIPE_INTERLEAVE_BYTES > M_SPLIT_SIZE {
                tile_mode = TileMode::ADDR_TM_2D_TILED_THIN2;
            }
        }
        TileMode::ADDR_TM_2D_TILED_THICK => {
            if num_samples > 1 || tile_slices > 1 || is_depth {
                tile_mode = TileMode::ADDR_TM_2D_TILED_THIN1;
            }
        }
        TileMode::ADDR_TM_2B_TILED_THIN2 => {
            if 2 * M_PIPE_INTERLEAVE_BYTES > M_SPLIT_SIZE {
                tile_mode = TileMode::ADDR_TM_2B_TILED_THIN1;
            }
        }
        TileMode::ADDR_TM_2B_TILED_THIN4 => {
            if 4 * M_PIPE_INTERLEAVE_BYTES > M_SPLIT_SIZE {
                tile_mode = TileMode::ADDR_TM_2B_TILED_THIN2;
            }
        }
        TileMode::ADDR_TM_2B_TILED_THICK => {
            if num_samples > 1 || tile_slices > 1 || is_depth {
                tile_mode = TileMode::ADDR_TM_2B_TILED_THIN1;
            }
        }
        TileMode::ADDR_TM_3D_TILED_THICK => {
            if num_samples > 1 || tile_slices > 1 || is_depth {
                tile_mode = TileMode::ADDR_TM_3D_TILED_THIN1;
            }
        }
        TileMode::ADDR_TM_3B_TILED_THICK => {
            if num_samples > 1 || tile_slices > 1 || is_depth {
                tile_mode = TileMode::ADDR_TM_3B_TILED_THIN1;
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
            TileMode::ADDR_TM_3D_TILED_THIN1 => {
                tile_mode = TileMode::ADDR_TM_2D_TILED_THIN1;
            }
            TileMode::ADDR_TM_3D_TILED_THICK => {
                tile_mode = TileMode::ADDR_TM_2D_TILED_THICK;
            }
            TileMode::ADDR_TM_3B_TILED_THIN1 => {
                tile_mode = TileMode::ADDR_TM_2B_TILED_THIN1;
            }
            TileMode::ADDR_TM_3B_TILED_THICK => {
                tile_mode = TileMode::ADDR_TM_2B_TILED_THICK;
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

    tile_mode = convert_to_non_bank_swapped_model(tile_mode);

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
        TileMode::ADDR_TM_2D_TILED_THIN1 | TileMode::ADDR_TM_3D_TILED_THIN1 => {
            if width < width_align_factor * macro_tile_width || height < macro_tile_height {
                tile_mode = TileMode::ADDR_TM_1D_TILED_THIN1;
            }
        }
        TileMode::ADDR_TM_2D_TILED_THIN2 => {
            macro_tile_width >>= 1;
            macro_tile_height *= 2;

            if width < width_align_factor * macro_tile_width || height < macro_tile_height {
                tile_mode = TileMode::ADDR_TM_1D_TILED_THIN1;
            }
        }
        TileMode::ADDR_TM_2D_TILED_THIN4 => {
            macro_tile_width >>= 2;
            macro_tile_height *= 4;

            if width < width_align_factor * macro_tile_width || height < macro_tile_height {
                tile_mode = TileMode::ADDR_TM_1D_TILED_THIN1;
            }
        }
        TileMode::ADDR_TM_2D_TILED_THICK | TileMode::ADDR_TM_3D_TILED_THICK => {
            if width < width_align_factor * macro_tile_width || height < macro_tile_height {
                tile_mode = TileMode::ADDR_TM_1D_TILED_THICK;
            }
        }
        _ => (),
    }

    if tile_mode == TileMode::ADDR_TM_1D_TILED_THICK && num_slices < 4 {
        tile_mode = TileMode::ADDR_TM_1D_TILED_THIN1;
    } else if tile_mode == TileMode::ADDR_TM_2D_TILED_THICK && num_slices < 4 {
        tile_mode = TileMode::ADDR_TM_2D_TILED_THIN1;
    } else if tile_mode == TileMode::ADDR_TM_3D_TILED_THICK && num_slices < 4 {
        tile_mode = TileMode::ADDR_TM_3D_TILED_THIN1;
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

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L750
fn compute_macro_tile_aspect_ratio(tile_mode: TileMode) -> u32 {
    match tile_mode {
        TileMode::ADDR_TM_2B_TILED_THIN1
        | TileMode::ADDR_TM_3D_TILED_THIN1
        | TileMode::ADDR_TM_3B_TILED_THIN1 => 1,
        TileMode::ADDR_TM_2D_TILED_THIN2 | TileMode::ADDR_TM_2B_TILED_THIN2 => 2,
        TileMode::ADDR_TM_2D_TILED_THIN4 | TileMode::ADDR_TM_2B_TILED_THIN4 => 4,
        _ => 1,
    }
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L1044
fn is_thick_macro_tiled(tile_mode: TileMode) -> bool {
    matches!(
        tile_mode,
        TileMode::ADDR_TM_2D_TILED_THICK
            | TileMode::ADDR_TM_2B_TILED_THICK
            | TileMode::ADDR_TM_3D_TILED_THICK
            | TileMode::ADDR_TM_3B_TILED_THICK
    )
}

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L1070
fn is_bank_swapped_tile_mode(tile_mode: TileMode) -> bool {
    matches!(
        tile_mode,
        TileMode::ADDR_TM_2B_TILED_THIN1
            | TileMode::ADDR_TM_2B_TILED_THIN2
            | TileMode::ADDR_TM_2B_TILED_THIN4
            | TileMode::ADDR_TM_2B_TILED_THICK
            | TileMode::ADDR_TM_3B_TILED_THIN1
            | TileMode::ADDR_TM_3B_TILED_THICK
    )
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

// https://github.com/decaf-emu/addrlib/blob/194162c47469ce620dd2470eb767ff5e42f5954a/src/r600/r600addrlib.cpp#L1384
fn get_tile_type(is_depth: bool) -> AddrTileType {
    if is_depth {
        AddrTileType::ADDR_NON_DISPLAYABLE
    } else {
        AddrTileType::ADDR_DISPLAYABLE
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

    if tile_mode == TileMode::ADDR_TM_1D_TILED_THICK {
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
        TileMode::ADDR_TM_2D_TILED_THIN2 | TileMode::ADDR_TM_2B_TILED_THIN2 => {
            macro_tile_pitch /= 2;
            macro_tile_height *= 2;
        }
        TileMode::ADDR_TM_2D_TILED_THIN4 | TileMode::ADDR_TM_2B_TILED_THIN4 => {
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
        TileMode::ADDR_TM_2B_TILED_THIN1
            | TileMode::ADDR_TM_2B_TILED_THIN2
            | TileMode::ADDR_TM_2B_TILED_THIN4
            | TileMode::ADDR_TM_2B_TILED_THICK
            | TileMode::ADDR_TM_3B_TILED_THIN1
            | TileMode::ADDR_TM_3B_TILED_THICK
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
pub fn dispatch_compute_surface_addrfrom_coord(p_in: &AddrComputeSurfaceAddrFromCoordInput) -> u32 {
    let num_samples = std::cmp::max(1, p_in.num_samples);

    match p_in.tile_mode {
        TileMode::ADDR_TM_LINEAR_GENERAL | TileMode::ADDR_TM_LINEAR_ALIGNED => {
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
        TileMode::ADDR_TM_1D_TILED_THIN1 | TileMode::ADDR_TM_1D_TILED_THICK => {
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
        TileMode::ADDR_TM_2D_TILED_THIN1
        | TileMode::ADDR_TM_2D_TILED_THIN2
        | TileMode::ADDR_TM_2D_TILED_THIN4
        | TileMode::ADDR_TM_2D_TILED_THICK
        | TileMode::ADDR_TM_2B_TILED_THIN1
        | TileMode::ADDR_TM_2B_TILED_THIN2
        | TileMode::ADDR_TM_2B_TILED_THIN4
        | TileMode::ADDR_TM_2B_TILED_THICK
        | TileMode::ADDR_TM_3D_TILED_THIN1
        | TileMode::ADDR_TM_3D_TILED_THICK
        | TileMode::ADDR_TM_3B_TILED_THIN1
        | TileMode::ADDR_TM_3B_TILED_THICK => compute_surface_addr_from_coord_macro_tiled(
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
