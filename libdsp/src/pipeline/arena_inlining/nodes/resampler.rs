use std::marker::PhantomData;

use crate::pipeline::arena_inlining::{DataExtractor, PipelineNode};

pub struct Resampler<
    const CHANNELS: usize,
    Data,
    SourceSampleRateDataExtractor,
    DestinationSampleRateDataExtractor,
> where
    SourceSampleRateDataExtractor: DataExtractor<Data, u32>,
    DestinationSampleRateDataExtractor: DataExtractor<Data, u32>,
{
    phantom_data: PhantomData<Data>,
    source_sample_rate_data_extractor: SourceSampleRateDataExtractor,
    destination_sample_rate_data_extractor: DestinationSampleRateDataExtractor,
}

impl<const CHANNELS: usize, Data, SourceSampleRateDataExtractor, DestinationSampleRateDataExtractor>
    Resampler<CHANNELS, Data, SourceSampleRateDataExtractor, DestinationSampleRateDataExtractor>
where
    SourceSampleRateDataExtractor: DataExtractor<Data, u32>,
    DestinationSampleRateDataExtractor: DataExtractor<Data, u32>,
{
    pub fn new(
        source_sample_rate_data_extractor: SourceSampleRateDataExtractor,
        destination_sample_rate_data_extractor: DestinationSampleRateDataExtractor,
    ) -> Self {
        Self {
            phantom_data: PhantomData,
            source_sample_rate_data_extractor,
            destination_sample_rate_data_extractor,
        }
    }
}

impl<const CHANNELS: usize, Data, SourceSampleRateDataExtractor, DestinationSampleRateDataExtractor>
    PipelineNode<CHANNELS, Data>
    for Resampler<CHANNELS, Data, SourceSampleRateDataExtractor, DestinationSampleRateDataExtractor>
where
    SourceSampleRateDataExtractor: DataExtractor<Data, u32>,
    DestinationSampleRateDataExtractor: DataExtractor<Data, u32>,
{
    fn update(&mut self, data: &Data) {}

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, input: [f32; CHANNELS], _: usize) -> [f32; CHANNELS] {
        input
    }

    fn reset(&mut self) {}
}
