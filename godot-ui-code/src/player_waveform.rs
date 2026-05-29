use godot::{
    classes::{BoxContainer, Control, IBoxContainer, Label, ShaderMaterial},
    prelude::*,
};

use crate::{ipc::IPC, waveform_to_shader_texture};

#[derive(GodotClass)]
#[class(base=BoxContainer)]
pub struct PlayerWaveformContainer {
    base: Base<BoxContainer>,

    #[export]
    player_number: i32,

    #[export]
    ipc: OnEditor<Gd<IPC>>,

    #[export]
    player_number_label: OnEditor<Gd<Label>>,

    #[export]
    waveform_container: OnEditor<Gd<Control>>,

    #[export]
    waveform: OnEditor<Gd<Control>>,
    waveform_length_seconds: f64,
    #[export]
    beat_marker: OnEditor<Gd<PackedScene>>,
    instantiated_beat_markers: Vec<Gd<Control>>,
}

impl PlayerWaveformContainer {}

#[godot_api]
impl IBoxContainer for PlayerWaveformContainer {
    fn init(base: Base<BoxContainer>) -> Self {
        Self {
            base,

            player_number: 0,

            ipc: OnEditor::default(),

            player_number_label: OnEditor::default(),

            waveform_container: OnEditor::default(),

            waveform: OnEditor::default(),
            waveform_length_seconds: 1.0,
            beat_marker: OnEditor::default(),
            instantiated_beat_markers: Vec::new(),
        }
    }

    #[allow(clippy::too_many_lines)]
    fn process(&mut self, _delta: f64) {
        let ipc = self.ipc.bind();

        #[allow(clippy::cast_sign_loss)]
        let Some(channel_data) = ipc
            .deck_state
            .mixer_channels
            .get(self.player_number as usize)
        else {
            return;
        };

        self.player_number_label
            .set_text(&format!("{}", self.player_number + 1));

        #[allow(clippy::cast_sign_loss)]
        if let Some(updated) = ipc.waveform_updated.get(self.player_number as usize)
            && *updated
        {
            self.waveform.set_modulate(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.0,
            });

            if let Some(waveform) = ipc.waveforms.get(self.player_number as usize)
                && let Some((texture, stride, waveform_length_seconds)) =
                    waveform_to_shader_texture(waveform)
            {
                let mut material = self
                    .waveform
                    .get_material()
                    .unwrap()
                    .cast::<ShaderMaterial>();
                material.set_shader_parameter("WAVEFORM", &Variant::from(texture));
                material.set_shader_parameter("STRIDE", &Variant::from(stride));

                self.waveform_length_seconds = f64::from(waveform_length_seconds);

                self.waveform.set_modulate(Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                });
            }
        }

        #[allow(clippy::cast_sign_loss)]
        let Some(track_analysis) = ipc.track_analyses.get(self.player_number as usize) else {
            return;
        };

        #[allow(clippy::cast_sign_loss)]
        if let Some(updated) = ipc.track_analysis_updated.get(self.player_number as usize)
            && *updated
        {
            if self.instantiated_beat_markers.len() < track_analysis.beat_grid.len() {
                while self.instantiated_beat_markers.len() < track_analysis.beat_grid.len() {
                    let beat_grid = self.beat_marker.instantiate_as::<Control>();

                    self.waveform_container.add_child(&beat_grid);

                    self.instantiated_beat_markers.push(beat_grid);
                }
            } else if self.instantiated_beat_markers.len() > track_analysis.beat_grid.len() {
                while self.instantiated_beat_markers.len() > track_analysis.beat_grid.len() {
                    if let Some(beat_grid) = self.instantiated_beat_markers.pop() {
                        beat_grid.free();
                    }
                }
            }
        }

        if channel_data.player.is_loading {
            self.waveform.set_modulate(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.0,
            });

            for beat_marker in &mut self.instantiated_beat_markers {
                beat_marker.set_modulate(Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.0,
                });
            }

            return;
        }

        let Some(_) = &channel_data.player.current_track else {
            self.waveform.set_modulate(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.0,
            });

            for beat_marker in &mut self.instantiated_beat_markers {
                beat_marker.set_modulate(Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.0,
                });
            }

            return;
        };

        self.waveform.set_modulate(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        });

        let size = self.waveform.get_size();
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
        let horizontal_size = ((ipc.waveform_pixels_per_second * self.waveform_length_seconds)
            / f64::from(channel_data.player.tempo_percent.max(0.1)))
            as f32;

        self.waveform.set_size(Vector2 {
            x: horizontal_size,
            y: size.y,
        });

        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
        let progress = ((channel_data.player.time.nanoseconds as f64 / 1_000_000_000.0)
            / self.waveform_length_seconds) as f32;

        let playhead_progress = horizontal_size * progress;

        self.waveform.set_position(Vector2 {
            x: -playhead_progress,
            y: 0.0,
        });

        let mut first_beat_accumulator = 0;

        for (beat_index, beat) in track_analysis.beat_grid.iter().enumerate() {
            if let Some(ref mut beat_marker) = self.instantiated_beat_markers.get_mut(beat_index) {
                if beat.beat_number == 0 {
                    first_beat_accumulator = (first_beat_accumulator + 1) % 4;
                }

                if beat.beat_number == 0 && first_beat_accumulator == 0 {
                    beat_marker.set_modulate(Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    });
                } else if ipc.waveform_pixels_per_second <= 8.0 {
                    beat_marker.set_modulate(Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 0.0,
                    });
                } else if beat.beat_number == 0 {
                    beat_marker.set_modulate(Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    });
                } else if ipc.waveform_pixels_per_second <= 40.0 {
                    beat_marker.set_modulate(Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 0.0,
                    });
                } else {
                    beat_marker.set_modulate(Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    });
                }

                #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
                let progress = (((channel_data.player.time.nanoseconds - beat.time.nanoseconds)
                    as f64
                    / 1_000_000_000.0)
                    / self.waveform_length_seconds) as f32;

                let beat_marker_progress = horizontal_size * progress;

                beat_marker.set_position(Vector2 {
                    x: -beat_marker_progress,
                    y: 0.0,
                });
            }
        }
    }
}

impl Drop for PlayerWaveformContainer {
    fn drop(&mut self) {}
}
