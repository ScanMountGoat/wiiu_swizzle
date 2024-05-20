# wiiu_swizzle 

[![Latest Version](https://img.shields.io/crates/v/wiiu_swizzle.svg)](https://crates.io/crates/wiiu_swizzle) [![docs.rs](https://docs.rs/wiiu_swizzle/badge.svg)](https://docs.rs/wiiu_swizzle)

A safe and efficient pure Rust implementation of texture memory tiling or "swizzling" for the Wii U. This library is still experimental and missing important features and bug fixes.

## Memory Tiling
GPU textures are often stored in a tiled memory layout to make texture accesses more cache friendly. The standard linear or row-major memory ordering is only cache friendly when the data is accessed in row-major order. This is rarely the case for image textures for models, so the bytes of a surface are rearranged to improve the number of cache misses using some form of tiling algorithm.

It's important to note that tiling affects memory addressing, so surfaces should be thought of as 2D or 3D arrays of bytes rather than pixels or 4x4 pixel blocks. Some functions require the bytes or bits per pixel for use in addrlib code. The "pixel" for compressed formats is typically a 4x4 block of pixels. The surface dimensions in pixels do not need to be powers of two for tiling to work correctly.

This technique has often been referred to in modding communities as "swizzling", "deswizzling", "unswizzling", or "un-swizzling". It's not accurate to describe the tiled address bits as rearranged or "swizzled" from linear address bits for all texture sizes. Thankfully, common usages of the term "swizzling" in modding communities almost always refer specifically to the tiled memory layout algorithm. The term "swizzling" is kept in crate and function names to improve discoverability, as this is likely what most programmers will search for.

## Credits
Much of this project was translated from open source C++ code from the Mesa driver adapted by [decaf-emu/addrlib](https://github.com/decaf-emu/addrlib) ([license](https://github.com/decaf-emu/addrlib/blob/master/LICENSE)) with Wii U specific constants taken from [Cemu](https://github.com/cemu-project/Cemu) ([license](https://github.com/cemu-project/Cemu/blob/main/LICENSE.txt)). Cemu was also used to generate data for test cases by extracting "deswizzled" surface data from RenderDoc using special textures with a unique index assigned to each chunk of 4 bytes. This provides a mapping from linear to tiled memory addresses. See the source code for details.
