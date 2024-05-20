#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: wiiu_swizzle::Gx2Surface| {
    let _ = input.deswizzle();
});
