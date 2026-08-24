# Bevy Creative Coding Sketchbook

This repo is a Cargo workspace for Nannou / openFrameworks / TouchDesigner-style Bevy sketches.

Each sketch lives in `sketches/<name>` as its own crate with its own `src` and `assets` folder. The root `Cargo.toml` only coordinates the workspace and shared dependency versions.

## Shared Code

Reusable sketch components live in `crates/sketchbook`.

### Sketch Controls

Add `sketchbook.workspace = true` to a sketch's `Cargo.toml`, then declare the controls that sketch needs:

```rust
use bevy::prelude::*;
use sketchbook::{SketchControls, SketchControlsPlugin};

App::new()
    .add_plugins((
        DefaultPlugins,
        SketchControlsPlugin::new("My Sketch")
            .with_target_fps(60.0)
            .with_slider("speed", "speed", 100.0, 0.0..=500.0),
    ))
    .add_systems(Update, update_sketch)
    .run();

fn update_sketch(controls: Res<SketchControls>) {
    let speed = controls.value("speed");
    // Use `speed` in the sketch here.
}
```

Press `H` to toggle the panel. Every panel includes a live FPS readout, an app FPS limiter, recording to a timestamped PNG sequence under `recordings/`, and Reset. Feedback loops automatically record their clean render texture without the settings panel. Other render-texture sketches can set `RecordingSource::Image(handle)` to select their clean output. Sketches without custom controls can keep using `FpsOverlayPlugin` to get only the shared FPS and recording tools.

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

A cursor sketch where a white square follows the mouse and paints permanent square stamps behind it. Press `H` to toggle the sketch controls.

Run it from the repo root:

```sh
cargo run -p mouse_trail
```

### `sprite_grid_trail`

A zoomed-in, jittered grid of image sprites loaded from the configured asset folder, such as `assets/pokemon` or `assets/cats`, paints into a persistent render texture. It drifts on its own; arrow keys temporarily take over the field motion, then the automatic drift continues from wherever you left it. Whole rows/columns wrap only after leaving the visible camera area, re-enter just offscreen on the opposite side, and pick fresh random sprites. Press `H` to toggle the sketch controls.

Run it from the repo root:

```sh
cargo run -p sprite_grid_trail
```

### `slime_mold_loop`

A GPU-generated slime mold colony with a deterministic 250-frame seamless loop. Press `H` for live density, filament, glow, and color controls plus clean render-texture recording.

```sh
cargo run -p slime_mold_loop
```

## Web Deployments

The repo can publish sketches as a static WebAssembly site. The first deployed sketch is `slime_mold_loop`.

Each deployed sketch has:

- a generated Trunk entry at `.trunk/<name>/index.html`
- a standalone route in `dist/<name>/`
- a manifest entry in `public/manifest.json`

The shared sketch HTML lives in `web/sketch.html`. Edit that file to change
the canvas shell for every sketch; the scripts only swap the title, summary,
asset copy links, and Cargo manifest path.

Install the WASM target and Trunk locally:

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk
```

Build one sketch:

```sh
npm run build:sketch -- slime_mold_loop
```

Serve one sketch while developing:

```sh
npm run serve:sketch -- slime_mold_loop
```

Build the full static sketch site:

```sh
npm run build:all
```

Preview the full static site from `dist/`:

```sh
npm run serve
```

Then open `http://127.0.0.1:5500/`. Do not preview `web/index.html`
directly with Live Server: `web/` is only the source shell, while routes like
`/slime_mold_loop/` are generated into `dist/`.

GitHub Actions deploys `dist/` to GitHub Pages on pushes to `main`. Enable Pages in the repository settings and select GitHub Actions as the source.

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
