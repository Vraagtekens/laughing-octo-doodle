use bevy::prelude::*;

use crate::sketch_controls::SketchControlsPlugin;

pub struct FpsOverlayPlugin;

impl Plugin for FpsOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SketchControlsPlugin::default());
    }
}
