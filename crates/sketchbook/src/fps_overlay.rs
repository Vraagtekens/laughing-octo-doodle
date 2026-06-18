use bevy::{
    diagnostic::{Diagnostic, DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

pub struct FpsOverlayPlugin;

impl Plugin for FpsOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Startup, spawn_fps_overlay)
            .add_systems(Update, (toggle_fps_overlay, update_fps_overlay).chain());
    }
}

#[derive(Component)]
pub struct FpsOverlay;

fn spawn_fps_overlay(mut commands: Commands) {
    commands.spawn((
        Text::new("FPS: --"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::WHITE),
        TextShadow::default(),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
        Visibility::Hidden,
        FpsOverlay,
    ));
}

fn toggle_fps_overlay(
    keys: Res<ButtonInput<KeyCode>>,
    mut visibility: Single<&mut Visibility, With<FpsOverlay>>,
) {
    if keys.just_pressed(KeyCode::KeyH) {
        **visibility = match **visibility {
            Visibility::Hidden => Visibility::Visible,
            _ => Visibility::Hidden,
        };
    }
}

fn update_fps_overlay(
    diagnostics: Res<DiagnosticsStore>,
    mut fps_text: Single<(&mut Text, &Visibility), With<FpsOverlay>>,
) {
    let (text, visibility) = &mut *fps_text;
    if **visibility == Visibility::Hidden {
        return;
    }

    if let Some(fps) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(Diagnostic::smoothed)
    {
        text.0 = format!("FPS: {fps:.0}");
    }
}
