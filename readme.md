# What42's Wgpu Template

This is a template for using Rust + Winit + Wgpu to build native desktop programs. It was originally created by following the [Learn Wgpu](https://sotrh.github.io/learn-wgpu/) tutorial, but I've completely restructured everything and added many features. This is basically a mini-engine, giving simple implementations for common tasks.

I don't exactly expect others to want to use this, but it'll still be very useful for myself (not just for making programs, but also updating this template's dependencies to see how to update other programs' dependencies).

## NOTE: Even though I'd like for this to work on any OS, only Windows 10 is tested for now.

<br>

## Current Features:

- **Model Loading**
- **Skyboxes**
- **Shadows** (soon, hopefully)
- **Text render** (soon, hopefully)
- **Post-Processing** (soon, hopefully)
- **Texture Compression** (soon, possibly)

<br>

## Qualities:

- **Simple, but Upfront** &nbsp; everything has the most straight-forward implementation I could think of, but none of the complexity is trying to be hidden
- **Flexible** &nbsp; this is my measure for how clean code is, and I always strive to keep my code as maluable as possible
- **Well Documented** &nbsp; self-documented wherever possible, with comments to explain any oddities
- **Up-to-date Dependencies** &nbsp; uses the latest crates available, at least at the time of writing this
- **Thorough Error Handling**

<br>

### Credits

- [Ben Hanson](https://github.com/sotrh): Learn Wgpu
- [Luke.RUSTLTD](https://opengameart.org/users/lukerustltd): Skybox Texture

<br>

### License: [CC0](LICENSE)

This license allows you to do anything you want with this code.
