use std::ops::RangeInclusive;

#[cfg(not(target_arch = "wasm32"))]
use std::{
    fs,
    path::PathBuf,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[cfg(not(target_arch = "wasm32"))]
use bevy::render::view::screenshot::{Capturing, Screenshot, save_to_disk};
use bevy::{
    diagnostic::{Diagnostic, DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

pub struct SketchControlsPlugin {
    controls: SketchControls,
    target_fps: f32,
}

#[derive(Resource, Clone, Default)]
#[cfg(not(target_arch = "wasm32"))]
pub enum RecordingSource {
    #[default]
    PrimaryWindow,
    Image(Handle<Image>),
}

impl Default for SketchControlsPlugin {
    fn default() -> Self {
        Self::new("Sketch")
    }
}

impl SketchControlsPlugin {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            controls: SketchControls {
                title: title.into(),
                sliders: Vec::new(),
            },
            target_fps: 60.0,
        }
    }

    pub fn with_target_fps(mut self, target_fps: f32) -> Self {
        self.target_fps = target_fps;
        self
    }

    pub fn with_slider(
        mut self,
        key: &'static str,
        label: &'static str,
        value: f32,
        range: RangeInclusive<f32>,
    ) -> Self {
        self.controls.sliders.push(SliderControl {
            key,
            label,
            value,
            initial: value,
            range,
        });
        self
    }
}

impl Plugin for SketchControlsPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin::default());
        }
        if !app.is_plugin_added::<FrameTimeDiagnosticsPlugin>() {
            app.add_plugins(FrameTimeDiagnosticsPlugin::default());
        }

        app.insert_resource(self.controls.clone())
            .init_resource::<ControlPanelState>()
            .add_systems(Update, toggle_control_panel)
            .add_systems(EguiPrimaryContextPass, draw_control_panel);

        #[cfg(not(target_arch = "wasm32"))]
        app.insert_resource(FramePacer::new(self.target_fps))
            .init_resource::<FrameRecorder>()
            .init_resource::<RecordingSource>()
            .add_systems(Update, capture_recording_frames)
            .add_systems(Last, pace_frame);
    }
}

#[derive(Resource, Clone)]
pub struct SketchControls {
    title: String,
    sliders: Vec<SliderControl>,
}

impl SketchControls {
    pub fn value(&self, key: &str) -> f32 {
        self.sliders
            .iter()
            .find(|slider| slider.key == key)
            .map_or_else(
                || panic!("Unknown sketch control: {key}"),
                |slider| slider.value,
            )
    }
}

#[derive(Clone)]
struct SliderControl {
    key: &'static str,
    label: &'static str,
    value: f32,
    initial: f32,
    range: RangeInclusive<f32>,
}

#[derive(Resource)]
struct ControlPanelState {
    visible: bool,
}

impl Default for ControlPanelState {
    fn default() -> Self {
        Self { visible: false }
    }
}

#[derive(Resource)]
#[cfg(not(target_arch = "wasm32"))]
struct FrameRecorder {
    active: bool,
    frame: u64,
    frame_rate: f32,
    elapsed: f32,
    output_dir: Option<PathBuf>,
}

#[derive(Resource)]
#[cfg(not(target_arch = "wasm32"))]
struct FramePacer {
    target_fps: f32,
    previous_frame: Instant,
}

#[cfg(not(target_arch = "wasm32"))]
impl FramePacer {
    fn new(target_fps: f32) -> Self {
        Self {
            target_fps,
            previous_frame: Instant::now(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for FrameRecorder {
    fn default() -> Self {
        Self {
            active: false,
            frame: 0,
            frame_rate: 30.0,
            elapsed: 0.0,
            output_dir: None,
        }
    }
}

fn toggle_control_panel(keys: Res<ButtonInput<KeyCode>>, mut panel: ResMut<ControlPanelState>) {
    if keys.just_pressed(KeyCode::KeyH) {
        panel.visible = !panel.visible;
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn draw_control_panel(
    mut contexts: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
    mut controls: ResMut<SketchControls>,
    panel: Res<ControlPanelState>,
    mut recorder: ResMut<FrameRecorder>,
    mut frame_pacer: ResMut<FramePacer>,
) -> Result {
    if !panel.visible {
        return Ok(());
    }

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(Diagnostic::smoothed);

    egui::Window::new(controls.title.clone())
        .anchor(egui::Align2::LEFT_TOP, [12.0, 12.0])
        .resizable(false)
        .collapsible(false)
        .show(contexts.ctx_mut()?, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "FPS: {}",
                    fps.map_or("--".into(), |fps| format!("{fps:.0}"))
                ));
                if recorder.active {
                    ui.colored_label(egui::Color32::from_rgb(235, 70, 70), "RECORDING");
                }
            });

            ui.separator();
            ui.add(
                egui::Slider::new(&mut frame_pacer.target_fps, 10.0..=120.0)
                    .integer()
                    .text("app fps"),
            );
            ui.horizontal(|ui| {
                let label = if recorder.active { "Stop" } else { "Record" };
                if ui.button(label).clicked() {
                    if recorder.active {
                        recorder.active = false;
                    } else {
                        start_recording(&mut recorder);
                    }
                }
                ui.add(
                    egui::Slider::new(&mut recorder.frame_rate, 1.0..=60.0)
                        .integer()
                        .text("capture fps"),
                );
            });
            if let Some(path) = &recorder.output_dir {
                ui.small(path.display().to_string());
            }

            if !controls.sliders.is_empty() {
                ui.separator();
                for slider in &mut controls.sliders {
                    ui.add(
                        egui::Slider::new(&mut slider.value, slider.range.clone())
                            .text(slider.label),
                    );
                }
                if ui.button("Reset controls").clicked() {
                    for slider in &mut controls.sliders {
                        slider.value = slider.initial;
                    }
                }
            }
        });

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn draw_control_panel(
    mut contexts: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
    mut controls: ResMut<SketchControls>,
    panel: Res<ControlPanelState>,
) -> Result {
    if !panel.visible {
        return Ok(());
    }

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(Diagnostic::smoothed);

    egui::Window::new(controls.title.clone())
        .anchor(egui::Align2::LEFT_TOP, [12.0, 12.0])
        .resizable(false)
        .collapsible(false)
        .show(contexts.ctx_mut()?, |ui| {
            ui.label(format!(
                "FPS: {}",
                fps.map_or("--".into(), |fps| format!("{fps:.0}"))
            ));

            if !controls.sliders.is_empty() {
                ui.separator();
                for slider in &mut controls.sliders {
                    ui.add(
                        egui::Slider::new(&mut slider.value, slider.range.clone())
                            .text(slider.label),
                    );
                }
                if ui.button("Reset controls").clicked() {
                    for slider in &mut controls.sliders {
                        slider.value = slider.initial;
                    }
                }
            }
        });

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn pace_frame(mut frame_pacer: ResMut<FramePacer>) {
    let frame_duration = Duration::from_secs_f32(1.0 / frame_pacer.target_fps.max(1.0));
    if let Some(remaining) = frame_duration.checked_sub(frame_pacer.previous_frame.elapsed()) {
        thread::sleep(remaining);
    }
    frame_pacer.previous_frame = Instant::now();
}

#[cfg(not(target_arch = "wasm32"))]
fn start_recording(recorder: &mut FrameRecorder) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let output_dir = PathBuf::from("recordings").join(format!("recording-{timestamp}"));
    if let Err(error) = fs::create_dir_all(&output_dir) {
        error!("Could not create recording directory: {error}");
        return;
    }

    recorder.active = true;
    recorder.frame = 0;
    recorder.elapsed = 0.0;
    recorder.output_dir = Some(output_dir);
}

#[cfg(not(target_arch = "wasm32"))]
fn capture_recording_frames(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut recorder: ResMut<FrameRecorder>,
    source: Res<RecordingSource>,
    captures: Query<(), With<Capturing>>,
) {
    if !recorder.active {
        return;
    }

    recorder.elapsed += time.delta_secs();
    let interval = 1.0 / recorder.frame_rate;
    if recorder.elapsed < interval || !captures.is_empty() {
        return;
    }
    recorder.elapsed %= interval;

    let Some(output_dir) = recorder.output_dir.clone() else {
        return;
    };
    let path = output_dir.join(format!("frame-{:06}.png", recorder.frame));
    recorder.frame += 1;
    let screenshot = match &*source {
        RecordingSource::PrimaryWindow => Screenshot::primary_window(),
        RecordingSource::Image(image) => Screenshot::image(image.clone()),
    };
    commands.spawn(screenshot).observe(save_to_disk(path));
}
