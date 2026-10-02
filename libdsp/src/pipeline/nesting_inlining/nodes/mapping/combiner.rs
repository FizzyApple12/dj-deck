use std::marker::PhantomData;

use crate::pipeline::nesting_inlining::PipelineNode;

#[derive(Clone)]
pub struct Combiner<const CHANNELS: usize, Data, Parents> {
    phantom_data: PhantomData<Data>,
    parents: Parents,
}

impl<const CHANNELS: usize, Data, T1, T2> Combiner<CHANNELS, Data, (T1, T2)>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
{
    pub fn new(parents: (T1, T2)) -> Self {
        Self {
            phantom_data: PhantomData,
            parents,
        }
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3> Combiner<CHANNELS, Data, (T1, T2, T3)>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
{
    pub fn new(parents: (T1, T2, T3)) -> Self {
        Self {
            phantom_data: PhantomData,
            parents,
        }
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3, T4> Combiner<CHANNELS, Data, (T1, T2, T3, T4)>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
    T4: PipelineNode<CHANNELS, Data>,
{
    pub fn new(parents: (T1, T2, T3, T4)) -> Self {
        Self {
            phantom_data: PhantomData,
            parents,
        }
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3, T4, T5> Combiner<CHANNELS, Data, (T1, T2, T3, T4, T5)>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
    T4: PipelineNode<CHANNELS, Data>,
    T5: PipelineNode<CHANNELS, Data>,
{
    pub fn new(parents: (T1, T2, T3, T4, T5)) -> Self {
        Self {
            phantom_data: PhantomData,
            parents,
        }
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3, T4, T5, T6>
    Combiner<CHANNELS, Data, (T1, T2, T3, T4, T5, T6)>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
    T4: PipelineNode<CHANNELS, Data>,
    T5: PipelineNode<CHANNELS, Data>,
    T6: PipelineNode<CHANNELS, Data>,
{
    pub fn new(parents: (T1, T2, T3, T4, T5, T6)) -> Self {
        Self {
            phantom_data: PhantomData,
            parents,
        }
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3, T4, T5, T6, T7>
    Combiner<CHANNELS, Data, (T1, T2, T3, T4, T5, T6, T7)>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
    T4: PipelineNode<CHANNELS, Data>,
    T5: PipelineNode<CHANNELS, Data>,
    T6: PipelineNode<CHANNELS, Data>,
    T7: PipelineNode<CHANNELS, Data>,
{
    pub fn new(parents: (T1, T2, T3, T4, T5, T6, T7)) -> Self {
        Self {
            phantom_data: PhantomData,
            parents,
        }
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3, T4, T5, T6, T7, T8>
    Combiner<CHANNELS, Data, (T1, T2, T3, T4, T5, T6, T7, T8)>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
    T4: PipelineNode<CHANNELS, Data>,
    T5: PipelineNode<CHANNELS, Data>,
    T6: PipelineNode<CHANNELS, Data>,
    T7: PipelineNode<CHANNELS, Data>,
    T8: PipelineNode<CHANNELS, Data>,
{
    pub fn new(parents: (T1, T2, T3, T4, T5, T6, T7, T8)) -> Self {
        Self {
            phantom_data: PhantomData,
            parents,
        }
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3, T4, T5, T6, T7, T8, T9>
    Combiner<CHANNELS, Data, (T1, T2, T3, T4, T5, T6, T7, T8, T9)>
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
{
    pub fn new(parents: (T1, T2, T3, T4, T5, T6, T7, T8, T9)) -> Self {
        Self {
            phantom_data: PhantomData,
            parents,
        }
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10>
    Combiner<CHANNELS, Data, (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10)>
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
{
    pub fn new(parents: (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10)) -> Self {
        Self {
            phantom_data: PhantomData,
            parents,
        }
    }
}

impl<const CHANNELS: usize, Data, T1, T2> PipelineNode<CHANNELS, Data>
    for Combiner<CHANNELS, Data, (T1, T2)>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
{
    fn update(&mut self, data: &Data) {
        self.parents.0.update(data);
        self.parents.1.update(data);
    }

    #[allow(clippy::indexing_slicing)]
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, clock: usize) -> [f32; CHANNELS] {
        let mut final_value = [0.0; CHANNELS];

        let parent_value_0 = self.parents.0.next(clock);
        let parent_value_1 = self.parents.1.next(clock);

        for channel in 0..CHANNELS {
            final_value[channel] += parent_value_0[channel] + parent_value_1[channel];
        }

        final_value
    }

    fn reset(&mut self) {
        self.parents.0.reset();
        self.parents.1.reset();
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3> PipelineNode<CHANNELS, Data>
    for Combiner<CHANNELS, Data, (T1, T2, T3)>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
{
    fn update(&mut self, data: &Data) {
        self.parents.0.update(data);
        self.parents.1.update(data);
        self.parents.2.update(data);
    }

    #[allow(clippy::indexing_slicing)]
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, clock: usize) -> [f32; CHANNELS] {
        let mut final_value = [0.0; CHANNELS];

        let parent_value_0 = self.parents.0.next(clock);
        let parent_value_1 = self.parents.1.next(clock);
        let parent_value_2 = self.parents.2.next(clock);

        for channel in 0..CHANNELS {
            final_value[channel] +=
                parent_value_0[channel] + parent_value_1[channel] + parent_value_2[channel];
        }

        final_value
    }

    fn reset(&mut self) {
        self.parents.0.reset();
        self.parents.1.reset();
        self.parents.2.reset();
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3, T4> PipelineNode<CHANNELS, Data>
    for Combiner<CHANNELS, Data, (T1, T2, T3, T4)>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
    T4: PipelineNode<CHANNELS, Data>,
{
    fn update(&mut self, data: &Data) {
        self.parents.0.update(data);
        self.parents.1.update(data);
        self.parents.2.update(data);
        self.parents.3.update(data);
    }

    #[allow(clippy::indexing_slicing)]
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, clock: usize) -> [f32; CHANNELS] {
        let mut final_value = [0.0; CHANNELS];

        let parent_value_0 = self.parents.0.next(clock);
        let parent_value_1 = self.parents.1.next(clock);
        let parent_value_2 = self.parents.2.next(clock);
        let parent_value_3 = self.parents.3.next(clock);

        for channel in 0..CHANNELS {
            final_value[channel] += parent_value_0[channel]
                + parent_value_1[channel]
                + parent_value_2[channel]
                + parent_value_3[channel];
        }

        final_value
    }

    fn reset(&mut self) {
        self.parents.0.reset();
        self.parents.1.reset();
        self.parents.2.reset();
        self.parents.3.reset();
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3, T4, T5> PipelineNode<CHANNELS, Data>
    for Combiner<CHANNELS, Data, (T1, T2, T3, T4, T5)>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
    T4: PipelineNode<CHANNELS, Data>,
    T5: PipelineNode<CHANNELS, Data>,
{
    fn update(&mut self, data: &Data) {
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
        let mut final_value = [0.0; CHANNELS];

        let parent_value_0 = self.parents.0.next(clock);
        let parent_value_1 = self.parents.1.next(clock);
        let parent_value_2 = self.parents.2.next(clock);
        let parent_value_3 = self.parents.3.next(clock);
        let parent_value_4 = self.parents.4.next(clock);

        for channel in 0..CHANNELS {
            final_value[channel] += parent_value_0[channel]
                + parent_value_1[channel]
                + parent_value_2[channel]
                + parent_value_3[channel]
                + parent_value_4[channel];
        }

        final_value
    }

    fn reset(&mut self) {
        self.parents.0.reset();
        self.parents.1.reset();
        self.parents.2.reset();
        self.parents.3.reset();
        self.parents.4.reset();
    }
}

impl<const CHANNELS: usize, Data, T1, T2, T3, T4, T5, T6> PipelineNode<CHANNELS, Data>
    for Combiner<CHANNELS, Data, (T1, T2, T3, T4, T5, T6)>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
    T4: PipelineNode<CHANNELS, Data>,
    T5: PipelineNode<CHANNELS, Data>,
    T6: PipelineNode<CHANNELS, Data>,
{
    fn update(&mut self, data: &Data) {
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
        let mut final_value = [0.0; CHANNELS];

        let parent_value_0 = self.parents.0.next(clock);
        let parent_value_1 = self.parents.1.next(clock);
        let parent_value_2 = self.parents.2.next(clock);
        let parent_value_3 = self.parents.3.next(clock);
        let parent_value_4 = self.parents.4.next(clock);
        let parent_value_5 = self.parents.5.next(clock);

        for channel in 0..CHANNELS {
            final_value[channel] += parent_value_0[channel]
                + parent_value_1[channel]
                + parent_value_2[channel]
                + parent_value_3[channel]
                + parent_value_4[channel]
                + parent_value_5[channel];
        }

        final_value
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

impl<const CHANNELS: usize, Data, T1, T2, T3, T4, T5, T6, T7> PipelineNode<CHANNELS, Data>
    for Combiner<CHANNELS, Data, (T1, T2, T3, T4, T5, T6, T7)>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
    T4: PipelineNode<CHANNELS, Data>,
    T5: PipelineNode<CHANNELS, Data>,
    T6: PipelineNode<CHANNELS, Data>,
    T7: PipelineNode<CHANNELS, Data>,
{
    fn update(&mut self, data: &Data) {
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
        let mut final_value = [0.0; CHANNELS];

        let parent_value_0 = self.parents.0.next(clock);
        let parent_value_1 = self.parents.1.next(clock);
        let parent_value_2 = self.parents.2.next(clock);
        let parent_value_3 = self.parents.3.next(clock);
        let parent_value_4 = self.parents.4.next(clock);
        let parent_value_5 = self.parents.5.next(clock);
        let parent_value_6 = self.parents.6.next(clock);

        for channel in 0..CHANNELS {
            final_value[channel] += parent_value_0[channel]
                + parent_value_1[channel]
                + parent_value_2[channel]
                + parent_value_3[channel]
                + parent_value_4[channel]
                + parent_value_5[channel]
                + parent_value_6[channel];
        }

        final_value
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

impl<const CHANNELS: usize, Data, T1, T2, T3, T4, T5, T6, T7, T8> PipelineNode<CHANNELS, Data>
    for Combiner<CHANNELS, Data, (T1, T2, T3, T4, T5, T6, T7, T8)>
where
    T1: PipelineNode<CHANNELS, Data>,
    T2: PipelineNode<CHANNELS, Data>,
    T3: PipelineNode<CHANNELS, Data>,
    T4: PipelineNode<CHANNELS, Data>,
    T5: PipelineNode<CHANNELS, Data>,
    T6: PipelineNode<CHANNELS, Data>,
    T7: PipelineNode<CHANNELS, Data>,
    T8: PipelineNode<CHANNELS, Data>,
{
    fn update(&mut self, data: &Data) {
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
        let mut final_value = [0.0; CHANNELS];

        let parent_value_0 = self.parents.0.next(clock);
        let parent_value_1 = self.parents.1.next(clock);
        let parent_value_2 = self.parents.2.next(clock);
        let parent_value_3 = self.parents.3.next(clock);
        let parent_value_4 = self.parents.4.next(clock);
        let parent_value_5 = self.parents.5.next(clock);
        let parent_value_6 = self.parents.6.next(clock);
        let parent_value_7 = self.parents.7.next(clock);

        for channel in 0..CHANNELS {
            final_value[channel] += parent_value_0[channel]
                + parent_value_1[channel]
                + parent_value_2[channel]
                + parent_value_3[channel]
                + parent_value_4[channel]
                + parent_value_5[channel]
                + parent_value_6[channel]
                + parent_value_7[channel];
        }

        final_value
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

impl<const CHANNELS: usize, Data, T1, T2, T3, T4, T5, T6, T7, T8, T9> PipelineNode<CHANNELS, Data>
    for Combiner<CHANNELS, Data, (T1, T2, T3, T4, T5, T6, T7, T8, T9)>
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
{
    fn update(&mut self, data: &Data) {
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
        let mut final_value = [0.0; CHANNELS];

        let parent_value_0 = self.parents.0.next(clock);
        let parent_value_1 = self.parents.1.next(clock);
        let parent_value_2 = self.parents.2.next(clock);
        let parent_value_3 = self.parents.3.next(clock);
        let parent_value_4 = self.parents.4.next(clock);
        let parent_value_5 = self.parents.5.next(clock);
        let parent_value_6 = self.parents.6.next(clock);
        let parent_value_7 = self.parents.7.next(clock);
        let parent_value_8 = self.parents.8.next(clock);

        for channel in 0..CHANNELS {
            final_value[channel] += parent_value_0[channel]
                + parent_value_1[channel]
                + parent_value_2[channel]
                + parent_value_3[channel]
                + parent_value_4[channel]
                + parent_value_5[channel]
                + parent_value_6[channel]
                + parent_value_7[channel]
                + parent_value_8[channel];
        }

        final_value
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

impl<const CHANNELS: usize, Data, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10>
    PipelineNode<CHANNELS, Data>
    for Combiner<CHANNELS, Data, (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10)>
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
{
    fn update(&mut self, data: &Data) {
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
        let mut final_value = [0.0; CHANNELS];

        let parent_value_0 = self.parents.0.next(clock);
        let parent_value_1 = self.parents.1.next(clock);
        let parent_value_2 = self.parents.2.next(clock);
        let parent_value_3 = self.parents.3.next(clock);
        let parent_value_4 = self.parents.4.next(clock);
        let parent_value_5 = self.parents.5.next(clock);
        let parent_value_6 = self.parents.6.next(clock);
        let parent_value_7 = self.parents.7.next(clock);
        let parent_value_8 = self.parents.8.next(clock);
        let parent_value_9 = self.parents.9.next(clock);

        for channel in 0..CHANNELS {
            final_value[channel] += parent_value_0[channel]
                + parent_value_1[channel]
                + parent_value_2[channel]
                + parent_value_3[channel]
                + parent_value_4[channel]
                + parent_value_5[channel]
                + parent_value_6[channel]
                + parent_value_7[channel]
                + parent_value_8[channel]
                + parent_value_9[channel];
        }

        final_value
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
