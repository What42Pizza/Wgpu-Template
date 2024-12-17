# What42's Wgpu Template

This is a template for using Rust + Winit + Wgpu to build native desktop programs. I don't exactly expect others to want to use this, but it'll still be very useful for myself (not just for making programs, but also updating this template's dependencies to see how to update other programs' dependencies).

## NOTES
- Even though I'd like for this to work on any OS, only Windows 10 is tested for now.
- You may want to use sdl3-rs instead of winit, you can see how to do so [here](https://github.com/revmischa/sdl3-rs/blob/master/examples/raw-window-handle-with-wgpu/main.rs)

<br>

## Current Features:

- **Model Loading and Rendering**
- **Skybox Loading and Rendering**
- **Shadows**
- **Frustum Culling**
- **Texture Compression**

<br>

## Qualities:

- **Hackable** &nbsp; everything is simple and flexible, and features can be added / removed with ease
- **Well Documented** &nbsp; self-documented wherever possible, with comments to explain any oddities
- **Up-to-date Dependencies** &nbsp; uses the latest crates available, at least at the time of writing this (no deprecated code either)
- **Thorough Error Handling**

<br>

### Credits

- [Ben Hanson](https://github.com/sotrh): Learn Wgpu
- [Luke.RUSTLTD](https://opengameart.org/users/lukerustltd): Skybox Texture

<br>

### License: [CC0](LICENSE)

This license allows you to do anything you want with this code.
