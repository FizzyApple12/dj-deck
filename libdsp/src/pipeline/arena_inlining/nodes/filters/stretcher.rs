use std::marker::PhantomData;

use crate::pipeline::arena_inlining::{DataExtractor, PipelineNode};

pub struct Stretcher<
    const CHANNELS: usize,
    Data,
    SourceSampleRateDataExtractor,
    DestinationSampleRateDataExtractor,
    PitchModificationDataExtractor,
> where
    SourceSampleRateDataExtractor: DataExtractor<Data, u32>,
    DestinationSampleRateDataExtractor: DataExtractor<Data, u32>,
    PitchModificationDataExtractor: DataExtractor<Data, f32>,
{
    phantom_data: PhantomData<Data>,
    source_sample_rate_data_extractor: SourceSampleRateDataExtractor,
    destination_sample_rate_data_extractor: DestinationSampleRateDataExtractor,
    pitch_modification_data_extractor: PitchModificationDataExtractor,
}

impl<
    const CHANNELS: usize,
    Data,
    SourceSampleRateDataExtractor,
    DestinationSampleRateDataExtractor,
    PitchModificationDataExtractor,
>
    Stretcher<
        CHANNELS,
        Data,
        SourceSampleRateDataExtractor,
        DestinationSampleRateDataExtractor,
        PitchModificationDataExtractor,
    >
where
    SourceSampleRateDataExtractor: DataExtractor<Data, u32>,
    DestinationSampleRateDataExtractor: DataExtractor<Data, u32>,
    PitchModificationDataExtractor: DataExtractor<Data, f32>,
{
    pub fn new(
        source_sample_rate_data_extractor: SourceSampleRateDataExtractor,
        destination_sample_rate_data_extractor: DestinationSampleRateDataExtractor,
        pitch_modification_data_extractor: PitchModificationDataExtractor,
    ) -> Self {
        Self {
            phantom_data: PhantomData,
            source_sample_rate_data_extractor,
            destination_sample_rate_data_extractor,
            pitch_modification_data_extractor,
        }
    }
}

impl<
    const CHANNELS: usize,
    Data,
    SourceSampleRateDataExtractor,
    DestinationSampleRateDataExtractor,
    PitchModificationDataExtractor,
> PipelineNode<CHANNELS, Data>
    for Stretcher<
        CHANNELS,
        Data,
        SourceSampleRateDataExtractor,
        DestinationSampleRateDataExtractor,
        PitchModificationDataExtractor,
    >
where
    SourceSampleRateDataExtractor: DataExtractor<Data, u32>,
    DestinationSampleRateDataExtractor: DataExtractor<Data, u32>,
    PitchModificationDataExtractor: DataExtractor<Data, f32>,
{
    fn update(&mut self, data: &Data) {}

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, input: [f32; CHANNELS], _: usize) -> [f32; CHANNELS] {
        input
    }

    fn reset(&mut self) {}
}
