use std::marker::PhantomData;

use crate::{
    db_to_amplitude,
    pipeline::arena_inlining::{DataExtractor, PipelineNode},
};

pub enum GainMode<Data, GainAmplitudeExtractor, GainDecibelExtractor> {
    Disabled(PhantomData<Data>),
    Amplitude(GainAmplitudeExtractor),
    Decibel(GainDecibelExtractor),
}

pub struct Gain<const CHANNELS: usize, Data, GainAmplitudeExtractor, GainDecibelExtractor>
where
    GainAmplitudeExtractor: DataExtractor<Data, f32>,
    GainDecibelExtractor: DataExtractor<Data, f32>,
{
    phantom_data: PhantomData<Data>,
    mode: GainMode<Data, GainAmplitudeExtractor, GainDecibelExtractor>,

    factor: f32,
}

impl<const CHANNELS: usize, Data, GainAmplitudeExtractor, GainDecibelExtractor>
    Gain<CHANNELS, Data, GainAmplitudeExtractor, GainDecibelExtractor>
where
    GainAmplitudeExtractor: DataExtractor<Data, f32>,
    GainDecibelExtractor: DataExtractor<Data, f32>,
{
    pub fn new(mode: GainMode<Data, GainAmplitudeExtractor, GainDecibelExtractor>) -> Self {
        Self {
            phantom_data: PhantomData,
            mode,

            factor: 1.0,
        }
    }
}

impl<const CHANNELS: usize, Data, GainAmplitudeExtractor, GainDecibelExtractor>
    PipelineNode<CHANNELS, Data>
    for Gain<CHANNELS, Data, GainAmplitudeExtractor, GainDecibelExtractor>
where
    GainAmplitudeExtractor: DataExtractor<Data, f32>,
    GainDecibelExtractor: DataExtractor<Data, f32>,
{
    fn update(&mut self, data: &Data) {
        self.factor = match &self.mode {
            GainMode::Disabled(_) => 1.0,
            GainMode::Amplitude(gain) => (gain)(data),
            GainMode::Decibel(gain) => db_to_amplitude((gain)(data)),
        };
    }

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, mut input: [f32; CHANNELS], _: usize) -> [f32; CHANNELS] {
        for data in &mut input {
            *data *= self.factor;
        }

        input
    }

    fn reset(&mut self) {}
}
