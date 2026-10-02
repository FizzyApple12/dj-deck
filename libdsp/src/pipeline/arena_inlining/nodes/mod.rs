pub mod filters;
pub mod generation;
pub mod mixing;
pub mod resampler;

use std::marker::PhantomData;

use crate::pipeline::arena_inlining::PipelineNode;

pub struct Null<const CHANNELS: usize, Data> {
    phantom_data: PhantomData<Data>,
}

impl<const CHANNELS: usize, Data> Default for Null<CHANNELS, Data> {
    fn default() -> Self {
        Self {
            phantom_data: PhantomData,
        }
    }
}

impl<const CHANNELS: usize, Data> PipelineNode<CHANNELS, Data> for Null<CHANNELS, Data> {
    fn update(&mut self, _: &Data) {}

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, input: [f32; CHANNELS], _: usize) -> [f32; CHANNELS] {
        input
    }

    fn reset(&mut self) {}
}
