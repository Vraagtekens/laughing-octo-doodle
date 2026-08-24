use bevy::{
    app::PluginGroupBuilder,
    asset::{AssetMetaCheck, AssetPlugin},
    audio::AudioPlugin,
    prelude::*,
};

pub const WEB_CANVAS_SELECTOR: &str = "#bevy-canvas";

pub fn sketch_plugins(
    title: impl Into<String>,
    width: u32,
    height: u32,
    asset_path: impl Into<String>,
) -> PluginGroupBuilder {
    let plugins = DefaultPlugins
        .set(AssetPlugin {
            file_path: asset_path.into(),
            meta_check: AssetMetaCheck::Never,
            ..default()
        })
        .set(WindowPlugin {
            primary_window: Some(sketch_window(title, width, height)),
            ..default()
        });

    if cfg!(target_arch = "wasm32") {
        plugins.disable::<AudioPlugin>()
    } else {
        plugins
    }
}

pub fn sketch_window(title: impl Into<String>, width: u32, height: u32) -> Window {
    let mut window = Window {
        title: title.into(),
        resolution: (width, height).into(),
        ..default()
    };

    if cfg!(target_arch = "wasm32") {
        window.canvas = Some(WEB_CANVAS_SELECTOR.into());
        window.fit_canvas_to_parent = true;
        window.prevent_default_event_handling = false;
    }

    window
}

pub fn local_asset_path(manifest_dir: &str) -> String {
    if cfg!(target_arch = "wasm32") {
        "assets".into()
    } else {
        format!("{manifest_dir}/assets")
    }
}

pub fn workspace_asset_path(manifest_dir: &str) -> String {
    if cfg!(target_arch = "wasm32") {
        "assets".into()
    } else {
        format!("{manifest_dir}/../../assets")
    }
}
