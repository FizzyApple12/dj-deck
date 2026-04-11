use symphonia::core::{
    codecs::{CODEC_TYPE_NULL, DecoderOptions},
    formats::FormatOptions,
    io::{MediaSourceStream, MediaSourceStreamOptions},
    meta::MetadataOptions,
    probe::Hint,
};
use symphonia_core::{audio::AudioBufferRef, conv::FromSample};
use thiserror::Error;

use crate::{AUDIO_CHANNELS, types::library::Track};

#[derive(Error, Debug)]
pub enum TrackLoadError {
    #[error("File Not Found: {0}")]
    FileNotFound(std::io::Error),

    #[error("Decode Error: {0}")]
    DecodeError(#[from] symphonia::core::errors::Error),

    #[error("No Valid Codecs")]
    NoValidCodecs,
}

pub struct TrackAudioData {
    pub samples: [Vec<f32>; AUDIO_CHANNELS],

    pub first_sample_time: f32,
    pub sample_rate: u32,
}

impl TrackAudioData {
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::manual_let_else, clippy::while_let_loop)]
    pub fn load_from_file(track: &Track) -> Result<TrackAudioData, TrackLoadError> {
        let source_file =
            std::fs::File::open(track.audio_path.clone()).map_err(TrackLoadError::FileNotFound)?;

        let media_source_stream =
            MediaSourceStream::new(Box::new(source_file), MediaSourceStreamOptions::default());

        let mut format_hint = Hint::new();
        if let Some(extension) = track.audio_path.extension()
            && let Some(extension) = extension.to_str()
        {
            format_hint.with_extension(extension);
        }

        let metadata_options: MetadataOptions = MetadataOptions::default();
        let format_options: FormatOptions = FormatOptions::default();

        let probed_formats = symphonia::default::get_probe().format(
            &format_hint,
            media_source_stream,
            &format_options,
            &metadata_options,
        )?;

        let mut format = probed_formats.format;

        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or(TrackLoadError::NoValidCodecs)?;

        let decoder_options: DecoderOptions = DecoderOptions::default();

        let mut decoder =
            symphonia::default::get_codecs().make(&track.codec_params, &decoder_options)?;

        let track_id = track.id;

        let mut new_track_audio_data = TrackAudioData {
            samples: Default::default(),

            // todo: track.codec_params.start_ts as f64,
            first_sample_time: 0.0,
            sample_rate: track.codec_params.sample_rate.unwrap_or(48000),
        };

        let mut sine_time: f32 = 0.0;

        loop {
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(_) => break,
            };

            while !format.metadata().is_latest() {
                format.metadata().pop();
            }

            if packet.track_id() != track_id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(audio_buffer) => match audio_buffer {
                    AudioBufferRef::U8(buffer) => {
                        for (sample_plane_number, sample_plane) in
                            buffer.planes().planes().iter().enumerate()
                        {
                            if let Some(destination_plane) =
                                new_track_audio_data.samples.get_mut(sample_plane_number)
                            {
                                for sample in *sample_plane {
                                    // destination_plane.push(f32::from_sample(*
                                    // sample));
                                    sine_time += 0.01;
                                    destination_plane.push(sine_time.sin() * 0.25);
                                }
                            }
                        }
                    }
                    AudioBufferRef::U16(buffer) => {
                        for (sample_plane_number, sample_plane) in
                            buffer.planes().planes().iter().enumerate()
                        {
                            if let Some(destination_plane) =
                                new_track_audio_data.samples.get_mut(sample_plane_number)
                            {
                                for sample in *sample_plane {
                                    // destination_plane.push(f32::from_sample(*
                                    // sample));
                                    sine_time += 0.01;
                                    destination_plane.push(sine_time.sin() * 0.25);
                                }
                            }
                        }
                    }
                    AudioBufferRef::U24(buffer) => {
                        for (sample_plane_number, sample_plane) in
                            buffer.planes().planes().iter().enumerate()
                        {
                            if let Some(destination_plane) =
                                new_track_audio_data.samples.get_mut(sample_plane_number)
                            {
                                for sample in *sample_plane {
                                    // destination_plane.push(f32::from_sample(*
                                    // sample));
                                    sine_time += 0.01;
                                    destination_plane.push(sine_time.sin() * 0.25);
                                }
                            }
                        }
                    }
                    AudioBufferRef::U32(buffer) => {
                        for (sample_plane_number, sample_plane) in
                            buffer.planes().planes().iter().enumerate()
                        {
                            if let Some(destination_plane) =
                                new_track_audio_data.samples.get_mut(sample_plane_number)
                            {
                                for sample in *sample_plane {
                                    // destination_plane.push(f32::from_sample(*
                                    // sample));
                                    sine_time += 0.01;
                                    destination_plane.push(sine_time.sin() * 0.25);
                                }
                            }
                        }
                    }
                    AudioBufferRef::S8(buffer) => {
                        for (sample_plane_number, sample_plane) in
                            buffer.planes().planes().iter().enumerate()
                        {
                            if let Some(destination_plane) =
                                new_track_audio_data.samples.get_mut(sample_plane_number)
                            {
                                for sample in *sample_plane {
                                    // destination_plane.push(f32::from_sample(*
                                    // sample));
                                    sine_time += 0.01;
                                    destination_plane.push(sine_time.sin() * 0.25);
                                }
                            }
                        }
                    }
                    AudioBufferRef::S16(buffer) => {
                        for (sample_plane_number, sample_plane) in
                            buffer.planes().planes().iter().enumerate()
                        {
                            if let Some(destination_plane) =
                                new_track_audio_data.samples.get_mut(sample_plane_number)
                            {
                                for sample in *sample_plane {
                                    // destination_plane.push(f32::from_sample(*
                                    // sample));
                                    sine_time += 0.01;
                                    destination_plane.push(sine_time.sin() * 0.25);
                                }
                            }
                        }
                    }
                    AudioBufferRef::S24(buffer) => {
                        for (sample_plane_number, sample_plane) in
                            buffer.planes().planes().iter().enumerate()
                        {
                            if let Some(destination_plane) =
                                new_track_audio_data.samples.get_mut(sample_plane_number)
                            {
                                for sample in *sample_plane {
                                    // destination_plane.push(f32::from_sample(*
                                    // sample));
                                    sine_time += 0.01;
                                    destination_plane.push(sine_time.sin() * 0.25);
                                }
                            }
                        }
                    }
                    AudioBufferRef::S32(buffer) => {
                        for (sample_plane_number, sample_plane) in
                            buffer.planes().planes().iter().enumerate()
                        {
                            if let Some(destination_plane) =
                                new_track_audio_data.samples.get_mut(sample_plane_number)
                            {
                                for sample in *sample_plane {
                                    // destination_plane.push(f32::from_sample(*
                                    // sample));
                                    sine_time += 0.01;
                                    destination_plane.push(sine_time.sin() * 0.25);
                                }
                            }
                        }
                    }
                    AudioBufferRef::F32(buffer) => {
                        for (sample_plane_number, sample_plane) in
                            buffer.planes().planes().iter().enumerate()
                        {
                            if let Some(destination_plane) =
                                new_track_audio_data.samples.get_mut(sample_plane_number)
                            {
                                for sample in *sample_plane {
                                    // destination_plane.push(f32::from_sample(*
                                    // sample));
                                    sine_time += 0.01;
                                    destination_plane.push(sine_time.sin() * 0.25);
                                }
                            }
                        }
                    }
                    AudioBufferRef::F64(buffer) => {
                        for (sample_plane_number, sample_plane) in
                            buffer.planes().planes().iter().enumerate()
                        {
                            if let Some(destination_plane) =
                                new_track_audio_data.samples.get_mut(sample_plane_number)
                            {
                                for sample in *sample_plane {
                                    // destination_plane.push(f32::from_sample(*
                                    // sample));
                                    sine_time += 0.01;
                                    destination_plane.push(sine_time.sin() * 0.25);
                                }
                            }
                        }
                    }
                },
                Err(_) => break,
            }
        }

        Ok(new_track_audio_data)
    }

    // todo: allow bidirectional reading
    #[allow(
        clippy::indexing_slicing,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        clippy::cast_possible_wrap,
        clippy::needless_range_loop
    )]
    pub fn read_samples(
        &self,
        start_time_seconds: f32,
        samples_needed: usize,
        output_buffers: &mut [Vec<f32>; AUDIO_CHANNELS],
    ) {
        let start_sample: i64 = ((start_time_seconds - self.first_sample_time)
            * self.sample_rate as f32)
            .round() as i64;

        let end_sample: i64 = start_sample + samples_needed as i64;

        for (channel_index, channel_samples) in self.samples.iter().enumerate() {
            let channel_sample_count = channel_samples.len();

            if end_sample <= 0
                || start_sample >= channel_sample_count as i64
                || channel_samples.is_empty()
            {
                if channel_sample_count < samples_needed {
                    output_buffers[channel_index].resize(samples_needed, 0.0);
                }

                for sample_index in 0..samples_needed {
                    output_buffers[channel_index][sample_index] = 0.0;
                }

                return;
            }

            let leading_zeros = if start_sample < 0 {
                (-start_sample) as usize
            } else {
                0
            };

            let trailing_zeros = if end_sample >= channel_sample_count as i64 {
                (end_sample - channel_sample_count as i64) as usize
            } else {
                0
            };

            let valid_start = leading_zeros;
            let valid_end = samples_needed - trailing_zeros;

            let source_frame_offset = if start_sample < 0 {
                0
            } else {
                start_sample as usize
            };

            if channel_sample_count < samples_needed {
                output_buffers[channel_index].resize(samples_needed, 0.0);
            }

            for sample_index in 0..valid_start {
                output_buffers[channel_index][sample_index] = 0.0;
            }

            for sample_index in valid_start..valid_end {
                let source_frame_index = source_frame_offset + (sample_index - valid_start);

                output_buffers[channel_index][sample_index] =
                    self.samples[channel_index][source_frame_index];
            }

            for sample_index in valid_end..samples_needed {
                output_buffers[channel_index][sample_index] = 0.0;
            }
        }
    }
}
