use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::Bloom,
    prelude::*,
    reflect::TypePath,
    render::render_resource::{
        AsBindGroup, Extent3d, ShaderType, TextureDescriptor, TextureDimension, TextureFormat,
        TextureUsages,
    },
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin},
    window::WindowResized,
};
use sketchbook::{local_asset_path, sketch_plugins};

const SKETCH_SHADER: &str = "shaders/sketch_material.wgsl";
const POST_SHADER: &str = "shaders/post_material.wgsl";

const CANVAS_WIDTH: u32 = 1280;
const CANVAS_HEIGHT: u32 = 720;

const SKETCH_LAYER: RenderLayers = RenderLayers::layer(0);
const SCREEN_LAYER: RenderLayers = RenderLayers::layer(1);

fn main() {
    App::new()
        .add_plugins((
            sketch_plugins(
                "Render Texture Template",
                CANVAS_WIDTH,
                CANVAS_HEIGHT,
                local_asset_path(env!("CARGO_MANIFEST_DIR")),
            ),
            Material2dPlugin::<SketchMaterial>::default(),
            Material2dPlugin::<PostMaterial>::default(),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(Update, (animate_sketch, update_shader_time, fit_canvas))
        .run();
}

#[derive(Component)]
struct SketchObject {
    orbit_radius: f32,
    orbit_speed: f32,
    spin_speed: f32,
    phase: f32,
}

#[derive(Component)]
struct ScreenCamera;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct SketchMaterial {
    #[uniform(0)]
    params: SketchParams,
}

#[derive(Clone, Copy, Debug, ShaderType)]
struct SketchParams {
    color_a: LinearRgba,
    color_b: LinearRgba,
    time: f32,
    index: f32,
    _padding: Vec2,
}

impl Material2d for SketchMaterial {
    fn fragment_shader() -> ShaderRef {
        SKETCH_SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct PostMaterial {
    #[texture(0)]
    #[sampler(1)]
    source: Handle<Image>,
    #[uniform(2)]
    params: PostParams,
}

#[derive(Clone, Copy, Debug, ShaderType)]
struct PostParams {
    resolution: Vec2,
    time: f32,
    feedback_mix: f32,
    vignette: f32,
    chroma: f32,
    _padding: Vec2,
}

impl Material2d for PostMaterial {
    fn fragment_shader() -> ShaderRef {
        POST_SHADER.into()
    }
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut sketch_materials: ResMut<Assets<SketchMaterial>>,
    mut post_materials: ResMut<Assets<PostMaterial>>,
) {
    let render_texture = images.add(render_texture(CANVAS_WIDTH, CANVAS_HEIGHT));

    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            clear_color: ClearColorConfig::Custom(Color::linear_rgb(0.005, 0.006, 0.012)),
            ..default()
        },
        RenderTarget::Image(render_texture.clone().into()),
        Msaa::Off,
        SKETCH_LAYER,
    ));

    spawn_sketch(&mut commands, &mut meshes, &mut sketch_materials);

    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Tonemapping::TonyMcMapface,
        Bloom::NATURAL,
        DebandDither::Enabled,
        Msaa::Off,
        ScreenCamera,
        SCREEN_LAYER,
    ));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(CANVAS_WIDTH as f32, CANVAS_HEIGHT as f32))),
        MeshMaterial2d(post_materials.add(PostMaterial {
            source: render_texture,
            params: PostParams {
                resolution: Vec2::new(CANVAS_WIDTH as f32, CANVAS_HEIGHT as f32),
                time: 0.0,
                feedback_mix: 0.62,
                vignette: 0.42,
                chroma: 1.35,
                _padding: Vec2::ZERO,
            },
        })),
        SCREEN_LAYER,
    ));
}

fn render_texture(width: u32, height: u32) -> Image {
    let size = Extent3d {
        width,
        height,
        ..default()
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("creative_render_texture"),
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

fn spawn_sketch(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<SketchMaterial>,
) {
    let palettes = [
        (
            LinearRgba::rgb(8.0, 0.75, 1.2),
            LinearRgba::rgb(0.2, 5.0, 7.0),
        ),
        (
            LinearRgba::rgb(0.6, 6.0, 2.3),
            LinearRgba::rgb(9.0, 2.0, 0.8),
        ),
        (
            LinearRgba::rgb(0.9, 1.3, 8.0),
            LinearRgba::rgb(8.0, 6.0, 0.8),
        ),
    ];

    for i in 0..18 {
        let t = i as f32 / 18.0;
        let mesh = if i % 3 == 0 {
            meshes.add(Circle::new(54.0 + 12.0 * (i % 4) as f32))
        } else if i % 3 == 1 {
            meshes.add(RegularPolygon::new(70.0, 3 + (i % 6) as u32))
        } else {
            meshes.add(Rectangle::new(95.0, 95.0))
        };

        let (color_a, color_b) = palettes[i % palettes.len()];
        commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(SketchMaterial {
                params: SketchParams {
                    color_a,
                    color_b,
                    time: 0.0,
                    index: i as f32,
                    _padding: Vec2::ZERO,
                },
            })),
            Transform::from_xyz(0.0, 0.0, t * 20.0)
                .with_rotation(Quat::from_rotation_z(t * std::f32::consts::TAU)),
            SketchObject {
                orbit_radius: 90.0 + t * 250.0,
                orbit_speed: 0.25 + t * 1.2,
                spin_speed: -1.4 + t * 3.2,
                phase: t * std::f32::consts::TAU,
            },
            SKETCH_LAYER,
        ));
    }
}

fn animate_sketch(time: Res<Time>, mut query: Query<(&SketchObject, &mut Transform)>) {
    let elapsed = time.elapsed_secs();
    for (object, mut transform) in &mut query {
        let theta = elapsed * object.orbit_speed + object.phase;
        let wobble = (elapsed * 1.7 + object.phase).sin() * 35.0;
        transform.translation.x = theta.cos() * (object.orbit_radius + wobble);
        transform.translation.y = theta.sin() * object.orbit_radius * 0.58;
        transform.rotation = Quat::from_rotation_z(elapsed * object.spin_speed + object.phase);
    }
}

fn update_shader_time(
    time: Res<Time>,
    mut sketch_materials: ResMut<Assets<SketchMaterial>>,
    mut post_materials: ResMut<Assets<PostMaterial>>,
) {
    let elapsed = time.elapsed_secs();
    for (_, material) in sketch_materials.iter_mut() {
        material.params.time = elapsed;
    }
    for (_, material) in post_materials.iter_mut() {
        material.params.time = elapsed;
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
        let horizontal = event.width / CANVAS_WIDTH as f32;
        let vertical = event.height / CANVAS_HEIGHT as f32;
        projection.scale = 1.0 / horizontal.min(vertical).max(0.001);
    }
}
