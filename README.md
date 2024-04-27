# wiiu_swizzle 

[![Latest Version](https://img.shields.io/crates/v/wiiu_swizzle.svg)](https://crates.io/crates/wiiu_swizzle) [![docs.rs](https://docs.rs/wiiu_swizzle/badge.svg)](https://docs.rs/wiiu_swizzle)

A safe and efficient pure Rust implementation of texture memory tiling for the Wii U. This library is still experimental and missing important features.

## Credits
Much of this project is translated from open source C++ code from the Mesa driver adapted by [decaf-emu/addrlib](https://github.com/decaf-emu/addrlib) ([license](https://github.com/decaf-emu/addrlib/blob/master/LICENSE)) with Wii U specific constants taken from [Cemu](https://github.com/cemu-project/Cemu) ([license](https://github.com/cemu-project/Cemu/blob/main/LICENSE.txt)). Cemu is also used to generate data for test cases by extracting "deswizzled" surface data from RenderDoc. See the source code for details.
