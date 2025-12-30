# chickadee engine

help from tim-lappe setting up `wgpu` with `winit`

big help from https://sotrh.github.io/learn-wgpu/

the purpose of this engine is to be simple and easy to use. i am using it for personal projects.

## cross-compilation for windows x86

`rustup target add x86_64-pc-windows-gnu`

`brew install mingw-w64`

`cargo build --target x86_64-pc-windows-gnu`

for some reason, `x86_64-pc-windows-msvc` has issues.

