use std::marker::PhantomData;

use crate::pipeline::arena_inlining::{DataExtractor, PipelineNode};

pub struct Injector<const CHANNELS: usize, Data, InjectionDataExtractor>
where
    InjectionDataExtractor: DataExtractor<Data, [f32; CHANNELS]>,
{
    phantom_data: PhantomData<Data>,
    injection_data_extractor: InjectionDataExtractor,

    data: [f32; CHANNELS],
}

impl<const CHANNELS: usize, Data, InjectionDataExtractor>
    Injector<CHANNELS, Data, InjectionDataExtractor>
where
    InjectionDataExtractor: DataExtractor<Data, [f32; CHANNELS]>,
{
    pub fn new(injection_data_extractor: InjectionDataExtractor) -> Self {
        Self {
            phantom_data: PhantomData,
            injection_data_extractor,

            data: [0.0; CHANNELS],
        }
    }
}

impl<const CHANNELS: usize, Data, InjectionDataExtractor> PipelineNode<CHANNELS, Data>
    for Injector<CHANNELS, Data, InjectionDataExtractor>
where
    InjectionDataExtractor: DataExtractor<Data, [f32; CHANNELS]>,
{
    fn update(&mut self, data: &Data) {
        self.data = (self.injection_data_extractor)(data);
    }

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, _: [f32; CHANNELS], _: usize) -> [f32; CHANNELS] {
        self.data
    }

    fn reset(&mut self) {}
}
