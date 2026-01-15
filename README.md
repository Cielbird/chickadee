# chickadee engine

help from tim-lappe setting up `wgpu` with `winit`

big help from https://sotrh.github.io/learn-wgpu/

the purpose of this engine is to be simple and easy to use. i am using it for personal projects.

## features

- *entity* hierachy of inherited 3d transformations
- game logic written with *components*
- simple box colliders
- easily add Models to entities from `obj` files
- retro asthetic

## cross-compilation for windows x86

`rustup target add x86_64-pc-windows-gnu`

`brew install mingw-w64`

`cargo build --target x86_64-pc-windows-gnu`

for some reason, `x86_64-pc-windows-msvc` has issues.

`PROJECT_OUT_DIR` should be defined to the location of the `res` folder for object loading. 
very subject to change!
