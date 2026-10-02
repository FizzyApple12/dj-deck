use std::marker::PhantomData;

use crate::pipeline::nesting_inlining::{DataExtractor, PipelineNode};

pub struct Resampler<
    const CHANNELS: usize,
    Data,
    Parent,
    InputSampleRateDataExtractor,
    OutputSampleRateDataExtractor,
> where
    Parent: PipelineNode<CHANNELS, Data>,
    InputSampleRateDataExtractor: DataExtractor<Data, u32>,
    OutputSampleRateDataExtractor: DataExtractor<Data, u32>,
{
    phantom_data: PhantomData<Data>,
    parent: Parent,

    parent_clock: usize,

    input_sample_rate_data_extractor: InputSampleRateDataExtractor,
    output_sample_rate_data_extractor: OutputSampleRateDataExtractor,

    ratio: f32,
    rolling_fractional: f32,

    history: [[f32; CHANNELS]; 4],
}

impl<
    const CHANNELS: usize,
    Data,
    Parent,
    InputSampleRateDataExtractor,
    OutputSampleRateDataExtractor,
> Resampler<CHANNELS, Data, Parent, InputSampleRateDataExtractor, OutputSampleRateDataExtractor>
where
    Parent: PipelineNode<CHANNELS, Data>,
    InputSampleRateDataExtractor: DataExtractor<Data, u32>,
    OutputSampleRateDataExtractor: DataExtractor<Data, u32>,
{
    pub fn new(
        parent: Parent,
        input_sample_rate_data_extractor: InputSampleRateDataExtractor,
        output_sample_rate_data_extractor: OutputSampleRateDataExtractor,
    ) -> Self {
        Self {
            phantom_data: PhantomData,
            parent,

            parent_clock: 0,

            input_sample_rate_data_extractor,
            output_sample_rate_data_extractor,

            ratio: 1.0,
            rolling_fractional: 0.0,

            history: [[0.0; CHANNELS]; 4],
        }
    }
}

impl<
    const CHANNELS: usize,
    Data,
    Parent,
    SourceSampleRateDataExtractor,
    DestinationSampleRateDataExtractor,
> PipelineNode<CHANNELS, Data>
    for Resampler<
        CHANNELS,
        Data,
        Parent,
        SourceSampleRateDataExtractor,
        DestinationSampleRateDataExtractor,
    >
where
    Parent: PipelineNode<CHANNELS, Data>,
    SourceSampleRateDataExtractor: DataExtractor<Data, u32>,
    DestinationSampleRateDataExtractor: DataExtractor<Data, u32>,
{
    #[allow(clippy::cast_precision_loss)]
    fn update(&mut self, data: &Data) {
        let input_sample_rate = (self.input_sample_rate_data_extractor)(data);
        let output_sample_rate = (self.output_sample_rate_data_extractor)(data);

        self.ratio = input_sample_rate as f32 / output_sample_rate as f32;

        self.parent.update(data);
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss
    )]
    fn next(&mut self, _: usize) -> [f32; CHANNELS] {
        let phase = self.rolling_fractional + self.ratio;
        let advance = phase.floor() as usize;
        self.rolling_fractional = phase - advance as f32;

        for _ in 0..advance {
            self.history
                .shift_left([self.parent.next(self.parent_clock)]);

            self.parent_clock = self.parent_clock.wrapping_add(1);
        }

        let [a, b, c, d] = self.history;
        let fractional = self.rolling_fractional;

        let mut output = [0.0; CHANNELS];

        for ((((data, a), b), c), d) in output.iter_mut().zip(a).zip(b).zip(c).zip(d) {
            let cb_diff = c - b;

            let k1 = (c - a) * 0.5;
            let k3 = k1 + (d - b) * 0.5 - cb_diff * 2.0;
            let k2 = cb_diff - k3 - k1;

            *data = b + fractional * (k1 + fractional * (k2 + fractional * k3));
        }

        output
    }

    fn reset(&mut self) {
        self.rolling_fractional = 0.0;

        self.history = [[0.0; CHANNELS]; 4];

        self.parent.reset();
    }
}
