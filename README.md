# wiiu_swizzle
A safe and efficient pure Rust implementation of texture memory tiling for the Wii U. This library is still experimental and missing important features.

## Credits
This project translates open source C++ code from the Mesa driver adapted by [decaf-emu/addrlib](https://github.com/decaf-emu/addrlib) ([license](https://github.com/decaf-emu/addrlib/blob/master/LICENSE)) with constants adapted from [Cemu](https://github.com/cemu-project/Cemu) ([license](https://github.com/cemu-project/Cemu/blob/main/LICENSE.txt)). Cemu is also used to generate data for test cases by extracting "deswizzled" surface data from RenderDoc. See the source code for details.