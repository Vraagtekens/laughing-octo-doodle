use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
};

#[cfg(not(target_arch = "wasm32"))]
use crate::RecordingSource;

pub struct FeedbackLoopPlugin;

impl Plugin for FeedbackLoopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, advance_feedback_loop);
    }
}

#[derive(Clone)]
pub struct FeedbackLoopCanvas {
    pub paint_layer: RenderLayers,
    pub screen_layer: RenderLayers,
    pub textures: [Handle<Image>; 2],
}

#[derive(Clone)]
pub struct FeedbackLoopSettings {
    pub width: u32,
    pub height: u32,
    pub zoom: f32,
    pub label: &'static str,
    pub paint_layer: RenderLayers,
    pub screen_layer: RenderLayers,
    pub feedback_scale: f32,
    pub feedback_alpha: f32,
    pub clear_color: Color,
}

impl FeedbackLoopSettings {
    pub const fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            zoom: 1.0,
            label: "feedback_loop_texture",
            paint_layer: RenderLayers::layer(0),
            screen_layer: RenderLayers::layer(1),
            feedback_scale: 0.985,
            feedback_alpha: 0.98,
            clear_color: Color::BLACK,
        }
    }

    pub const fn with_zoom(mut self, zoom: f32) -> Self {
        self.zoom = zoom;
        self
    }

    pub const fn with_label(mut self, label: &'static str) -> Self {
        self.label = label;
        self
    }

    pub fn with_layers(mut self, paint_layer: RenderLayers, screen_layer: RenderLayers) -> Self {
        self.paint_layer = paint_layer;
        self.screen_layer = screen_layer;
        self
    }

    pub const fn with_feedback(mut self, scale: f32, alpha: f32) -> Self {
        self.feedback_scale = scale;
        self.feedback_alpha = alpha;
        self
    }

    pub const fn with_clear_color(mut self, color: Color) -> Self {
        self.clear_color = color;
        self
    }
}

#[derive(Resource)]
struct FeedbackLoopState {
    textures: [Handle<Image>; 2],
    target_index: usize,
    paint_camera: Entity,
    feedback_sprite: Entity,
    screen_sprite: Entity,
}

#[derive(Component)]
struct FeedbackLoopPaintCamera;

#[derive(Component)]
pub struct FeedbackLoopEffect;

pub fn spawn_feedback_loop(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    settings: FeedbackLoopSettings,
) -> FeedbackLoopCanvas {
    let texture_a = images.add(feedback_texture(
        settings.width,
        settings.height,
        settings.label,
    ));
    let texture_b = images.add(feedback_texture(
        settings.width,
        settings.height,
        settings.label,
    ));
    let textures = [texture_a, texture_b];
    let target_index = 1;

    let paint_camera = commands
        .spawn((
            Camera2d,
            Camera {
                order: -1,
                clear_color: ClearColorConfig::Custom(settings.clear_color),
                ..default()
            },
            Projection::Orthographic(OrthographicProjection {
                scale: settings.zoom,
                ..OrthographicProjection::default_2d()
            }),
            RenderTarget::Image(textures[target_index].clone().into()),
            Msaa::Off,
            settings.paint_layer.clone(),
            FeedbackLoopPaintCamera,
        ))
        .id();

    let paint_view_size = Vec2::new(
        settings.width as f32 * settings.zoom,
        settings.height as f32 * settings.zoom,
    );

    let feedback_sprite = commands
        .spawn((
            Sprite {
                image: textures[0].clone(),
                custom_size: Some(paint_view_size),
                color: Color::srgba(1.0, 1.0, 1.0, settings.feedback_alpha),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, -100.0).with_scale(Vec3::splat(settings.feedback_scale)),
            settings.paint_layer.clone(),
            FeedbackLoopEffect,
        ))
        .id();

    let screen_sprite = commands
        .spawn((
            Sprite {
                image: textures[target_index].clone(),
                custom_size: Some(Vec2::new(settings.width as f32, settings.height as f32)),
                ..default()
            },
            settings.screen_layer.clone(),
        ))
        .id();

    commands.spawn((Camera2d, Msaa::Off, settings.screen_layer.clone()));
    commands.insert_resource(FeedbackLoopState {
        textures: textures.clone(),
        target_index,
        paint_camera,
        feedback_sprite,
        screen_sprite,
    });
    #[cfg(not(target_arch = "wasm32"))]
    commands.insert_resource(RecordingSource::Image(textures[target_index].clone()));

    FeedbackLoopCanvas {
        paint_layer: settings.paint_layer.clone(),
        screen_layer: settings.screen_layer.clone(),
        textures,
    }
}

fn advance_feedback_loop(
    state: Option<ResMut<FeedbackLoopState>>,
    #[cfg(not(target_arch = "wasm32"))] recording_source: Option<ResMut<RecordingSource>>,
    mut targets: Query<&mut RenderTarget, With<FeedbackLoopPaintCamera>>,
    mut sprites: Query<&mut Sprite>,
) {
    let Some(mut state) = state else {
        return;
    };

    let source_index = state.target_index;
    let target_index = 1 - source_index;
    state.target_index = target_index;

    if let Ok(mut target) = targets.get_mut(state.paint_camera) {
        *target = RenderTarget::Image(state.textures[target_index].clone().into());
    }
    if let Ok(mut sprite) = sprites.get_mut(state.feedback_sprite) {
        sprite.image = state.textures[source_index].clone();
    }
    if let Ok(mut sprite) = sprites.get_mut(state.screen_sprite) {
        sprite.image = state.textures[target_index].clone();
    }
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(mut recording_source) = recording_source {
        *recording_source = RecordingSource::Image(state.textures[target_index].clone());
    }
}

fn feedback_texture(width: u32, height: u32, label: &'static str) -> Image {
    let size = Extent3d {
        width,
        height,
        ..default()
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some(label),
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
