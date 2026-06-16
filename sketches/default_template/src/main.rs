use bevy::{
    asset::AssetPlugin,
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
    sprite_render::{Material2d, Material2dPlugin},
};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const SQUARE_SHADER: &str = "shaders/square.wgsl";

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: format!("{}/assets", env!("CARGO_MANIFEST_DIR")),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Default Sketch Template".into(),
                        resolution: (WIDTH, HEIGHT).into(),
                        ..default()
                    }),
                    ..default()
                }),
            Material2dPlugin::<SquareMaterial>::default(),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .run();
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct SquareMaterial {
    #[uniform(0)]
    params: SquareParams,
}

#[derive(Clone, Copy, Debug, ShaderType)]
struct SquareParams {
    color: LinearRgba,
}

impl Material2d for SquareMaterial {
    fn fragment_shader() -> ShaderRef {
        SQUARE_SHADER.into()
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SquareMaterial>>,
) {
    commands.spawn(Camera2d);

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(240.0, 240.0))),
        MeshMaterial2d(materials.add(SquareMaterial {
            params: SquareParams {
                color: LinearRgba::WHITE,
            },
        })),
    ));
}
