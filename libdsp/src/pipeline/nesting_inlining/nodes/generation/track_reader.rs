use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use timecode::Timecode;

use crate::{
    audio_loader::TrackAudioData,
    nanoseconds_to_samples,
    pipeline::nesting_inlining::{DataExtractor, PipelineNode},
};

#[derive(Clone, Copy)]
pub struct TrackReaderSourceTimecode {
    pub start_time: Timecode,
    pub end_time: Timecode,

    pub wrap_times: Option<(Timecode, Timecode)>,
}

pub struct TrackReader<const CHANNELS: usize, Data, SourceTimecodeDataExtractor>
where
    SourceTimecodeDataExtractor: DataExtractor<Data, Option<TrackReaderSourceTimecode>>,
{
    phantom_data: PhantomData<Data>,
    source_timecode_extractor: SourceTimecodeDataExtractor,

    track_audio_data_ref: Rc<RefCell<Option<TrackAudioData>>>,
    source_timecode: Option<TrackReaderSourceTimecode>,

    sample_accumulator: i64,

    wrap_source: Option<i64>,
    wrap_target: i64,

    has_wrapped: bool,
}

impl<const CHANNELS: usize, Data, SourceTimecodeDataExtractor>
    TrackReader<CHANNELS, Data, SourceTimecodeDataExtractor>
where
    SourceTimecodeDataExtractor: DataExtractor<Data, Option<TrackReaderSourceTimecode>>,
{
    pub fn new(
        track_audio_data_ref: Rc<RefCell<Option<TrackAudioData>>>,
        source_timecode_extractor: SourceTimecodeDataExtractor,
    ) -> Self {
        Self {
            phantom_data: PhantomData,
            source_timecode_extractor,

            track_audio_data_ref,
            source_timecode: None,

            sample_accumulator: 0,

            wrap_source: None,
            wrap_target: 0,

            has_wrapped: false,
        }
    }
}

impl<const CHANNELS: usize, Data, SourceTimecodeDataExtractor> PipelineNode<CHANNELS, Data>
    for TrackReader<CHANNELS, Data, SourceTimecodeDataExtractor>
where
    SourceTimecodeDataExtractor: DataExtractor<Data, Option<TrackReaderSourceTimecode>>,
{
    fn update(&mut self, data: &Data) {
        self.source_timecode = (self.source_timecode_extractor)(data);

        let Some(source_timecode) = self.source_timecode else {
            self.sample_accumulator = 0;

            self.wrap_source = None;
            self.wrap_target = 0;

            return;
        };

        let track_data = self.track_audio_data_ref.borrow();
        let Some(track_data) = track_data.as_ref() else {
            return;
        };

        let sample_rate = track_data.sample_rate;

        self.sample_accumulator =
            nanoseconds_to_samples(source_timecode.start_time.nanoseconds, sample_rate);

        if let Some(wrap_times) = source_timecode.wrap_times {
            self.wrap_source = Some(nanoseconds_to_samples(
                wrap_times.0.nanoseconds,
                sample_rate,
            ));
            self.wrap_target = nanoseconds_to_samples(wrap_times.1.nanoseconds, sample_rate);
        } else {
            self.wrap_source = None;
            self.wrap_target = 0;
        }

        self.has_wrapped = false;
    }

    #[allow(
        clippy::inline_always,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    #[inline(always)]
    fn next(&mut self, _: usize) -> [f32; CHANNELS] {
        if self.source_timecode.is_none() {
            return [0.0; CHANNELS];
        }

        let track_data = self.track_audio_data_ref.borrow();
        let Some(track_data) = track_data.as_ref() else {
            return [0.0; CHANNELS];
        };

        let mut returned_sample = [0.0; CHANNELS];

        if track_data.samples.len() < CHANNELS {
            for output in &mut returned_sample {
                *output = if let Some(channel) = track_data.samples.first()
                    && self.sample_accumulator >= 0
                    && self.sample_accumulator <= channel.len() as i64
                    && let Some(sample) = channel.get(self.sample_accumulator as usize)
                {
                    *sample
                } else {
                    0.0
                };
            }
        } else {
            for (output, channel) in returned_sample.iter_mut().zip(&track_data.samples) {
                *output = if self.sample_accumulator >= 0
                    && self.sample_accumulator <= channel.len() as i64
                    && let Some(sample) = channel.get(self.sample_accumulator as usize)
                {
                    *sample
                } else {
                    0.0
                };
            }
        }

        self.sample_accumulator += 1;

        if let Some(wrap_start) = self.wrap_source
            && self.sample_accumulator == wrap_start
            && !self.has_wrapped
        {
            self.sample_accumulator = self.wrap_target;
            self.has_wrapped = true;
        }

        returned_sample
    }

    fn reset(&mut self) {
        self.source_timecode = None;

        self.sample_accumulator = 0;

        self.wrap_source = None;
        self.wrap_target = 0;
    }
}
