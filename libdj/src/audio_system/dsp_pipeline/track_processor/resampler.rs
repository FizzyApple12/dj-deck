use libdsp::dsp::delay::interpolators::{InterpolatorTrait, cubic::InterpolatorCubic};

use crate::AUDIO_CHANNELS;

pub struct Resampler {
    interpolator: InterpolatorCubic<f32>,
    history: Vec<[f32; 3]>,
}

impl Default for Resampler {
    fn default() -> Self {
        Self {
            interpolator: InterpolatorCubic::default(),
            history: vec![[0.0; 3]; AUDIO_CHANNELS],
        }
    }
}

impl Resampler {
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_wrap,
        clippy::cast_possible_truncation,
        clippy::indexing_slicing
    )]
    pub fn process(
        &mut self,
        input: &[Vec<f32>; AUDIO_CHANNELS],
        input_samples: usize,
        output: &mut [Vec<f32>; AUDIO_CHANNELS],
        output_samples: usize,
        do_mult: bool,
    ) {
        if output_samples == 0 {
            return;
        }

        if input_samples == 0 {
            for output_channel in output {
                for output_sample in &mut output_channel[0..output_samples] {
                    *output_sample = 0.0;
                }
            }

            return;
        }

        let ratio = input_samples as f32 / output_samples as f32;

        for (channel_number, (input_channel, output_channel)) in
            input.iter().zip(output.iter_mut()).enumerate()
        {
            let history = self.history[channel_number];

            let read = |index: isize| -> f32 {
                if index < 0 {
                    let h = 3 + index;

                    if h >= 0 { history[h as usize] } else { 0.0 }
                } else if index >= input_samples as isize {
                    input_channel[input_samples - 1]
                } else {
                    input_channel[index as usize]
                }
            };

            for (output_index, output_sample) in
                output_channel[0..output_samples].iter_mut().enumerate()
            {
                let position = output_index as f32 * ratio;
                let base_index = position.floor() as isize;
                let fract = position - base_index as f32;

                *output_sample = self.interpolator.fractional(
                    &[
                        read(base_index - 1),
                        read(base_index),
                        read(base_index + 1),
                        read(base_index + 2),
                    ],
                    fract,
                ) * if do_mult { 8000.0 } else { 1.0 };
            }

            self.history[channel_number] = [
                if input_samples >= 3 {
                    input_channel[input_samples - 3]
                } else {
                    history[2]
                },
                if input_samples >= 2 {
                    input_channel[input_samples - 2]
                } else {
                    input_channel[input_samples - 1]
                },
                input_channel[input_samples - 1],
            ];
        }
    }

    pub fn reset(&mut self) {
        for ch in &mut self.history {
            *ch = [0.0; 3];
        }
    }
}
