use std::marker::PhantomData;

use crate::pipeline::nesting_inlining::{DataExtractor, PipelineNode};

pub struct ResetInjector<const CHANNELS: usize, Data, Parent, InjectionDataExtractor>
where
    Parent: PipelineNode<CHANNELS, Data>,
    InjectionDataExtractor: DataExtractor<Data, bool>,
{
    phantom_data: PhantomData<Data>,
    parent: Parent,
    injection_data_extractor: InjectionDataExtractor,
}

impl<const CHANNELS: usize, Data, Parent, InjectionDataExtractor>
    ResetInjector<CHANNELS, Data, Parent, InjectionDataExtractor>
where
    Parent: PipelineNode<CHANNELS, Data>,
    InjectionDataExtractor: DataExtractor<Data, bool>,
{
    pub fn new(parent: Parent, injection_data_extractor: InjectionDataExtractor) -> Self {
        Self {
            phantom_data: PhantomData,
            parent,
            injection_data_extractor,
        }
    }
}

impl<const CHANNELS: usize, Data, Parent, InjectionDataExtractor> PipelineNode<CHANNELS, Data>
    for ResetInjector<CHANNELS, Data, Parent, InjectionDataExtractor>
where
    Parent: PipelineNode<CHANNELS, Data>,
    InjectionDataExtractor: DataExtractor<Data, bool>,
{
    fn update(&mut self, data: &Data) {
        if (self.injection_data_extractor)(data) {
            self.parent.reset();
        }

        self.parent.update(data);
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

pub struct ResetGuard<const CHANNELS: usize, ClockDomain, Data, Parent>
where
    Parent: PipelineNode<CHANNELS, Data>,
{
    phantom_data: PhantomData<Data>,
    phantom_clock_domain: PhantomData<ClockDomain>,
    parent: Parent,
}

impl<const CHANNELS: usize, ClockDomain, Data, Parent>
    ResetGuard<CHANNELS, ClockDomain, Data, Parent>
where
    Parent: PipelineNode<CHANNELS, Data>,
{
    pub fn new(parent: Parent) -> Self {
        Self {
            phantom_data: PhantomData,
            phantom_clock_domain: PhantomData,
            parent,
        }
    }
}

impl<const CHANNELS: usize, ClockDomain, Data, Parent> PipelineNode<CHANNELS, Data>
    for ResetGuard<CHANNELS, ClockDomain, Data, Parent>
where
    Parent: PipelineNode<CHANNELS, Data>,
{
    fn update(&mut self, data: &Data) {
        self.parent.update(data);
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
