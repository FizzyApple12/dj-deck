use std::marker::PhantomData;

use crate::pipeline::nesting_inlining::{DataExtractor, PipelineNode};

pub struct DataMapper<const CHANNELS: usize, DataOut, DataIn, Parent, DataOutExtractor>
where
    Parent: PipelineNode<CHANNELS, DataOut>,
    DataOutExtractor: DataExtractor<DataIn, DataOut>,
{
    phantom_data_out: PhantomData<DataOut>,
    phantom_data_in: PhantomData<DataIn>,
    parent: Parent,
    data_out_extractor: DataOutExtractor,
}

impl<const CHANNELS: usize, DataOut, DataIn, Parent, DataOutExtractor>
    DataMapper<CHANNELS, DataOut, DataIn, Parent, DataOutExtractor>
where
    Parent: PipelineNode<CHANNELS, DataOut>,
    DataOutExtractor: DataExtractor<DataIn, DataOut>,
{
    pub fn new(parent: Parent, data_out_extractor: DataOutExtractor) -> Self {
        Self {
            phantom_data_out: PhantomData,
            phantom_data_in: PhantomData,
            parent,
            data_out_extractor,
        }
    }
}

impl<const CHANNELS: usize, DataOut, DataIn, Parent, DataOutExtractor>
    PipelineNode<CHANNELS, DataIn>
    for DataMapper<CHANNELS, DataOut, DataIn, Parent, DataOutExtractor>
where
    Parent: PipelineNode<CHANNELS, DataOut>,
    DataOutExtractor: DataExtractor<DataIn, DataOut>,
{
    fn update(&mut self, data: &DataIn) {
        let mapped_data = (self.data_out_extractor)(data);

        self.parent.update(&mapped_data);
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
