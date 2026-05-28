use symphonia::core::{
    codecs::{CODEC_TYPE_NULL, DecoderOptions},
    formats::FormatOptions,
    io::{MediaSourceStream, MediaSourceStreamOptions},
    meta::MetadataOptions,
    probe::Hint,
};
use symphonia_core::{audio::AudioBufferRef, conv::FromSample};
use thiserror::Error;

use crate::{
    AUDIO_CHANNELS,
    math::sampling::nanoseconds_to_samples,
    types::{library::Track, timecode::Timecode},
};

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
                                    destination_plane.push(f32::from_sample(*sample));
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
                                    destination_plane.push(f32::from_sample(*sample));
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
                                    destination_plane.push(f32::from_sample(*sample));
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
                                    destination_plane.push(f32::from_sample(*sample));
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
                                    destination_plane.push(f32::from_sample(*sample));
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
                                    destination_plane.push(f32::from_sample(*sample));
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
                                    destination_plane.push(f32::from_sample(*sample));
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
                                    destination_plane.push(f32::from_sample(*sample));
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
                                    destination_plane.push(f32::from_sample(*sample));
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
                                    destination_plane.push(f32::from_sample(*sample));
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

    #[allow(
        clippy::indexing_slicing,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_possible_wrap,
        clippy::needless_range_loop,
        clippy::too_many_lines
    )]
    pub fn read_samples(
        &self,
        start: Timecode,
        end: Timecode,
        wrap: Option<(Timecode, Timecode)>,
        output_buffers: &mut [Vec<f32>; AUDIO_CHANNELS],
    ) -> usize {
        match (start, end, wrap) {
            (start, end, None) => {
                let start = nanoseconds_to_samples(start.to_nanoseconds(), self.sample_rate);
                let end = nanoseconds_to_samples(end.to_nanoseconds(), self.sample_rate);

                let total_samples = if start < end {
                    (end - start) as usize
                } else {
                    (start - end) as usize
                };

                for buffer in output_buffers.iter_mut() {
                    if buffer.len() < total_samples {
                        buffer.resize(total_samples, 0.0);
                    }
                }

                if start < end {
                    for buffer_index in 0..AUDIO_CHANNELS {
                        let max_sample_index = self.samples[buffer_index].len() as i64;

                        for (target_index, source_index) in (start..end).enumerate() {
                            output_buffers[buffer_index][target_index] =
                                if source_index >= 0 && source_index < max_sample_index {
                                    self.samples[buffer_index][source_index as usize]
                                } else {
                                    0.0
                                };
                        }
                    }
                } else {
                    for buffer_index in 0..AUDIO_CHANNELS {
                        let max_sample_index = self.samples[buffer_index].len() as i64;

                        for (target_index, source_index) in (end..start).rev().enumerate() {
                            output_buffers[buffer_index][target_index] =
                                if source_index >= 0 && source_index < max_sample_index {
                                    self.samples[buffer_index][source_index as usize]
                                } else {
                                    0.0
                                };
                        }
                    }
                }

                total_samples
            }

            (start, end, Some((wrap_start, wrap_end))) => {
                let start = nanoseconds_to_samples(start.to_nanoseconds(), self.sample_rate);
                let wrap_start =
                    nanoseconds_to_samples(wrap_start.to_nanoseconds(), self.sample_rate);
                let wrap_end = nanoseconds_to_samples(wrap_end.to_nanoseconds(), self.sample_rate);
                let end = nanoseconds_to_samples(end.to_nanoseconds(), self.sample_rate);

                let total_prewrap_samples = if start < wrap_start {
                    (wrap_start - start) as usize
                } else {
                    (start - wrap_start) as usize
                };
                let total_postwrap_samples = if wrap_end < end {
                    (end - wrap_end) as usize
                } else {
                    (wrap_end - end) as usize
                };

                for buffer in output_buffers.iter_mut() {
                    if buffer.len() < total_prewrap_samples + total_postwrap_samples {
                        buffer.resize(total_prewrap_samples + total_postwrap_samples, 0.0);
                    }
                }

                if start < wrap_start {
                    for buffer_index in 0..AUDIO_CHANNELS {
                        let max_sample_index = self.samples[buffer_index].len() as i64;

                        for (target_index, source_index) in (start..wrap_start).enumerate() {
                            output_buffers[buffer_index][target_index] =
                                if source_index >= 0 && source_index < max_sample_index {
                                    self.samples[buffer_index][source_index as usize]
                                } else {
                                    0.0
                                };
                        }
                    }
                } else {
                    for buffer_index in 0..AUDIO_CHANNELS {
                        let max_sample_index = self.samples[buffer_index].len() as i64;

                        for (target_index, source_index) in (wrap_start..start).rev().enumerate() {
                            output_buffers[buffer_index][target_index] =
                                if source_index >= 0 && source_index < max_sample_index {
                                    self.samples[buffer_index][source_index as usize]
                                } else {
                                    0.0
                                };
                        }
                    }
                }

                if wrap_end < end {
                    for buffer_index in 0..AUDIO_CHANNELS {
                        let max_sample_index = self.samples[buffer_index].len() as i64;

                        for (target_index, source_index) in (wrap_end..end).enumerate() {
                            output_buffers[buffer_index][target_index + total_prewrap_samples] =
                                if source_index >= 0 && source_index < max_sample_index {
                                    self.samples[buffer_index][source_index as usize]
                                } else {
                                    0.0
                                };
                        }
                    }
                } else {
                    for buffer_index in 0..AUDIO_CHANNELS {
                        let max_sample_index = self.samples[buffer_index].len() as i64;

                        for (target_index, source_index) in (end..wrap_end).rev().enumerate() {
                            output_buffers[buffer_index][target_index + total_prewrap_samples] =
                                if source_index >= 0 && source_index < max_sample_index {
                                    self.samples[buffer_index][source_index as usize]
                                } else {
                                    0.0
                                };
                        }
                    }
                }

                total_prewrap_samples + total_postwrap_samples
            }
        }
    }
}
