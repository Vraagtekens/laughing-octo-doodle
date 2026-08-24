use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    prelude::*,
    reflect::TypePath,
    render::render_resource::{
        AsBindGroup, Extent3d, ShaderType, TextureDescriptor, TextureDimension, TextureFormat,
        TextureUsages,
    },
    shader::ShaderRef,
    sprite_render::{Material2d, Material2dPlugin},
    window::WindowResized,
};
#[cfg(not(target_arch = "wasm32"))]
use sketchbook::RecordingSource;
use sketchbook::{SketchControls, SketchControlsPlugin, local_asset_path, sketch_plugins};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const LOOP_FRAMES: u32 = 250;
const FRAME_RATE: f32 = 30.0;
const SLIME_SHADER: &str = "shaders/slime_mold.wgsl";

const CONTROL_DENSITY: &str = "density";
const CONTROL_FILAMENTS: &str = "filaments";
const CONTROL_GLOW: &str = "glow";
const CONTROL_THICKNESS: &str = "thickness";
const CONTROL_REACH: &str = "reach";
const CONTROL_CONTRAST: &str = "contrast";

const SKETCH_LAYER: RenderLayers = RenderLayers::layer(0);
const SCREEN_LAYER: RenderLayers = RenderLayers::layer(1);

fn main() {
    App::new()
        .add_plugins((
            default_plugins(),
            Material2dPlugin::<SlimeMaterial>::default(),
            SketchControlsPlugin::new("Slime Mold Loop")
                .with_target_fps(FRAME_RATE)
                .with_slider(CONTROL_DENSITY, "cell irregularity", 1.0, 0.35..=2.0)
                .with_slider(CONTROL_FILAMENTS, "edge detail", 7.0, 2.0..=14.0)
                .with_slider(CONTROL_THICKNESS, "wall thickness", 0.22, 0.05..=0.38)
                .with_slider(CONTROL_REACH, "cell movement", 0.62, 0.0..=1.6)
                .with_slider(CONTROL_GLOW, "diffusion", 0.72, 0.0..=2.0)
                .with_slider(CONTROL_CONTRAST, "contrast", 1.45, 0.7..=3.0),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(LoopClock::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            ((advance_loop, update_material).chain(), fit_canvas),
        )
        .run();
}

fn default_plugins() -> bevy::app::PluginGroupBuilder {
    sketch_plugins(
        "250 Frame Slime Mold Loop",
        WIDTH,
        HEIGHT,
        local_asset_path(env!("CARGO_MANIFEST_DIR")),
    )
}

#[derive(Resource, Default)]
struct LoopClock {
    frame: u32,
}

#[derive(Component)]
struct ScreenCamera;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct SlimeMaterial {
    #[uniform(0)]
    params: SlimeParams,
}

#[derive(Clone, Copy, Debug, ShaderType)]
struct SlimeParams {
    resolution: Vec2,
    phase: f32,
    density: f32,
    filaments: f32,
    glow: f32,
    thickness: f32,
    reach: f32,
    contrast: f32,
    _padding: Vec2,
}

impl Material2d for SlimeMaterial {
    fn fragment_shader() -> ShaderRef {
        SLIME_SHADER.into()
    }
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SlimeMaterial>>,
) {
    let output = images.add(render_texture(WIDTH, HEIGHT));

    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        RenderTarget::Image(output.clone().into()),
        Msaa::Off,
        SKETCH_LAYER,
    ));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(WIDTH as f32, HEIGHT as f32))),
        MeshMaterial2d(materials.add(SlimeMaterial {
            params: SlimeParams {
                resolution: Vec2::new(WIDTH as f32, HEIGHT as f32),
                phase: 0.0,
                density: 1.0,
                filaments: 7.0,
                glow: 0.72,
                thickness: 0.22,
                reach: 0.62,
                contrast: 1.45,
                _padding: Vec2::ZERO,
            },
        })),
        SKETCH_LAYER,
    ));

    commands.spawn((Camera2d, Msaa::Off, ScreenCamera, SCREEN_LAYER));
    commands.spawn((
        Sprite {
            image: output.clone(),
            custom_size: Some(Vec2::new(WIDTH as f32, HEIGHT as f32)),
            ..default()
        },
        SCREEN_LAYER,
    ));

    #[cfg(not(target_arch = "wasm32"))]
    commands.insert_resource(RecordingSource::Image(output.clone()));
}

fn advance_loop(mut clock: ResMut<LoopClock>) {
    clock.frame = (clock.frame + 1) % LOOP_FRAMES;
}

fn update_material(
    clock: Res<LoopClock>,
    controls: Res<SketchControls>,
    mut materials: ResMut<Assets<SlimeMaterial>>,
) {
    let phase = clock.frame as f32 / LOOP_FRAMES as f32;
    for (_, material) in materials.iter_mut() {
        material.params.phase = phase;
        material.params.density = controls.value(CONTROL_DENSITY);
        material.params.filaments = controls.value(CONTROL_FILAMENTS);
        material.params.glow = controls.value(CONTROL_GLOW);
        material.params.thickness = controls.value(CONTROL_THICKNESS);
        material.params.reach = controls.value(CONTROL_REACH);
        material.params.contrast = controls.value(CONTROL_CONTRAST);
    }
}

fn fit_canvas(
    mut resize_events: MessageReader<WindowResized>,
    mut projection: Single<&mut Projection, With<ScreenCamera>>,
) {
    let Projection::Orthographic(projection) = &mut **projection else {
        return;
    };

    for event in resize_events.read() {
        let horizontal = event.width / WIDTH as f32;
        let vertical = event.height / HEIGHT as f32;
        projection.scale = 1.0 / horizontal.min(vertical).max(0.001);
    }
}

fn render_texture(width: u32, height: u32) -> Image {
    let size = Extent3d {
        width,
        height,
        ..default()
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("slime_mold_loop_output"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
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
