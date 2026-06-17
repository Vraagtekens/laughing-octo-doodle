# Bevy Creative Coding Sketchbook

This repo is a Cargo workspace for Nannou / openFrameworks / TouchDesigner-style Bevy sketches.

Each sketch lives in `sketches/<name>` as its own crate with its own `src` and `assets` folder. The root `Cargo.toml` only coordinates the workspace and shared dependency versions.

## Shared Code

Reusable sketch components live in `crates/sketchbook`.

### FPS Overlay

Add `sketchbook.workspace = true` to a sketch's `Cargo.toml`, then add the plugin:

```rust
use sketchbook::FpsOverlayPlugin;

App::new()
    .add_plugins((DefaultPlugins, FpsOverlayPlugin))
    .run();
```

Press `H` to toggle the FPS readout.

## Sketches

### `default_template`

A very small starter sketch with:

- A window
- A camera
- One white square whose color comes from a tiny WGSL material shader

Run it from the repo root:

```sh
cargo run -p default_template
```

### `render_texture_template`

A starter patch with:

- An offscreen render texture canvas at `1280x720`
- A sketch camera that renders animated 2D geometry into that texture
- A screen camera that draws the texture through a custom post-processing material
- WGSL shader files for both generative material color and post-processing
- Bevy bloom, tonemapping, and deband dithering on the final camera
- Render layers to keep sketch-space and screen-space passes separate

Run it from the repo root:

```sh
cargo run -p render_texture_template
```

Or from the sketch folder:

```sh
cd sketches/render_texture_template
cargo run
```

### `mouse_trail`

A cursor sketch where a white square follows the mouse and paints permanent square stamps behind it. Press `H` to toggle the FPS readout in the top-left corner.

Run it from the repo root:

```sh
cargo run -p mouse_trail
```

### `sprite_grid_trail`

A zoomed-in, jittered grid of image sprites loaded from the configured asset folder, such as `assets/pokemon` or `assets/cats`, paints into a persistent render texture. It drifts on its own; arrow keys temporarily take over the field motion, then the automatic drift continues from wherever you left it. Whole rows/columns wrap only after leaving the visible camera area, re-enter just offscreen on the opposite side, and pick fresh random sprites. Press `H` to toggle the FPS readout.

Run it from the repo root:

```sh
cargo run -p sprite_grid_trail
```

## Layout

```text
.
├── Cargo.toml
├── Cargo.lock
├── crates
│   └── sketchbook
│       ├── Cargo.toml
│       └── src/lib.rs
└── sketches
    ├── default_template
    │   ├── Cargo.toml
    │   ├── src/main.rs
    │   └── assets/shaders/square.wgsl
    ├── mouse_trail
    │   ├── Cargo.toml
    │   └── src/main.rs
    ├── render_texture_template
    │   ├── Cargo.toml
    │   ├── src/main.rs
    │   └── assets/shaders
    │       ├── sketch_material.wgsl
    │       └── post_material.wgsl
    └── sprite_grid_trail
        ├── Cargo.toml
        └── src/main.rs
```

## Add A Sketch

1. Create `sketches/my_new_sketch`.
2. Add a `Cargo.toml` with `bevy.workspace = true`.
3. Add it to the root workspace `members`.
4. Put its shaders, images, and other files under `sketches/my_new_sketch/assets`.

In each sketch, this pattern makes asset loading independent of your current working directory:

```rust
DefaultPlugins.set(AssetPlugin {
    file_path: format!("{}/assets", env!("CARGO_MANIFEST_DIR")),
    ..default()
})
```
