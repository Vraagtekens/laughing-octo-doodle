use bevy::prelude::*;
use sketchbook::{FpsOverlayPlugin, local_asset_path, sketch_plugins};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const SQUARE_SIZE: f32 = 56.0;
const TRAIL_STEP_SECONDS: f32 = 0.025;

fn main() {
    App::new()
        .add_plugins((
            sketch_plugins(
                "Mouse Trail",
                WIDTH,
                HEIGHT,
                local_asset_path(env!("CARGO_MANIFEST_DIR")),
            ),
            FpsOverlayPlugin,
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(TrailClock(Timer::from_seconds(
            TRAIL_STEP_SECONDS,
            TimerMode::Repeating,
        )))
        .add_systems(Startup, setup)
        .add_systems(Update, (move_square, stamp_trail).chain())
        .run();
}

#[derive(Component)]
struct CursorSquare;

#[derive(Resource)]
struct TrailClock(Timer);

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite::from_color(Color::WHITE, Vec2::splat(SQUARE_SIZE)),
        Transform::from_xyz(0.0, 0.0, 10.0),
        CursorSquare,
    ));
}

fn move_square(
    mut square: Single<&mut Transform, With<CursorSquare>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
) {
    let (camera, camera_transform) = *camera_query;

    if let Some(cursor_position) = window.cursor_position()
        && let Ok(cursor_world_position) =
            camera.viewport_to_world_2d(camera_transform, cursor_position)
    {
        square.translation.x = cursor_world_position.x;
        square.translation.y = cursor_world_position.y;
    }
}

fn stamp_trail(
    mut commands: Commands,
    time: Res<Time>,
    mut trail_clock: ResMut<TrailClock>,
    square: Single<&Transform, With<CursorSquare>>,
) {
    trail_clock.0.tick(time.delta());

    if !trail_clock.0.just_finished() {
        return;
    }

    commands.spawn((
        Sprite::from_color(Color::srgba(1.0, 1.0, 1.0, 0.55), Vec2::splat(SQUARE_SIZE)),
        Transform::from_xyz(square.translation.x, square.translation.y, 0.0),
    ));
}
