#[cfg(not(target_arch = "wasm32"))]
use std::{fs, path::Path};

use bevy::{camera::visibility::RenderLayers, prelude::*};
use sketchbook::{
    SketchControls, SketchControlsPlugin,
    feedback_loop::{
        FeedbackLoopEffect, FeedbackLoopPlugin, FeedbackLoopSettings, spawn_feedback_loop,
    },
    sketch_plugins, workspace_asset_path,
};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const CAMERA_ZOOM: f32 = 0.72;
const COLS: i32 = 11;
const ROWS: i32 = 7;
const CELL_GAP: Vec2 = Vec2::new(128.0, 112.0);
const VISIBLE_HALF: Vec2 = Vec2::new(
    WIDTH as f32 * CAMERA_ZOOM * 0.5,
    HEIGHT as f32 * CAMERA_ZOOM * 0.5,
);
const JITTER: Vec2 = Vec2::new(42.0, 34.0);
const SPRITE_SIZE: f32 = 78.0;
const FEEDBACK_SCALE: f32 = 0.972;
const FEEDBACK_ALPHA: f32 = 0.965;
const GRID_SPEED: f32 = 320.0;
const WRAP_EXIT_MARGIN: Vec2 = Vec2::new(SPRITE_SIZE * 1.8, SPRITE_SIZE * 1.8);
const WRAP_ENTER_MARGIN: Vec2 = Vec2::new(SPRITE_SIZE * 1.4, SPRITE_SIZE * 1.4);
const SPRITE_ASSET_FOLDER: &str = "pokemon";
#[cfg(not(target_arch = "wasm32"))]
const SPRITE_ASSET_EXTENSIONS: [&str; 5] = ["png", "jpg", "jpeg", "gif", "webp"];
const CONTROL_FEEDBACK_SCALE: &str = "feedback_scale";
const CONTROL_FEEDBACK_ALPHA: &str = "feedback_alpha";
const CONTROL_SPRITE_SIZE: &str = "sprite_size";
const CONTROL_GRID_SPEED: &str = "grid_speed";

const PAINT_LAYER: RenderLayers = RenderLayers::layer(0);
const SCREEN_LAYER: RenderLayers = RenderLayers::layer(1);

fn main() {
    App::new()
        .add_plugins((
            sketch_plugins("Feedback Sprite Grid", WIDTH, HEIGHT, asset_root()),
            SketchControlsPlugin::new("Feedback Sprite Grid")
                .with_slider(
                    CONTROL_FEEDBACK_SCALE,
                    "feedback scale",
                    FEEDBACK_SCALE,
                    0.90..=1.0,
                )
                .with_slider(
                    CONTROL_FEEDBACK_ALPHA,
                    "trail fade",
                    FEEDBACK_ALPHA,
                    0.80..=1.0,
                )
                .with_slider(
                    CONTROL_SPRITE_SIZE,
                    "sprite size",
                    SPRITE_SIZE,
                    24.0..=160.0,
                )
                .with_slider(CONTROL_GRID_SPEED, "grid speed", GRID_SPEED, 0.0..=800.0),
            FeedbackLoopPlugin,
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(GridMotion::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (move_sprites, update_feedback_effect))
        .run();
}

#[derive(Resource)]
struct GridMotion {
    offset: Vec2,
    previous_auto_offset: Vec2,
}

impl Default for GridMotion {
    fn default() -> Self {
        let offset = automatic_offset(0.0);
        Self {
            offset,
            previous_auto_offset: offset,
        }
    }
}

#[derive(Resource)]
struct SpriteImages {
    handles: Vec<Handle<Image>>,
}

#[derive(Component)]
struct MovingSprite {
    home: Vec2,
    column_x: f32,
    row_y: f32,
    size_factor: f32,
    phase: f32,
    spin: f32,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    let canvas = spawn_feedback_loop(
        &mut commands,
        &mut images,
        FeedbackLoopSettings::new(WIDTH, HEIGHT)
            .with_zoom(CAMERA_ZOOM)
            .with_label("feedback_sprite_grid_texture")
            .with_layers(PAINT_LAYER, SCREEN_LAYER)
            .with_feedback(FEEDBACK_SCALE, FEEDBACK_ALPHA),
    );

    let sprite_paths = sprite_asset_paths();
    let image_handles = sprite_paths
        .iter()
        .map(|path| asset_server.load(path.clone()))
        .collect::<Vec<Handle<Image>>>();
    commands.insert_resource(SpriteImages {
        handles: image_handles.clone(),
    });

    for y in 0..ROWS {
        for x in 0..COLS {
            let index = (y * COLS + x) as usize;
            let centered = Vec2::new(
                (x as f32 - (COLS - 1) as f32 * 0.5) * CELL_GAP.x,
                (y as f32 - (ROWS - 1) as f32 * 0.5) * CELL_GAP.y,
            );
            let jitter = Vec2::new(
                signed_hash(index as f32 + 12.0) * JITTER.x,
                signed_hash(index as f32 + 77.0) * JITTER.y,
            );
            let size_factor = 0.72 + hash(index as f32 + 31.0) * 0.58;
            let size = SPRITE_SIZE * size_factor;
            let phase = hash(index as f32 + 101.0) * std::f32::consts::TAU;
            let spin = signed_hash(index as f32 + 251.0) * 0.18;
            let home = centered + jitter;

            commands.spawn((
                Sprite {
                    image: image_handles[random_index(index, image_handles.len())].clone(),
                    custom_size: Some(Vec2::splat(size)),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.42),
                    ..default()
                },
                Transform::from_xyz(home.x, home.y, 10.0),
                MovingSprite {
                    home,
                    column_x: centered.x,
                    row_y: centered.y,
                    size_factor,
                    phase,
                    spin,
                },
                canvas.paint_layer.clone(),
            ));
        }
    }
}

fn move_sprites(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    images: Res<SpriteImages>,
    controls: Res<SketchControls>,
    mut motion: ResMut<GridMotion>,
    mut query: Query<(&mut MovingSprite, &mut Sprite, &mut Transform)>,
) {
    let t = time.elapsed_secs();
    let mut direction = Vec2::ZERO;
    if keys.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }
    if keys.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }

    let auto_offset = automatic_offset(t);
    let auto_delta = auto_offset - motion.previous_auto_offset;
    motion.previous_auto_offset = auto_offset;

    if direction != Vec2::ZERO {
        motion.offset +=
            direction.normalize() * controls.value(CONTROL_GRID_SPEED) * time.delta_secs();
    } else {
        motion.offset += auto_delta;
    }

    for (mut moving_sprite, mut sprite, mut transform) in &mut query {
        sprite.custom_size = Some(Vec2::splat(
            controls.value(CONTROL_SPRITE_SIZE) * moving_sprite.size_factor,
        ));
        let local_drift = Vec2::new(
            (t * 0.9 + moving_sprite.phase).sin() * 10.0,
            (t * 0.7 + moving_sprite.phase).cos() * 8.0,
        );
        let mut position = moving_sprite.home + motion.offset + local_drift;

        wrap_sprite_home(
            &mut moving_sprite,
            &mut sprite,
            &images,
            &motion,
            &mut position,
            t,
        );

        transform.translation.x = position.x;
        transform.translation.y = position.y;
        transform.rotation =
            Quat::from_rotation_z((t + moving_sprite.phase).sin() * moving_sprite.spin);
    }
}

fn update_feedback_effect(
    controls: Res<SketchControls>,
    mut effect: Single<(&mut Sprite, &mut Transform), With<FeedbackLoopEffect>>,
) {
    let (sprite, transform) = &mut *effect;
    sprite.color = Color::srgba(1.0, 1.0, 1.0, controls.value(CONTROL_FEEDBACK_ALPHA));
    transform.scale = Vec3::splat(controls.value(CONTROL_FEEDBACK_SCALE));
}

fn automatic_offset(t: f32) -> Vec2 {
    Vec2::new((t * 0.63).sin() * 150.0, (t * 0.47).cos() * 88.0)
}

fn wrap_sprite_home(
    moving_sprite: &mut MovingSprite,
    sprite: &mut Sprite,
    images: &SpriteImages,
    motion: &GridMotion,
    position: &mut Vec2,
    t: f32,
) {
    let exit_half = VISIBLE_HALF + WRAP_EXIT_MARGIN;
    let enter_half = VISIBLE_HALF + WRAP_ENTER_MARGIN;
    let column_position = moving_sprite.column_x + motion.offset.x;
    let row_position = moving_sprite.row_y + motion.offset.y;
    let mut wrapped = false;

    if column_position > exit_half.x {
        let overshoot = column_position - exit_half.x;
        let target = -enter_half.x + overshoot;
        let delta = target - column_position;
        moving_sprite.home.x += delta;
        moving_sprite.column_x += delta;
        position.x += delta;
        wrapped = true;
    }
    if column_position < -exit_half.x {
        let overshoot = column_position + exit_half.x;
        let target = enter_half.x + overshoot;
        let delta = target - column_position;
        moving_sprite.home.x += delta;
        moving_sprite.column_x += delta;
        position.x += delta;
        wrapped = true;
    }

    if row_position > exit_half.y {
        let overshoot = row_position - exit_half.y;
        let target = -enter_half.y + overshoot;
        let delta = target - row_position;
        moving_sprite.home.y += delta;
        moving_sprite.row_y += delta;
        position.y += delta;
        wrapped = true;
    }
    if row_position < -exit_half.y {
        let overshoot = row_position + exit_half.y;
        let target = enter_half.y + overshoot;
        let delta = target - row_position;
        moving_sprite.home.y += delta;
        moving_sprite.row_y += delta;
        position.y += delta;
        wrapped = true;
    }

    if wrapped {
        let index = random_index_from_value(t + moving_sprite.phase * 19.0, images.handles.len());
        sprite.image = images.handles[index].clone();
    }
}

fn asset_root() -> String {
    workspace_asset_path(env!("CARGO_MANIFEST_DIR"))
}

#[cfg(target_arch = "wasm32")]
fn sprite_asset_paths() -> Vec<String> {
    POKEMON_ASSETS
        .iter()
        .map(|asset| format!("{SPRITE_ASSET_FOLDER}/{asset}"))
        .collect()
}

#[cfg(not(target_arch = "wasm32"))]
fn sprite_asset_paths() -> Vec<String> {
    let sprite_dir = Path::new(&asset_root()).join(SPRITE_ASSET_FOLDER);
    let mut paths = fs::read_dir(&sprite_dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .filter_map(|entry| {
            let path = entry.path();
            let file_name = path.file_name()?.to_string_lossy();
            if file_name.starts_with("._") {
                return None;
            }

            let is_supported_image = path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| {
                    SPRITE_ASSET_EXTENSIONS
                        .iter()
                        .any(|supported| extension.eq_ignore_ascii_case(supported))
                });

            is_supported_image.then(|| format!("{SPRITE_ASSET_FOLDER}/{file_name}"))
        })
        .collect::<Vec<_>>();

    paths.sort();
    if paths.is_empty() {
        panic!(
            "No supported image files found in {}. Expected one of: {}",
            sprite_dir.display(),
            SPRITE_ASSET_EXTENSIONS.join(", ")
        );
    }
    paths
}

#[cfg(target_arch = "wasm32")]
const POKEMON_ASSETS: &[&str] = &[
    "awakening.png",
    "calcium.png",
    "leftovers.png",
    "max-revive.png",
    "moomoo-milk.png",
    "normal.png",
    "rare-candy.png",
    "revival-herb.png",
];

fn random_index(index: usize, len: usize) -> usize {
    ((hash(index as f32 + 911.0) * len as f32).floor() as usize).min(len - 1)
}

fn random_index_from_value(value: f32, len: usize) -> usize {
    ((hash(value + 1337.0) * len as f32).floor() as usize).min(len - 1)
}

fn hash(value: f32) -> f32 {
    (value.sin() * 43_758.547).fract().abs()
}

fn signed_hash(value: f32) -> f32 {
    hash(value) * 2.0 - 1.0
}
