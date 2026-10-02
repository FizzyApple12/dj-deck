use std::marker::PhantomData;

use crate::pipeline::arena_inlining::PipelineNode;

pub struct Constant<const CHANNELS: usize, Data> {
    phantom_data: PhantomData<Data>,
    constant: [f32; CHANNELS],
}

impl<const CHANNELS: usize, Data> Constant<CHANNELS, Data> {
    pub fn new(constant: [f32; CHANNELS]) -> Self {
        Self {
            phantom_data: PhantomData,
            constant,
        }
    }
}

impl<const CHANNELS: usize, Data> PipelineNode<CHANNELS, Data> for Constant<CHANNELS, Data> {
    fn update(&mut self, _: &Data) {}

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, _: [f32; CHANNELS], _: usize) -> [f32; CHANNELS] {
        self.constant
    }

    fn reset(&mut self) {}
}
