use std::marker::PhantomData;

use crate::pipeline::nesting_inlining::{DataExtractor, PipelineNode};

pub struct Fader<const CHANNELS: usize, Data, Parent, FadePercentExtractor>
where
    Parent: PipelineNode<CHANNELS, Data>,
    FadePercentExtractor: DataExtractor<Data, f32>,
{
    phantom_data: PhantomData<Data>,
    parent: Parent,
    fade_percent_extractor: FadePercentExtractor,

    fade_percent: f32,
}

impl<const CHANNELS: usize, Data, Parent, FadePercentExtractor>
    Fader<CHANNELS, Data, Parent, FadePercentExtractor>
where
    Parent: PipelineNode<CHANNELS, Data>,
    FadePercentExtractor: DataExtractor<Data, f32>,
{
    pub fn new(parent: Parent, fade_percent_extractor: FadePercentExtractor) -> Self {
        Self {
            phantom_data: PhantomData,
            parent,
            fade_percent_extractor,

            fade_percent: 1.0,
        }
    }
}

impl<const CHANNELS: usize, Data, Parent, FadePercentExtractor> PipelineNode<CHANNELS, Data>
    for Fader<CHANNELS, Data, Parent, FadePercentExtractor>
where
    Parent: PipelineNode<CHANNELS, Data>,
    FadePercentExtractor: DataExtractor<Data, f32>,
{
    fn update(&mut self, data: &Data) {
        self.fade_percent = (self.fade_percent_extractor)(data).clamp(0.0, 1.0);

        self.parent.update(data);
    }

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, clock: usize) -> [f32; CHANNELS] {
        let mut parent_data = self.parent.next(clock);

        for data in &mut parent_data {
            *data *= self.fade_percent;
        }

        parent_data
    }

    fn reset(&mut self) {
        self.parent.reset();
    }
}
