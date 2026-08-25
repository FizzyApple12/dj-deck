use std::marker::PhantomData;

use crate::pipeline::nesting_inlining::{DataExtractor, PipelineNode};

pub enum FilterConfiguration<
    Data,
    ScaledFrequencyDataExtractor,
    OctaveTargetDataExtractor,
    QFactorDataExtractor,
    GainDataExtractor,
    DecibelGainDataExtractor,
> {
    Disabled(PhantomData<Data>),
    LowPass(ScaledFrequencyDataExtractor, OctaveTargetDataExtractor),
    LowPassQ(ScaledFrequencyDataExtractor, QFactorDataExtractor),
    HighPass(ScaledFrequencyDataExtractor, OctaveTargetDataExtractor),
    HighPassQ(ScaledFrequencyDataExtractor, QFactorDataExtractor),
    BandPass(ScaledFrequencyDataExtractor, OctaveTargetDataExtractor),
    BandPassQ(ScaledFrequencyDataExtractor, QFactorDataExtractor),
    AllPass(ScaledFrequencyDataExtractor, OctaveTargetDataExtractor),
    AllPassQ(ScaledFrequencyDataExtractor, QFactorDataExtractor),
    Notch(ScaledFrequencyDataExtractor, OctaveTargetDataExtractor),
    NotchQ(ScaledFrequencyDataExtractor, QFactorDataExtractor),
    Peak(
        ScaledFrequencyDataExtractor,
        GainDataExtractor,
        OctaveTargetDataExtractor,
    ),
    PeakDB(
        ScaledFrequencyDataExtractor,
        DecibelGainDataExtractor,
        OctaveTargetDataExtractor,
    ),
    PeakQ(
        ScaledFrequencyDataExtractor,
        GainDataExtractor,
        QFactorDataExtractor,
    ),
    PeakDBQ(
        ScaledFrequencyDataExtractor,
        DecibelGainDataExtractor,
        QFactorDataExtractor,
    ),
    HighShelf(
        ScaledFrequencyDataExtractor,
        GainDataExtractor,
        OctaveTargetDataExtractor,
    ),
    HighShelfDB(
        ScaledFrequencyDataExtractor,
        DecibelGainDataExtractor,
        OctaveTargetDataExtractor,
    ),
    HighShelfQ(
        ScaledFrequencyDataExtractor,
        GainDataExtractor,
        QFactorDataExtractor,
    ),
    HighShelfDBQ(
        ScaledFrequencyDataExtractor,
        DecibelGainDataExtractor,
        QFactorDataExtractor,
    ),
    LowShelf(
        ScaledFrequencyDataExtractor,
        GainDataExtractor,
        OctaveTargetDataExtractor,
    ),
    LowShelfDB(
        ScaledFrequencyDataExtractor,
        DecibelGainDataExtractor,
        OctaveTargetDataExtractor,
    ),
    LowShelfQ(
        ScaledFrequencyDataExtractor,
        GainDataExtractor,
        QFactorDataExtractor,
    ),
    LowShelfDBQ(
        ScaledFrequencyDataExtractor,
        DecibelGainDataExtractor,
        QFactorDataExtractor,
    ),
}

pub struct BiquadFilter<
    const CHANNELS: usize,
    Data,
    Parent,
    ScaledFrequencyDataExtractor,
    OctaveTargetDataExtractor,
    QFactorDataExtractor,
    GainDataExtractor,
    DecibelGainDataExtractor,
> where
    Parent: PipelineNode<CHANNELS, Data>,
    ScaledFrequencyDataExtractor: DataExtractor<Data, f64>,
    OctaveTargetDataExtractor: DataExtractor<Data, f64>,
    QFactorDataExtractor: DataExtractor<Data, f64>,
    GainDataExtractor: DataExtractor<Data, f64>,
    DecibelGainDataExtractor: DataExtractor<Data, f64>,
{
    phantom_data: PhantomData<Data>,
    parent: Parent,
    configuration: FilterConfiguration<
        Data,
        ScaledFrequencyDataExtractor,
        OctaveTargetDataExtractor,
        QFactorDataExtractor,
        GainDataExtractor,
        DecibelGainDataExtractor,
    >,

    a1: f32,
    a2: f32,

    b0: f32,
    b1: f32,
    b2: f32,

    x1: [f32; CHANNELS],
    x2: [f32; CHANNELS],
    y1: [f32; CHANNELS],
    y2: [f32; CHANNELS],
}

impl<
    const CHANNELS: usize,
    Data,
    Parent,
    ScaledFrequencyDataExtractor,
    OctaveTargetDataExtractor,
    QFactorDataExtractor,
    GainDataExtractor,
    DecibelGainDataExtractor,
>
    BiquadFilter<
        CHANNELS,
        Data,
        Parent,
        ScaledFrequencyDataExtractor,
        OctaveTargetDataExtractor,
        QFactorDataExtractor,
        GainDataExtractor,
        DecibelGainDataExtractor,
    >
where
    Parent: PipelineNode<CHANNELS, Data>,
    ScaledFrequencyDataExtractor: DataExtractor<Data, f64>,
    OctaveTargetDataExtractor: DataExtractor<Data, f64>,
    QFactorDataExtractor: DataExtractor<Data, f64>,
    GainDataExtractor: DataExtractor<Data, f64>,
    DecibelGainDataExtractor: DataExtractor<Data, f64>,
{
    pub fn new(
        parent: Parent,
        configuration: FilterConfiguration<
            Data,
            ScaledFrequencyDataExtractor,
            OctaveTargetDataExtractor,
            QFactorDataExtractor,
            GainDataExtractor,
            DecibelGainDataExtractor,
        >,
    ) -> Self {
        Self {
            phantom_data: PhantomData,
            parent,
            configuration,

            a1: 0.0,
            a2: 0.0,

            b0: 0.0,
            b1: 0.0,
            b2: 0.0,

            x1: [0.0; CHANNELS],
            x2: [0.0; CHANNELS],
            y1: [0.0; CHANNELS],
            y2: [0.0; CHANNELS],
        }
    }
}

impl<
    const CHANNELS: usize,
    Data,
    Parent,
    ScaledFrequencyDataExtractor,
    OctaveTargetDataExtractor,
    QFactorDataExtractor,
    GainDataExtractor,
    DecibelGainDataExtractor,
> PipelineNode<CHANNELS, Data>
    for BiquadFilter<
        CHANNELS,
        Data,
        Parent,
        ScaledFrequencyDataExtractor,
        OctaveTargetDataExtractor,
        QFactorDataExtractor,
        GainDataExtractor,
        DecibelGainDataExtractor,
    >
where
    Parent: PipelineNode<CHANNELS, Data>,
    ScaledFrequencyDataExtractor: DataExtractor<Data, f64>,
    OctaveTargetDataExtractor: DataExtractor<Data, f64>,
    QFactorDataExtractor: DataExtractor<Data, f64>,
    GainDataExtractor: DataExtractor<Data, f64>,
    DecibelGainDataExtractor: DataExtractor<Data, f64>,
{
    fn update(&mut self, data: &Data) {
        match self.configuration {
            FilterConfiguration::Disabled(_) => {}
            FilterConfiguration::LowPass(_, _) => {}
            FilterConfiguration::LowPassQ(_, _) => {}
            FilterConfiguration::HighPass(_, _) => {}
            FilterConfiguration::HighPassQ(_, _) => {}
            FilterConfiguration::BandPass(_, _) => {}
            FilterConfiguration::BandPassQ(_, _) => {}
            FilterConfiguration::AllPass(_, _) => {}
            FilterConfiguration::AllPassQ(_, _) => {}
            FilterConfiguration::Notch(_, _) => {}
            FilterConfiguration::NotchQ(_, _) => {}
            FilterConfiguration::Peak(_, _, _) => {}
            FilterConfiguration::PeakDB(_, _, _) => {}
            FilterConfiguration::PeakQ(_, _, _) => {}
            FilterConfiguration::PeakDBQ(_, _, _) => {}
            FilterConfiguration::HighShelf(_, _, _) => {}
            FilterConfiguration::HighShelfDB(_, _, _) => {}
            FilterConfiguration::HighShelfQ(_, _, _) => {}
            FilterConfiguration::HighShelfDBQ(_, _, _) => {}
            FilterConfiguration::LowShelf(_, _, _) => {}
            FilterConfiguration::LowShelfDB(_, _, _) => {}
            FilterConfiguration::LowShelfQ(_, _, _) => {}
            FilterConfiguration::LowShelfDBQ(_, _, _) => {}
        }
    }

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, clock: usize) -> [f32; CHANNELS] {
        let mut parent_data = self.parent.next(clock);

        for ((((input, x1), x2), y1), y2) in parent_data
            .iter_mut()
            .zip(&mut self.x1)
            .zip(&mut self.x2)
            .zip(&mut self.y1)
            .zip(&mut self.y2)
        {
            let y0 =
                *input * self.b0 + *x1 * self.b1 + *x2 * self.b2 - *y1 * self.a1 - *y2 * self.a2;

            *y2 = *y1;
            *y1 = y0;
            *x2 = *x1;
            *x1 = *input;

            *input = y0;
        }

        parent_data
    }

    fn reset(&mut self) {
        self.x1 = [0.0; CHANNELS];
        self.x2 = [0.0; CHANNELS];
        self.y1 = [0.0; CHANNELS];
        self.y2 = [0.0; CHANNELS];

        self.parent.reset();
    }
}
