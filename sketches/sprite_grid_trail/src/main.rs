use bevy::{
    asset::AssetPlugin,
    camera::{RenderTarget, visibility::RenderLayers},
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
};
use sketchbook::FpsOverlayPlugin;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const COLS: i32 = 9;
const ROWS: i32 = 5;
const CELL_GAP: Vec2 = Vec2::new(128.0, 112.0);
const JITTER: Vec2 = Vec2::new(42.0, 34.0);
const SPRITE_SIZE: f32 = 78.0;

const PAINT_LAYER: RenderLayers = RenderLayers::layer(0);
const SCREEN_LAYER: RenderLayers = RenderLayers::layer(1);

const POKEMON_SPRITES: [&str; 8] = [
    "pokemon/revival-herb.png",
    "pokemon/awakening.png",
    "pokemon/moomoo-milk.png",
    "pokemon/leftovers.png",
    "pokemon/normal.png",
    "pokemon/max-revive.png",
    "pokemon/calcium.png",
    "pokemon/rare-candy.png",
];

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: format!("{}/../../assets", env!("CARGO_MANIFEST_DIR")),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Sprite Grid Trail".into(),
                        resolution: (WIDTH, HEIGHT).into(),
                        ..default()
                    }),
                    ..default()
                }),
            FpsOverlayPlugin,
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(Update, move_sprites)
        .run();
}

#[derive(Component)]
struct MovingSprite {
    home: Vec2,
    phase: f32,
    spin: f32,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    let paint_texture = images.add(paint_texture(WIDTH, HEIGHT));

    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        RenderTarget::Image(paint_texture.clone().into()),
        Msaa::Off,
        PAINT_LAYER,
    ));

    commands.spawn((
        Sprite {
            image: paint_texture,
            custom_size: Some(Vec2::new(WIDTH as f32, HEIGHT as f32)),
            ..default()
        },
        SCREEN_LAYER,
    ));

    commands.spawn((Camera2d, Msaa::Off, SCREEN_LAYER));

    let image_handles = POKEMON_SPRITES
        .iter()
        .map(|path| asset_server.load(*path))
        .collect::<Vec<Handle<Image>>>();

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
            let size = Vec2::splat(SPRITE_SIZE * (0.72 + hash(index as f32 + 31.0) * 0.58));
            let phase = hash(index as f32 + 101.0) * std::f32::consts::TAU;
            let spin = signed_hash(index as f32 + 251.0) * 0.18;
            let home = centered + jitter;

            commands.spawn((
                Sprite {
                    image: image_handles[(index * 5) % image_handles.len()].clone(),
                    custom_size: Some(size),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.42),
                    ..default()
                },
                Transform::from_xyz(home.x, home.y, 10.0),
                MovingSprite { home, phase, spin },
                PAINT_LAYER,
            ));
        }
    }
}

fn paint_texture(width: u32, height: u32) -> Image {
    let size = Extent3d {
        width,
        height,
        ..default()
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("sprite_grid_paint_texture"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    image.resize(size);
    image
}

fn move_sprites(time: Res<Time>, mut query: Query<(&MovingSprite, &mut Transform)>) {
    let t = time.elapsed_secs();
    let layer_offset = Vec2::new((t * 0.63).sin() * 150.0, (t * 0.47).cos() * 88.0);

    for (sprite, mut transform) in &mut query {
        let local_drift = Vec2::new(
            (t * 0.9 + sprite.phase).sin() * 10.0,
            (t * 0.7 + sprite.phase).cos() * 8.0,
        );
        let position = sprite.home + layer_offset + local_drift;

        transform.translation.x = position.x;
        transform.translation.y = position.y;
        transform.rotation = Quat::from_rotation_z((t + sprite.phase).sin() * sprite.spin);
    }
}

fn hash(value: f32) -> f32 {
    (value.sin() * 43_758.547).fract().abs()
}

fn signed_hash(value: f32) -> f32 {
    hash(value) * 2.0 - 1.0
}
