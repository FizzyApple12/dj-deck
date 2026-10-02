use std::marker::PhantomData;

use crate::pipeline::nesting_inlining::{DataExtractor, PipelineNode};

pub struct Stretcher<
    const CHANNELS: usize,
    Data,
    Parent,
    SourceSampleRateDataExtractor,
    DestinationSampleRateDataExtractor,
    PitchModificationDataExtractor,
> where
    Parent: PipelineNode<CHANNELS, Data>,
    SourceSampleRateDataExtractor: DataExtractor<Data, u32>,
    DestinationSampleRateDataExtractor: DataExtractor<Data, u32>,
    PitchModificationDataExtractor: DataExtractor<Data, f32>,
{
    phantom_data: PhantomData<Data>,
    parent: Parent,
    source_sample_rate_data_extractor: SourceSampleRateDataExtractor,
    destination_sample_rate_data_extractor: DestinationSampleRateDataExtractor,
    pitch_modification_data_extractor: PitchModificationDataExtractor,
}

impl<
    const CHANNELS: usize,
    Data,
    Parent,
    SourceSampleRateDataExtractor,
    DestinationSampleRateDataExtractor,
    PitchModificationDataExtractor,
>
    Stretcher<
        CHANNELS,
        Data,
        Parent,
        SourceSampleRateDataExtractor,
        DestinationSampleRateDataExtractor,
        PitchModificationDataExtractor,
    >
where
    Parent: PipelineNode<CHANNELS, Data>,
    SourceSampleRateDataExtractor: DataExtractor<Data, u32>,
    DestinationSampleRateDataExtractor: DataExtractor<Data, u32>,
    PitchModificationDataExtractor: DataExtractor<Data, f32>,
{
    pub fn new(
        parent: Parent,
        source_sample_rate_data_extractor: SourceSampleRateDataExtractor,
        destination_sample_rate_data_extractor: DestinationSampleRateDataExtractor,
        pitch_modification_data_extractor: PitchModificationDataExtractor,
    ) -> Self {
        Self {
            phantom_data: PhantomData,
            parent,
            source_sample_rate_data_extractor,
            destination_sample_rate_data_extractor,
            pitch_modification_data_extractor,
        }
    }
}

impl<
    const CHANNELS: usize,
    Data,
    Parent,
    SourceSampleRateDataExtractor,
    DestinationSampleRateDataExtractor,
    PitchModificationDataExtractor,
> PipelineNode<CHANNELS, Data>
    for Stretcher<
        CHANNELS,
        Data,
        Parent,
        SourceSampleRateDataExtractor,
        DestinationSampleRateDataExtractor,
        PitchModificationDataExtractor,
    >
where
    Parent: PipelineNode<CHANNELS, Data>,
    SourceSampleRateDataExtractor: DataExtractor<Data, u32>,
    DestinationSampleRateDataExtractor: DataExtractor<Data, u32>,
    PitchModificationDataExtractor: DataExtractor<Data, f32>,
{
    fn update(&mut self, data: &Data) {
        self.parent.update(data);
    }

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, clock: usize) -> [f32; CHANNELS] {
        self.parent.next(clock)
    }

    fn reset(&mut self) {
        self.parent.reset();
    }
}
