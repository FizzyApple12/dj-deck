use std::marker::PhantomData;

use crate::{
    db_to_amplitude,
    pipeline::nesting_inlining::{DataExtractor, PipelineNode},
};

pub enum GainMode<Data, GainAmplitudeExtractor, GainDecibelExtractor> {
    Disabled(PhantomData<Data>),
    Amplitude(GainAmplitudeExtractor),
    Decibel(GainDecibelExtractor),
}

pub struct Gain<const CHANNELS: usize, Data, Parent, GainAmplitudeExtractor, GainDecibelExtractor>
where
    Parent: PipelineNode<CHANNELS, Data>,
    GainAmplitudeExtractor: DataExtractor<Data, f32>,
    GainDecibelExtractor: DataExtractor<Data, f32>,
{
    phantom_data: PhantomData<Data>,
    parent: Parent,
    mode: GainMode<Data, GainAmplitudeExtractor, GainDecibelExtractor>,

    gain: f32,
}

impl<const CHANNELS: usize, Data, Parent, GainAmplitudeExtractor, GainDecibelExtractor>
    Gain<CHANNELS, Data, Parent, GainAmplitudeExtractor, GainDecibelExtractor>
where
    Parent: PipelineNode<CHANNELS, Data>,
    GainAmplitudeExtractor: DataExtractor<Data, f32>,
    GainDecibelExtractor: DataExtractor<Data, f32>,
{
    pub fn new(
        parent: Parent,
        mode: GainMode<Data, GainAmplitudeExtractor, GainDecibelExtractor>,
    ) -> Self {
        Self {
            phantom_data: PhantomData,
            parent,
            mode,

            gain: 1.0,
        }
    }
}

impl<const CHANNELS: usize, Data, Parent, GainAmplitudeExtractor, GainDecibelExtractor>
    PipelineNode<CHANNELS, Data>
    for Gain<CHANNELS, Data, Parent, GainAmplitudeExtractor, GainDecibelExtractor>
where
    Parent: PipelineNode<CHANNELS, Data>,
    GainAmplitudeExtractor: DataExtractor<Data, f32>,
    GainDecibelExtractor: DataExtractor<Data, f32>,
{
    fn update(&mut self, data: &Data) {
        self.gain = match &self.mode {
            GainMode::Disabled(_) => 1.0,
            GainMode::Amplitude(gain) => (gain)(data),
            GainMode::Decibel(gain) => db_to_amplitude((gain)(data)),
        };

        self.parent.update(data);
    }

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, clock: usize) -> [f32; CHANNELS] {
        let mut parent_data = self.parent.next(clock);

        for data in &mut parent_data {
            *data *= self.gain;
        }

        parent_data
    }

    fn reset(&mut self) {
        self.parent.reset();
    }
}
