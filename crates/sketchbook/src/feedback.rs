use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
};

#[derive(Clone)]
pub struct FeedbackCanvas {
    pub texture: Handle<Image>,
    pub paint_layer: RenderLayers,
    pub screen_layer: RenderLayers,
}

#[derive(Clone)]
pub struct FeedbackSettings {
    pub width: u32,
    pub height: u32,
    pub zoom: f32,
    pub label: &'static str,
    pub paint_layer: RenderLayers,
    pub screen_layer: RenderLayers,
    pub clear_color: ClearColorConfig,
}

impl FeedbackSettings {
    pub const fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            zoom: 1.0,
            label: "feedback_texture",
            paint_layer: RenderLayers::layer(0),
            screen_layer: RenderLayers::layer(1),
            clear_color: ClearColorConfig::None,
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

    pub const fn with_clear_color(mut self, clear_color: ClearColorConfig) -> Self {
        self.clear_color = clear_color;
        self
    }
}

pub fn spawn_feedback_canvas(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    settings: FeedbackSettings,
) -> FeedbackCanvas {
    let texture = images.add(feedback_texture(
        settings.width,
        settings.height,
        settings.label,
    ));

    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            clear_color: settings.clear_color,
            ..default()
        },
        Projection::Orthographic(OrthographicProjection {
            scale: settings.zoom,
            ..OrthographicProjection::default_2d()
        }),
        RenderTarget::Image(texture.clone().into()),
        Msaa::Off,
        settings.paint_layer.clone(),
    ));

    commands.spawn((
        Sprite {
            image: texture.clone(),
            custom_size: Some(Vec2::new(settings.width as f32, settings.height as f32)),
            ..default()
        },
        settings.screen_layer.clone(),
    ));

    commands.spawn((Camera2d, Msaa::Off, settings.screen_layer.clone()));

    FeedbackCanvas {
        texture,
        paint_layer: settings.paint_layer.clone(),
        screen_layer: settings.screen_layer.clone(),
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
