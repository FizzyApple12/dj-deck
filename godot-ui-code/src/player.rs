use godot::{
    classes::{
        CanvasItem, Control, IPanelContainer, Label, PanelContainer, ShaderMaterial, TextureRect,
    },
    prelude::*,
};
use libdj::types::deck::TempoRange;
use libdsp::timecode::Timecode;

use crate::{ipc::IPC, preview_waveform_to_shader_texture};

#[derive(GodotClass)]
#[class(base=PanelContainer)]
pub struct PlayerContainer {
    base: Base<PanelContainer>,

    #[export]
    player_number: i32,

    #[export]
    ipc: OnEditor<Gd<IPC>>,

    #[export]
    player_data: OnEditor<Gd<CanvasItem>>,

    #[export]
    not_loaded: OnEditor<Gd<CanvasItem>>,

    #[export]
    loading: OnEditor<Gd<CanvasItem>>,

    #[export]
    player_number_label: OnEditor<Gd<Label>>,

    #[export]
    album_art: OnEditor<Gd<TextureRect>>,

    #[export]
    track_detail_title: OnEditor<Gd<Label>>,
    #[export]
    track_detail_time: OnEditor<Gd<Label>>,
    #[export]
    track_detail_bpm: OnEditor<Gd<Label>>,
    #[export]
    track_detail_key: OnEditor<Gd<Label>>,

    #[export]
    time_remain_label: OnEditor<Gd<Label>>,
    #[export]
    time_time_label: OnEditor<Gd<Label>>,
    #[export]
    track_time: OnEditor<Gd<Label>>,
    #[export]
    track_time_fractional: OnEditor<Gd<Label>>,

    #[export]
    tempo_sign: OnEditor<Gd<Label>>,
    #[export]
    tempo_percent: OnEditor<Gd<Label>>,
    #[export]
    tempo_percent_fractional: OnEditor<Gd<Label>>,
    #[export]
    tempo_range: OnEditor<Gd<Label>>,

    #[export]
    bpm: OnEditor<Gd<Label>>,
    #[export]
    bpm_fractional: OnEditor<Gd<Label>>,
    #[export]
    bpm_master: OnEditor<Gd<CanvasItem>>,

    #[export]
    key: OnEditor<Gd<Label>>,
    #[export]
    master_tempo: OnEditor<Gd<CanvasItem>>,

    #[export]
    waveform_container: OnEditor<Gd<Control>>,
    #[export]
    waveform: OnEditor<Gd<Control>>,
    #[export]
    playhead: OnEditor<Gd<Control>>,

    display_remain: bool,
}

impl PlayerContainer {}

#[godot_api]
impl IPanelContainer for PlayerContainer {
    fn init(base: Base<PanelContainer>) -> Self {
        Self {
            base,

            player_number: 0,

            ipc: OnEditor::default(),

            player_data: OnEditor::default(),

            not_loaded: OnEditor::default(),

            loading: OnEditor::default(),

            player_number_label: OnEditor::default(),

            album_art: OnEditor::default(),

            track_detail_title: OnEditor::default(),
            track_detail_time: OnEditor::default(),
            track_detail_bpm: OnEditor::default(),
            track_detail_key: OnEditor::default(),

            time_remain_label: OnEditor::default(),
            time_time_label: OnEditor::default(),
            track_time: OnEditor::default(),
            track_time_fractional: OnEditor::default(),

            tempo_sign: OnEditor::default(),
            tempo_percent: OnEditor::default(),
            tempo_percent_fractional: OnEditor::default(),
            tempo_range: OnEditor::default(),

            bpm: OnEditor::default(),
            bpm_fractional: OnEditor::default(),
            bpm_master: OnEditor::default(),

            key: OnEditor::default(),
            master_tempo: OnEditor::default(),

            waveform_container: OnEditor::default(),
            waveform: OnEditor::default(),
            playhead: OnEditor::default(),

            display_remain: true,
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
        if let Some(updated) = ipc
            .preview_waveform_updated
            .get(self.player_number as usize)
            && *updated
        {
            self.waveform_container.set_modulate(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.0,
            });

            if let Some(waveform) = &ipc.preview_waveforms.get(self.player_number as usize)
                && let Some(texture) = preview_waveform_to_shader_texture(waveform)
            {
                let mut material = self
                    .waveform
                    .get_material()
                    .unwrap()
                    .cast::<ShaderMaterial>();
                material.set_shader_parameter("WAVEFORM", &Variant::from(texture));

                self.waveform_container.set_modulate(Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                });
            }
        }

        if channel_data.player.is_loading {
            self.player_data.set_modulate(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.0,
            });
            self.not_loaded.set_modulate(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.0,
            });
            self.loading.set_modulate(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            });

            return;
        }

        let Some((device, track)) = &channel_data.player.current_track else {
            self.player_data.set_modulate(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.0,
            });
            self.not_loaded.set_modulate(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            });
            self.loading.set_modulate(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.0,
            });

            return;
        };

        let Some((_device_name, Some(device_library))) = &ipc.devices.get(device) else {
            self.player_data.set_modulate(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.0,
            });
            self.not_loaded.set_modulate(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            });
            self.loading.set_modulate(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.0,
            });

            return;
        };

        self.player_data.set_modulate(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        });
        self.not_loaded.set_modulate(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.0,
        });
        self.loading.set_modulate(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.0,
        });

        self.track_detail_title.set_text(&track.title);
        self.track_detail_time.set_text(&format!(
            "{:.0}:{:02.0}",
            track.duration / 60,
            track.duration % 60,
        ));
        self.track_detail_bpm.set_text(&format!("{:.1}", track.bpm));
        self.track_detail_key.set_text(
            device_library
                .keys
                .get(&track.key_id)
                .map_or(&String::new(), |key| &key.name),
        );

        self.time_remain_label.set_modulate(if self.display_remain {
            Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }
        } else {
            Color {
                r: 0.31,
                g: 0.31,
                b: 0.31,
                a: 1.0,
            }
        });
        self.time_time_label.set_modulate(if self.display_remain {
            Color {
                r: 0.31,
                g: 0.31,
                b: 0.31,
                a: 1.0,
            }
        } else {
            Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }
        });
        let total_time = if let Some(ref track_analysis) =
            channel_data.player.current_track_analysis
            && let Some(last_beat) = track_analysis.beat_grid.last()
        {
            #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
            let progress = (channel_data.player.time.nanoseconds as f64
                / last_beat.time.nanoseconds as f64) as f32;

            let mut material = self
                .waveform
                .get_material()
                .unwrap()
                .cast::<ShaderMaterial>();
            material.set_shader_parameter("PROGRESS", &Variant::from(progress));

            let playhead_progress =
                (self.waveform.get_size().x * progress).clamp(0.0, self.waveform.get_size().x);

            self.playhead.set_position(Vector2 {
                x: playhead_progress,
                y: 0.0,
            });

            last_beat.time
        } else {
            let mut material = self
                .waveform
                .get_material()
                .unwrap()
                .cast::<ShaderMaterial>();
            material.set_shader_parameter("PROGRESS", &Variant::from(0.0));

            self.playhead.set_position(Vector2 { x: 0.0, y: 0.0 });

            Timecode::from_seconds(i64::from(track.duration))
        };

        self.track_time.set_text(&if self.display_remain {
            let track_remaining_seconds =
                (total_time - channel_data.player.time).to_nanoseconds() / 1_000_000_000;

            format!(
                "{:.0}:{:02.0}",
                track_remaining_seconds / 60,
                (track_remaining_seconds % 60).abs(),
            )
        } else {
            let track_time_seconds = channel_data.player.time.to_nanoseconds() / 1_000_000_000;

            format!(
                "{:.0}:{:02.0}",
                track_time_seconds / 60,
                (track_time_seconds % 60).abs(),
            )
        });
        self.track_time_fractional
            .set_text(&if self.display_remain {
                let track_remaining_milliseconds =
                    (total_time - channel_data.player.time).to_nanoseconds() / 1_000_000;

                format!(".{:03.0}", (track_remaining_milliseconds % 1000).abs())
            } else {
                let track_time_milliseconds = channel_data.player.time.to_nanoseconds() / 1_000_000;

                format!(".{:03.0}", (track_time_milliseconds % 1000).abs())
            });

        self.tempo_sign.set_text(
            if (channel_data.player.tempo_percent - 1.0) < -f32::EPSILON {
                "-"
            } else {
                "+"
            },
        );
        self.tempo_percent.set_text(&format!(
            "{:.0}",
            (channel_data.player.tempo_percent - 1.0).abs().floor() * 100.0
        ));
        self.tempo_percent_fractional.set_text(&format!(
            ".{:02.0}",
            ((channel_data.player.tempo_percent - 1.0).fract() * 100.0)
                .abs()
                .floor()
        ));
        self.tempo_range
            .set_text(match channel_data.player.tempo_range {
                TempoRange::SixPercent => "± 6",
                TempoRange::TenPercent => "± 10",
                TempoRange::SixteenPercent => "± 16",
                TempoRange::OneHundredPercent => "WIDE",
            });

        let current_bpm = channel_data.player.get_current_bpm().unwrap_or(0.0);

        self.bpm.set_text(&format!("{:.0}", current_bpm.floor()));
        self.bpm_fractional
            .set_text(&format!(".{:01.0}", (current_bpm.fract() * 10.0).floor()));
        #[allow(clippy::cast_sign_loss)]
        self.bpm_master.set_modulate(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: if let Some(master) = ipc.deck_state.master_channel
                && master == self.player_number as usize
            {
                1.0
            } else {
                0.0
            },
        });

        self.key.set_text(
            device_library
                .keys
                .get(&track.key_id)
                .map_or(&String::new(), |key| &key.name),
        );

        self.master_tempo.set_modulate(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: if channel_data.player.master_tempo {
                1.0
            } else {
                0.0
            },
        });
    }
}

impl Drop for PlayerContainer {
    fn drop(&mut self) {}
}
