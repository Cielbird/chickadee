# wgpu_test

rust 1.85.0

help from tim-lappe setting up `wgpu` with `winit`

big help from https://sotrh.github.io/learn-wgpu/

## cross-compilation for windows x86

`rustup target add x86_64-pc-windows-gnu`

`brew install mingw-w64`

`cargo build --target x86_64-pc-windows-gnu`

for some reason, `x86_64-pc-windows-msvc` has issues.


## engine

the engine is not flexible, it doesn't need to be.

coordinates are left handed, x to the right, y up, z out of screen
