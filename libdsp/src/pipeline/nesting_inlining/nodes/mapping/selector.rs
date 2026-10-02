use std::marker::PhantomData;

use crate::pipeline::nesting_inlining::{DataExtractor, PipelineNode};

#[derive(Clone)]
pub struct Selector<const CHANNELS: usize, Data, Parents, SelectedBranchExtractor>
where
    SelectedBranchExtractor: DataExtractor<Data, usize>,
{
    phantom_data: PhantomData<Data>,
    parents: Parents,

    selected_branch_extractor: SelectedBranchExtractor,

    selected_branch: usize,
}

impl<const CHANNELS: usize, Data, T1, T2, SelectedBranchExtractor>
    Selector<CHANNELS, Data, (T1, T2), SelectedBranchExtractor>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    SelectedBranchExtractor: DataExtractor<Data, usize>,
{
    pub fn new(parents: (T1, T2), selected_branch_extractor: SelectedBranchExtractor) -> Self {
        Self {
            phantom_data: PhantomData,
            parents,

            selected_branch_extractor,

            selected_branch: 0,
        }
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3, SelectedBranchExtractor>
    Selector<CHANNELS, Data, (T1, T2, T3), SelectedBranchExtractor>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
    SelectedBranchExtractor: DataExtractor<Data, usize>,
{
    pub fn new(parents: (T1, T2, T3), selected_branch_extractor: SelectedBranchExtractor) -> Self {
        Self {
            phantom_data: PhantomData,
            parents,

            selected_branch_extractor,

            selected_branch: 0,
        }
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3, T4, SelectedBranchExtractor>
    Selector<CHANNELS, Data, (T1, T2, T3, T4), SelectedBranchExtractor>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
    T4: PipelineNode<CHANNELS, Data>,
    SelectedBranchExtractor: DataExtractor<Data, usize>,
{
    pub fn new(
        parents: (T1, T2, T3, T4),
        selected_branch_extractor: SelectedBranchExtractor,
    ) -> Self {
        Self {
            phantom_data: PhantomData,
            parents,

            selected_branch_extractor,

            selected_branch: 0,
        }
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3, T4, T5, SelectedBranchExtractor>
    Selector<CHANNELS, Data, (T1, T2, T3, T4, T5), SelectedBranchExtractor>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
    T4: PipelineNode<CHANNELS, Data>,
    T5: PipelineNode<CHANNELS, Data>,
    SelectedBranchExtractor: DataExtractor<Data, usize>,
{
    pub fn new(
        parents: (T1, T2, T3, T4, T5),
        selected_branch_extractor: SelectedBranchExtractor,
    ) -> Self {
        Self {
            phantom_data: PhantomData,
            parents,

            selected_branch_extractor,

            selected_branch: 0,
        }
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3, T4, T5, T6, SelectedBranchExtractor>
    Selector<CHANNELS, Data, (T1, T2, T3, T4, T5, T6), SelectedBranchExtractor>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
    T4: PipelineNode<CHANNELS, Data>,
    T5: PipelineNode<CHANNELS, Data>,
    T6: PipelineNode<CHANNELS, Data>,
    SelectedBranchExtractor: DataExtractor<Data, usize>,
{
    pub fn new(
        parents: (T1, T2, T3, T4, T5, T6),
        selected_branch_extractor: SelectedBranchExtractor,
    ) -> Self {
        Self {
            phantom_data: PhantomData,
            parents,

            selected_branch_extractor,

            selected_branch: 0,
        }
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3, T4, T5, T6, T7, SelectedBranchExtractor>
    Selector<CHANNELS, Data, (T1, T2, T3, T4, T5, T6, T7), SelectedBranchExtractor>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
    T4: PipelineNode<CHANNELS, Data>,
    T5: PipelineNode<CHANNELS, Data>,
    T6: PipelineNode<CHANNELS, Data>,
    T7: PipelineNode<CHANNELS, Data>,
    SelectedBranchExtractor: DataExtractor<Data, usize>,
{
    pub fn new(
        parents: (T1, T2, T3, T4, T5, T6, T7),
        selected_branch_extractor: SelectedBranchExtractor,
    ) -> Self {
        Self {
            phantom_data: PhantomData,
            parents,

            selected_branch_extractor,

            selected_branch: 0,
        }
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3, T4, T5, T6, T7, T8, SelectedBranchExtractor>
    Selector<CHANNELS, Data, (T1, T2, T3, T4, T5, T6, T7, T8), SelectedBranchExtractor>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
    T4: PipelineNode<CHANNELS, Data>,
    T5: PipelineNode<CHANNELS, Data>,
    T6: PipelineNode<CHANNELS, Data>,
    T7: PipelineNode<CHANNELS, Data>,
    T8: PipelineNode<CHANNELS, Data>,
    SelectedBranchExtractor: DataExtractor<Data, usize>,
{
    pub fn new(
        parents: (T1, T2, T3, T4, T5, T6, T7, T8),
        selected_branch_extractor: SelectedBranchExtractor,
    ) -> Self {
        Self {
            phantom_data: PhantomData,
            parents,

            selected_branch_extractor,

            selected_branch: 0,
        }
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3, T4, T5, T6, T7, T8, T9, SelectedBranchExtractor>
    Selector<CHANNELS, Data, (T1, T2, T3, T4, T5, T6, T7, T8, T9), SelectedBranchExtractor>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
    T4: PipelineNode<CHANNELS, Data>,
    T5: PipelineNode<CHANNELS, Data>,
    T6: PipelineNode<CHANNELS, Data>,
    T7: PipelineNode<CHANNELS, Data>,
    T8: PipelineNode<CHANNELS, Data>,
    T9: PipelineNode<CHANNELS, Data>,
    SelectedBranchExtractor: DataExtractor<Data, usize>,
{
    pub fn new(
        parents: (T1, T2, T3, T4, T5, T6, T7, T8, T9),
        selected_branch_extractor: SelectedBranchExtractor,
    ) -> Self {
        Self {
            phantom_data: PhantomData,
            parents,

            selected_branch_extractor,

            selected_branch: 0,
        }
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, SelectedBranchExtractor>
    Selector<CHANNELS, Data, (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10), SelectedBranchExtractor>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
    T4: PipelineNode<CHANNELS, Data>,
    T5: PipelineNode<CHANNELS, Data>,
    T6: PipelineNode<CHANNELS, Data>,
    T7: PipelineNode<CHANNELS, Data>,
    T8: PipelineNode<CHANNELS, Data>,
    T9: PipelineNode<CHANNELS, Data>,
    T10: PipelineNode<CHANNELS, Data>,
    SelectedBranchExtractor: DataExtractor<Data, usize>,
{
    pub fn new(
        parents: (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10),
        selected_branch_extractor: SelectedBranchExtractor,
    ) -> Self {
        Self {
            phantom_data: PhantomData,
            parents,

            selected_branch_extractor,

            selected_branch: 0,
        }
    }
}

impl<const CHANNELS: usize, Data, T1, T2, SelectedBranchExtractor> PipelineNode<CHANNELS, Data>
    for Selector<CHANNELS, Data, (T1, T2), SelectedBranchExtractor>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    SelectedBranchExtractor: DataExtractor<Data, usize>,
{
    fn update(&mut self, data: &Data) {
        self.selected_branch = (self.selected_branch_extractor)(data);

        self.parents.0.update(data);
        self.parents.1.update(data);
    }

    #[allow(clippy::indexing_slicing)]
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, clock: usize) -> [f32; CHANNELS] {
        let parent_values = [self.parents.0.next(clock), self.parents.1.next(clock)];

        *parent_values
            .get(self.selected_branch)
            .unwrap_or(&[0.0; CHANNELS])
    }

    fn reset(&mut self) {
        self.parents.0.reset();
        self.parents.1.reset();
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3, SelectedBranchExtractor> PipelineNode<CHANNELS, Data>
    for Selector<CHANNELS, Data, (T1, T2, T3), SelectedBranchExtractor>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
    SelectedBranchExtractor: DataExtractor<Data, usize>,
{
    fn update(&mut self, data: &Data) {
        self.selected_branch = (self.selected_branch_extractor)(data);

        self.parents.0.update(data);
        self.parents.1.update(data);
        self.parents.2.update(data);
    }

    #[allow(clippy::indexing_slicing)]
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, clock: usize) -> [f32; CHANNELS] {
        let parent_values = [
            self.parents.0.next(clock),
            self.parents.1.next(clock),
            self.parents.2.next(clock),
        ];

        *parent_values
            .get(self.selected_branch)
            .unwrap_or(&[0.0; CHANNELS])
    }

    fn reset(&mut self) {
        self.parents.0.reset();
        self.parents.1.reset();
        self.parents.2.reset();
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3, T4, SelectedBranchExtractor>
    PipelineNode<CHANNELS, Data>
    for Selector<CHANNELS, Data, (T1, T2, T3, T4), SelectedBranchExtractor>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
    T4: PipelineNode<CHANNELS, Data>,
    SelectedBranchExtractor: DataExtractor<Data, usize>,
{
    fn update(&mut self, data: &Data) {
        self.selected_branch = (self.selected_branch_extractor)(data);

        self.parents.0.update(data);
        self.parents.1.update(data);
        self.parents.2.update(data);
        self.parents.3.update(data);
    }

    #[allow(clippy::indexing_slicing)]
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, clock: usize) -> [f32; CHANNELS] {
        let parent_values = [
            self.parents.0.next(clock),
            self.parents.1.next(clock),
            self.parents.2.next(clock),
            self.parents.3.next(clock),
        ];

        *parent_values
            .get(self.selected_branch)
            .unwrap_or(&[0.0; CHANNELS])
    }

    fn reset(&mut self) {
        self.parents.0.reset();
        self.parents.1.reset();
        self.parents.2.reset();
        self.parents.3.reset();
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3, T4, T5, SelectedBranchExtractor>
    PipelineNode<CHANNELS, Data>
    for Selector<CHANNELS, Data, (T1, T2, T3, T4, T5), SelectedBranchExtractor>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
    T4: PipelineNode<CHANNELS, Data>,
    T5: PipelineNode<CHANNELS, Data>,
    SelectedBranchExtractor: DataExtractor<Data, usize>,
{
    fn update(&mut self, data: &Data) {
        self.selected_branch = (self.selected_branch_extractor)(data);

        self.parents.0.update(data);
        self.parents.1.update(data);
        self.parents.2.update(data);
        self.parents.3.update(data);
        self.parents.4.update(data);
    }

    #[allow(clippy::indexing_slicing)]
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, clock: usize) -> [f32; CHANNELS] {
        let parent_values = [
            self.parents.0.next(clock),
            self.parents.1.next(clock),
            self.parents.2.next(clock),
            self.parents.3.next(clock),
        ];

        *parent_values
            .get(self.selected_branch)
            .unwrap_or(&[0.0; CHANNELS])
    }

    fn reset(&mut self) {
        self.parents.0.reset();
        self.parents.1.reset();
        self.parents.2.reset();
        self.parents.3.reset();
        self.parents.4.reset();
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3, T4, T5, T6, SelectedBranchExtractor>
    PipelineNode<CHANNELS, Data>
    for Selector<CHANNELS, Data, (T1, T2, T3, T4, T5, T6), SelectedBranchExtractor>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
    T4: PipelineNode<CHANNELS, Data>,
    T5: PipelineNode<CHANNELS, Data>,
    T6: PipelineNode<CHANNELS, Data>,
    SelectedBranchExtractor: DataExtractor<Data, usize>,
{
    fn update(&mut self, data: &Data) {
        self.selected_branch = (self.selected_branch_extractor)(data);

        self.parents.0.update(data);
        self.parents.1.update(data);
        self.parents.2.update(data);
        self.parents.3.update(data);
        self.parents.4.update(data);
        self.parents.5.update(data);
    }

    #[allow(clippy::indexing_slicing)]
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, clock: usize) -> [f32; CHANNELS] {
        let parent_values = [
            self.parents.0.next(clock),
            self.parents.1.next(clock),
            self.parents.2.next(clock),
            self.parents.3.next(clock),
            self.parents.4.next(clock),
            self.parents.5.next(clock),
        ];

        *parent_values
            .get(self.selected_branch)
            .unwrap_or(&[0.0; CHANNELS])
    }

    fn reset(&mut self) {
        self.parents.0.reset();
        self.parents.1.reset();
        self.parents.2.reset();
        self.parents.3.reset();
        self.parents.4.reset();
        self.parents.5.reset();
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3, T4, T5, T6, T7, SelectedBranchExtractor>
    PipelineNode<CHANNELS, Data>
    for Selector<CHANNELS, Data, (T1, T2, T3, T4, T5, T6, T7), SelectedBranchExtractor>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
    T4: PipelineNode<CHANNELS, Data>,
    T5: PipelineNode<CHANNELS, Data>,
    T6: PipelineNode<CHANNELS, Data>,
    T7: PipelineNode<CHANNELS, Data>,
    SelectedBranchExtractor: DataExtractor<Data, usize>,
{
    fn update(&mut self, data: &Data) {
        self.selected_branch = (self.selected_branch_extractor)(data);

        self.parents.0.update(data);
        self.parents.1.update(data);
        self.parents.2.update(data);
        self.parents.3.update(data);
        self.parents.4.update(data);
        self.parents.5.update(data);
        self.parents.6.update(data);
    }

    #[allow(clippy::indexing_slicing)]
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, clock: usize) -> [f32; CHANNELS] {
        let parent_values = [
            self.parents.0.next(clock),
            self.parents.1.next(clock),
            self.parents.2.next(clock),
            self.parents.3.next(clock),
            self.parents.4.next(clock),
            self.parents.5.next(clock),
            self.parents.6.next(clock),
        ];

        *parent_values
            .get(self.selected_branch)
            .unwrap_or(&[0.0; CHANNELS])
    }

    fn reset(&mut self) {
        self.parents.0.reset();
        self.parents.1.reset();
        self.parents.2.reset();
        self.parents.3.reset();
        self.parents.4.reset();
        self.parents.5.reset();
        self.parents.6.reset();
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3, T4, T5, T6, T7, T8, SelectedBranchExtractor>
    PipelineNode<CHANNELS, Data>
    for Selector<CHANNELS, Data, (T1, T2, T3, T4, T5, T6, T7, T8), SelectedBranchExtractor>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
    T4: PipelineNode<CHANNELS, Data>,
    T5: PipelineNode<CHANNELS, Data>,
    T6: PipelineNode<CHANNELS, Data>,
    T7: PipelineNode<CHANNELS, Data>,
    T8: PipelineNode<CHANNELS, Data>,
    SelectedBranchExtractor: DataExtractor<Data, usize>,
{
    fn update(&mut self, data: &Data) {
        self.selected_branch = (self.selected_branch_extractor)(data);

        self.parents.0.update(data);
        self.parents.1.update(data);
        self.parents.2.update(data);
        self.parents.3.update(data);
        self.parents.4.update(data);
        self.parents.5.update(data);
        self.parents.6.update(data);
        self.parents.7.update(data);
    }

    #[allow(clippy::indexing_slicing)]
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, clock: usize) -> [f32; CHANNELS] {
        let parent_values = [
            self.parents.0.next(clock),
            self.parents.1.next(clock),
            self.parents.2.next(clock),
            self.parents.3.next(clock),
            self.parents.4.next(clock),
            self.parents.5.next(clock),
            self.parents.6.next(clock),
            self.parents.7.next(clock),
        ];

        *parent_values
            .get(self.selected_branch)
            .unwrap_or(&[0.0; CHANNELS])
    }

    fn reset(&mut self) {
        self.parents.0.reset();
        self.parents.1.reset();
        self.parents.2.reset();
        self.parents.3.reset();
        self.parents.4.reset();
        self.parents.5.reset();
        self.parents.6.reset();
        self.parents.7.reset();
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3, T4, T5, T6, T7, T8, T9, SelectedBranchExtractor>
    PipelineNode<CHANNELS, Data>
    for Selector<CHANNELS, Data, (T1, T2, T3, T4, T5, T6, T7, T8, T9), SelectedBranchExtractor>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
    T4: PipelineNode<CHANNELS, Data>,
    T5: PipelineNode<CHANNELS, Data>,
    T6: PipelineNode<CHANNELS, Data>,
    T7: PipelineNode<CHANNELS, Data>,
    T8: PipelineNode<CHANNELS, Data>,
    T9: PipelineNode<CHANNELS, Data>,
    SelectedBranchExtractor: DataExtractor<Data, usize>,
{
    fn update(&mut self, data: &Data) {
        self.selected_branch = (self.selected_branch_extractor)(data);

        self.parents.0.update(data);
        self.parents.1.update(data);
        self.parents.2.update(data);
        self.parents.3.update(data);
        self.parents.4.update(data);
        self.parents.5.update(data);
        self.parents.6.update(data);
        self.parents.7.update(data);
        self.parents.8.update(data);
    }

    #[allow(clippy::indexing_slicing)]
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, clock: usize) -> [f32; CHANNELS] {
        let parent_values = [
            self.parents.0.next(clock),
            self.parents.1.next(clock),
            self.parents.2.next(clock),
            self.parents.3.next(clock),
            self.parents.4.next(clock),
            self.parents.5.next(clock),
            self.parents.6.next(clock),
            self.parents.7.next(clock),
            self.parents.8.next(clock),
        ];

        *parent_values
            .get(self.selected_branch)
            .unwrap_or(&[0.0; CHANNELS])
    }

    fn reset(&mut self) {
        self.parents.0.reset();
        self.parents.1.reset();
        self.parents.2.reset();
        self.parents.3.reset();
        self.parents.4.reset();
        self.parents.5.reset();
        self.parents.6.reset();
        self.parents.7.reset();
        self.parents.8.reset();
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, SelectedBranchExtractor>
    PipelineNode<CHANNELS, Data>
    for Selector<CHANNELS, Data, (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10), SelectedBranchExtractor>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
    T4: PipelineNode<CHANNELS, Data>,
    T5: PipelineNode<CHANNELS, Data>,
    T6: PipelineNode<CHANNELS, Data>,
    T7: PipelineNode<CHANNELS, Data>,
    T8: PipelineNode<CHANNELS, Data>,
    T9: PipelineNode<CHANNELS, Data>,
    T10: PipelineNode<CHANNELS, Data>,
    SelectedBranchExtractor: DataExtractor<Data, usize>,
{
    fn update(&mut self, data: &Data) {
        self.selected_branch = (self.selected_branch_extractor)(data);

        self.parents.0.update(data);
        self.parents.1.update(data);
        self.parents.2.update(data);
        self.parents.3.update(data);
        self.parents.4.update(data);
        self.parents.5.update(data);
        self.parents.6.update(data);
        self.parents.7.update(data);
        self.parents.8.update(data);
        self.parents.9.update(data);
    }

    #[allow(clippy::indexing_slicing)]
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, clock: usize) -> [f32; CHANNELS] {
        let parent_values = [
            self.parents.0.next(clock),
            self.parents.1.next(clock),
            self.parents.2.next(clock),
            self.parents.3.next(clock),
            self.parents.4.next(clock),
            self.parents.5.next(clock),
            self.parents.6.next(clock),
            self.parents.7.next(clock),
            self.parents.8.next(clock),
            self.parents.9.next(clock),
        ];

        *parent_values
            .get(self.selected_branch)
            .unwrap_or(&[0.0; CHANNELS])
    }

    fn reset(&mut self) {
        self.parents.0.reset();
        self.parents.1.reset();
        self.parents.2.reset();
        self.parents.3.reset();
        self.parents.4.reset();
        self.parents.5.reset();
        self.parents.6.reset();
        self.parents.7.reset();
        self.parents.8.reset();
        self.parents.9.reset();
    }
}
