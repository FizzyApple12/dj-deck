use std::marker::PhantomData;

use crate::pipeline::{
    CrossFaderSide,
    nesting_inlining::{DataExtractor, PipelineNode},
};

pub struct CrossFader<const CHANNELS: usize, Data, Parent, FadePercentExtractor, FadeSideExtractor>
where
    Parent: PipelineNode<CHANNELS, Data>,
    FadePercentExtractor: DataExtractor<Data, f32>,
    FadeSideExtractor: DataExtractor<Data, CrossFaderSide>,
{
    phantom_data: PhantomData<Data>,
    parent: Parent,
    fade_percent_extractor: FadePercentExtractor,
    fade_side_extractor: FadeSideExtractor,

    real_percent: f32,
}

impl<const CHANNELS: usize, Data, Parent, FadePercentExtractor, FadeSideExtractor>
    CrossFader<CHANNELS, Data, Parent, FadePercentExtractor, FadeSideExtractor>
where
    Parent: PipelineNode<CHANNELS, Data>,
    FadePercentExtractor: DataExtractor<Data, f32>,
    FadeSideExtractor: DataExtractor<Data, CrossFaderSide>,
{
    pub fn new(
        parent: Parent,
        fade_percent_extractor: FadePercentExtractor,
        fade_side_extractor: FadeSideExtractor,
    ) -> Self {
        Self {
            phantom_data: PhantomData,
            parent,
            fade_percent_extractor,
            fade_side_extractor,

            real_percent: 1.0,
        }
    }
}

impl<const CHANNELS: usize, Data, Parent, FadePercentExtractor, FadeSideExtractor>
    PipelineNode<CHANNELS, Data>
    for CrossFader<CHANNELS, Data, Parent, FadePercentExtractor, FadeSideExtractor>
where
    Parent: PipelineNode<CHANNELS, Data>,
    FadePercentExtractor: DataExtractor<Data, f32>,
    FadeSideExtractor: DataExtractor<Data, CrossFaderSide>,
{
    fn update(&mut self, data: &Data) {
        let fade_percent = (self.fade_percent_extractor)(data);
        let fader_side = (self.fade_side_extractor)(data);

        self.real_percent = match fader_side {
            CrossFaderSide::A => (1.0 - fade_percent).clamp(0.0, 1.0),
            CrossFaderSide::B => (1.0 + fade_percent).clamp(0.0, 1.0),
            CrossFaderSide::None => 1.0,
        };

        self.parent.update(data);
    }

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, clock: usize) -> [f32; CHANNELS] {
        let mut parent_data = self.parent.next(clock);

        for data in &mut parent_data {
            *data *= self.real_percent;
        }

        parent_data
    }

    fn reset(&mut self) {
        self.parent.reset();
    }
}
