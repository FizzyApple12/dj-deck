use std::marker::PhantomData;

use crate::pipeline::{
    CrossFaderSide,
    arena_inlining::{DataExtractor, PipelineNode},
};

pub struct CrossFader<const CHANNELS: usize, Data, FadePercentExtractor, FadeSideExtractor>
where
    FadePercentExtractor: DataExtractor<Data, f32>,
    FadeSideExtractor: DataExtractor<Data, CrossFaderSide>,
{
    phantom_data: PhantomData<Data>,
    fade_percent_extractor: FadePercentExtractor,
    fade_side_extractor: FadeSideExtractor,

    real_percent: f32,
}

impl<const CHANNELS: usize, Data, FadePercentExtractor, FadeSideExtractor>
    CrossFader<CHANNELS, Data, FadePercentExtractor, FadeSideExtractor>
where
    FadePercentExtractor: DataExtractor<Data, f32>,
    FadeSideExtractor: DataExtractor<Data, CrossFaderSide>,
{
    pub fn new(
        fade_percent_extractor: FadePercentExtractor,
        fade_side_extractor: FadeSideExtractor,
    ) -> Self {
        Self {
            phantom_data: PhantomData,
            fade_percent_extractor,
            fade_side_extractor,

            real_percent: 1.0,
        }
    }
}

impl<const CHANNELS: usize, Data, FadePercentExtractor, FadeSideExtractor>
    PipelineNode<CHANNELS, Data>
    for CrossFader<CHANNELS, Data, FadePercentExtractor, FadeSideExtractor>
where
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
    }

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, mut input: [f32; CHANNELS], _: usize) -> [f32; CHANNELS] {
        for data in &mut input {
            *data *= self.real_percent;
        }

        input
    }

    fn reset(&mut self) {}
}
