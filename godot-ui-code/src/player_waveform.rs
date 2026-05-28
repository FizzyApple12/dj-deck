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
    waveform: OnEditor<Gd<Control>>,
    waveform_length_seconds: f64,
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

            waveform: OnEditor::default(),
            waveform_length_seconds: 1.0,
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
            if let Some(waveform) = &ipc.waveforms.get(self.player_number as usize)
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
            } else {
                self.waveform.set_modulate(Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.0,
                });
            }
        }

        if channel_data.player.is_loading {
            self.waveform.set_modulate(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.0,
            });

            return;
        }

        let Some(_) = &channel_data.player.current_track else {
            self.waveform.set_modulate(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.0,
            });

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
        let horizontal_size = (ipc.waveform_pixels_per_second
            * self.waveform_length_seconds
            * f64::from(1.0 + channel_data.player.tempo_percent))
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
    }
}

impl Drop for PlayerWaveformContainer {
    fn drop(&mut self) {}
}
