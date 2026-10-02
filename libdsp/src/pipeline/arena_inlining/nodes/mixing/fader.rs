use std::marker::PhantomData;

use crate::pipeline::arena_inlining::{DataExtractor, PipelineNode};

pub struct Fader<const CHANNELS: usize, Data, FadePercentExtractor>
where
    FadePercentExtractor: DataExtractor<Data, f32>,
{
    phantom_data: PhantomData<Data>,
    fade_percent_extractor: FadePercentExtractor,

    fade_percent: f32,
}

impl<const CHANNELS: usize, Data, FadePercentExtractor> Fader<CHANNELS, Data, FadePercentExtractor>
where
    FadePercentExtractor: DataExtractor<Data, f32>,
{
    pub fn new(fade_percent_extractor: FadePercentExtractor) -> Self {
        Self {
            phantom_data: PhantomData,
            fade_percent_extractor,

            fade_percent: 1.0,
        }
    }
}

impl<const CHANNELS: usize, Data, FadePercentExtractor> PipelineNode<CHANNELS, Data>
    for Fader<CHANNELS, Data, FadePercentExtractor>
where
    FadePercentExtractor: DataExtractor<Data, f32>,
{
    fn update(&mut self, data: &Data) {
        self.fade_percent = (self.fade_percent_extractor)(data).clamp(0.0, 1.0);
    }

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, mut input: [f32; CHANNELS], _: usize) -> [f32; CHANNELS] {
        for data in &mut input {
            *data *= self.fade_percent;
        }

        input
    }

    fn reset(&mut self) {}
}
