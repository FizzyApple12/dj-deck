use std::{cell::RefCell, rc::Rc};

use criterion::Criterion;
use libdj::{
    MIXER_EQ_HIGH_CUTOFF, MIXER_EQ_HIGH_OCTAVES, MIXER_EQ_LOW_CUTOFF, MIXER_EQ_LOW_OCTAVES,
    MIXER_EQ_MID_CENTER, MIXER_EQ_MID_Q, playback::DeckUpdateResults, types::deck::DeckState,
};
use libdsp::audio_loader::TrackAudioData;

use crate::{
    DummyExtractorF32, DummyExtractorF64, OUTPUT_SAMPLE_RATE, PipelineChannelData,
    PipelineDeckData, VIRTUAL_BATCH_SIZE,
};

pub fn test_arena_inlining(
    criterion: &mut Criterion,
    deck_state: &DeckState,
    deck_update_results: &DeckUpdateResults,
    loaded_tracks: &[Rc<RefCell<Option<TrackAudioData>>>; 4],
) {
    use libdsp::pipeline::arena_inlining::{
        PipelineEngine, PipelineResetSpecifier,
        nodes::{
            Null,
            filters::{
                biquad::{BiquadFilter, FilterConfiguration},
                stretcher::Stretcher,
            },
            generation::track_reader::{TrackReader, TrackReaderSourceTimecode},
            mixing::{
                crossfader::CrossFader,
                fader::Fader,
                gain::{Gain, GainMode},
            },
            resampler::Resampler,
        },
    };

    let (_, receiver_a) = std::sync::mpsc::channel();
    let (_, receiver_b) = std::sync::mpsc::channel();
    let (_, receiver_c) = std::sync::mpsc::channel();
    let (_, receiver_d) = std::sync::mpsc::channel();

    let mut pipeline_builder = PipelineEngine::<2, PipelineDeckData>::default();

    let channels = core::array::from_fn::<_, 4, _>(|index| {
        let loaded_track_data_ref = loaded_tracks.get(index).unwrap().clone();

        let master_base = pipeline_builder.add::<0>(
            &[],
            TrackReader::new(
                loaded_track_data_ref.clone(),
                move |data: &PipelineDeckData| {
                    Some(TrackReaderSourceTimecode {
                        start_time: data
                            .deck_update_results
                            .channels
                            .get(index)
                            .unwrap()
                            .player
                            .playback_frame_start_time,
                        end_time: data
                            .deck_update_results
                            .channels
                            .get(index)
                            .unwrap()
                            .player
                            .playback_frame_end_time,

                        wrap_times: data
                            .deck_update_results
                            .channels
                            .get(index)
                            .unwrap()
                            .player
                            .playback_wrap_times,
                    })
                },
            ),
        );
        let mut reset_domain = PipelineResetSpecifier::new(&master_base);
        let master_base = pipeline_builder.add::<1>(
            &[master_base],
            Stretcher::new(
                move |data: &PipelineDeckData| *data.track_sample_rates.get(index).unwrap(),
                move |data: &PipelineDeckData| *data.track_sample_rates.get(index).unwrap(),
                |_: &PipelineDeckData| 0.0,
            ),
        );
        reset_domain.extend(&master_base);
        let master_base = pipeline_builder.add::<1>(
            &[master_base],
            Resampler::new(
                move |data: &PipelineDeckData| *data.track_sample_rates.get(index).unwrap(),
                move |data: &PipelineDeckData| data.output_sample_rate,
            ),
        );
        reset_domain.extend(&master_base);
        let master_base = pipeline_builder.add::<1>(
            &[master_base],
            Gain::new(GainMode::<_, DummyExtractorF32, _>::Decibel(
                move |data: &PipelineDeckData| {
                    data.deck_state.mixer_channels.get(index).unwrap().gain
                },
            )),
        );
        reset_domain.extend(&master_base);
        let master_base = pipeline_builder.add::<1>(
            &[master_base],
            BiquadFilter::new(FilterConfiguration::<
                _,
                _,
                _,
                DummyExtractorF64,
                _,
                DummyExtractorF64,
            >::LowShelf(
                |data: &PipelineDeckData| MIXER_EQ_LOW_CUTOFF / f64::from(data.output_sample_rate),
                move |data: &PipelineDeckData| {
                    f64::from(data.deck_state.mixer_channels.get(index).unwrap().eq.0 + 1.0)
                },
                |_: &PipelineDeckData| MIXER_EQ_LOW_OCTAVES,
            )),
        );
        reset_domain.extend(&master_base);
        let master_base = pipeline_builder.add::<1>(
            &[master_base],
            BiquadFilter::new(FilterConfiguration::<
                _,
                _,
                DummyExtractorF64,
                _,
                _,
                DummyExtractorF64,
            >::PeakQ(
                |data: &PipelineDeckData| MIXER_EQ_MID_CENTER / f64::from(data.output_sample_rate),
                move |data: &PipelineDeckData| {
                    f64::from(data.deck_state.mixer_channels.get(index).unwrap().eq.1 + 1.0)
                },
                |_: &PipelineDeckData| MIXER_EQ_MID_Q,
            )),
        );
        reset_domain.extend(&master_base);
        let master_base = pipeline_builder.add::<1>(
            &[master_base],
            BiquadFilter::new(FilterConfiguration::<
                _,
                _,
                _,
                DummyExtractorF64,
                _,
                DummyExtractorF64,
            >::HighShelf(
                |data: &PipelineDeckData| MIXER_EQ_HIGH_CUTOFF / f64::from(data.output_sample_rate),
                move |data: &PipelineDeckData| {
                    f64::from(data.deck_state.mixer_channels.get(index).unwrap().eq.2 + 1.0)
                },
                |_: &PipelineDeckData| MIXER_EQ_HIGH_OCTAVES,
            )),
        );
        reset_domain.extend(&master_base);

        let master = pipeline_builder.add::<1>(
            &[master_base],
            Fader::new(move |data: &PipelineDeckData| {
                data.deck_state.mixer_channels.get(index).unwrap().fade
            }),
        );
        let master = pipeline_builder.add::<1>(
            &[master],
            CrossFader::new(
                |data: &PipelineDeckData| data.deck_state.crossfade,
                move |data: &PipelineDeckData| {
                    data.deck_state
                        .mixer_channels
                        .get(index)
                        .unwrap()
                        .cross_fader_side
                },
            ),
        );

        let touch_cue_base = pipeline_builder.add::<0>(
            &[],
            TrackReader::new(loaded_track_data_ref, move |data: &PipelineDeckData| {
                data.deck_update_results
                    .channels
                    .get(index)
                    .unwrap()
                    .player
                    .touch_cue_playback_times
                    .map(|(start_time, end_time)| TrackReaderSourceTimecode {
                        start_time,
                        end_time,

                        wrap_times: None,
                    })
            }),
        );
        reset_domain.extend(&touch_cue_base);
        let touch_cue_base = pipeline_builder.add::<1>(
            &[touch_cue_base],
            Stretcher::new(
                move |data: &PipelineDeckData| *data.track_sample_rates.get(index).unwrap(),
                move |data: &PipelineDeckData| *data.track_sample_rates.get(index).unwrap(),
                |_: &PipelineDeckData| 0.0,
            ),
        );
        reset_domain.extend(&master_base);
        let master_base = pipeline_builder.add::<1>(
            &[master_base],
            Resampler::new(
                move |data: &PipelineDeckData| *data.track_sample_rates.get(index).unwrap(),
                move |data: &PipelineDeckData| data.output_sample_rate,
            ),
        );
        reset_domain.extend(&master_base);
        let master_base = pipeline_builder.add::<1>(
            &[master_base],
            Gain::new(GainMode::<_, DummyExtractorF32, _>::Decibel(
                move |data: &PipelineDeckData| {
                    data.deck_state.mixer_channels.get(index).unwrap().gain
                },
            )),
        );
        reset_domain.extend(&touch_cue_base);
        let touch_cue_base = pipeline_builder.add::<1>(
            &[touch_cue_base],
            BiquadFilter::new(FilterConfiguration::<
                _,
                _,
                _,
                DummyExtractorF64,
                _,
                DummyExtractorF64,
            >::LowShelf(
                |data: &PipelineDeckData| MIXER_EQ_LOW_CUTOFF / f64::from(data.output_sample_rate),
                move |data: &PipelineDeckData| {
                    f64::from(data.deck_state.mixer_channels.get(index).unwrap().eq.0 + 1.0)
                },
                |_: &PipelineDeckData| MIXER_EQ_LOW_OCTAVES,
            )),
        );
        reset_domain.extend(&touch_cue_base);
        let touch_cue_base = pipeline_builder.add::<1>(
            &[touch_cue_base],
            BiquadFilter::new(FilterConfiguration::<
                _,
                _,
                DummyExtractorF64,
                _,
                _,
                DummyExtractorF64,
            >::PeakQ(
                |data: &PipelineDeckData| MIXER_EQ_MID_CENTER / f64::from(data.output_sample_rate),
                move |data: &PipelineDeckData| {
                    f64::from(data.deck_state.mixer_channels.get(index).unwrap().eq.1 + 1.0)
                },
                |_: &PipelineDeckData| MIXER_EQ_MID_Q,
            )),
        );
        reset_domain.extend(&touch_cue_base);
        let touch_cue_base = pipeline_builder.add::<1>(
            &[touch_cue_base],
            BiquadFilter::new(FilterConfiguration::<
                _,
                _,
                _,
                DummyExtractorF64,
                _,
                DummyExtractorF64,
            >::HighShelf(
                |data: &PipelineDeckData| MIXER_EQ_HIGH_CUTOFF / f64::from(data.output_sample_rate),
                move |data: &PipelineDeckData| {
                    f64::from(data.deck_state.mixer_channels.get(index).unwrap().eq.2 + 1.0)
                },
                |_: &PipelineDeckData| MIXER_EQ_HIGH_OCTAVES,
            )),
        );
        reset_domain.extend(&touch_cue_base);

        let cue = pipeline_builder.add::<2>(&[master_base, touch_cue_base], Null::default());

        (master, cue, reset_domain)
    });

    let [
        (master_a, cue_a, reset_domain_a),
        (master_b, cue_b, reset_domain_b),
        (master_c, cue_c, reset_domain_c),
        (master_d, cue_d, reset_domain_d),
    ] = channels;

    let master_pipeline =
        pipeline_builder.add::<4>(&[master_a, master_b, master_c, master_d], Null::default());
    let cue_pipeline = pipeline_builder.add::<4>(&[cue_a, cue_b, cue_c, cue_d], Null::default());

    let source_sample_rates: [u32; 4] = core::array::from_fn::<u32, 4, _>(|index| {
        loaded_tracks
            .get(index)
            .unwrap()
            .borrow()
            .as_ref()
            .map_or(0, |track_audio_data| track_audio_data.sample_rate)
    });

    pipeline_builder.reset_segment(&reset_domain_a);
    pipeline_builder.reset_segment(&reset_domain_b);
    pipeline_builder.reset_segment(&reset_domain_c);
    pipeline_builder.reset_segment(&reset_domain_d);

    pipeline_builder.reset();

    let data = PipelineDeckData {
        output_sample_rate: OUTPUT_SAMPLE_RATE,

        track_sample_rates: source_sample_rates,
        track_change_signals: [&receiver_a, &receiver_b, &receiver_c, &receiver_d],

        deck_state,
        deck_update_results,
    };

    pipeline_builder.update(&data);

    let mut master_output_data = Vec::<[f32; 2]>::with_capacity(VIRTUAL_BATCH_SIZE);
    let mut cue_output_data = Vec::<[f32; 2]>::with_capacity(VIRTUAL_BATCH_SIZE);

    criterion.bench_function("arena inlining", |b| {
        b.iter(|| {
            for (clock, (master_samples, cue_samples)) in master_output_data
                .iter_mut()
                .zip(&mut cue_output_data)
                .enumerate()
            {
                *master_samples = pipeline_builder.next(&master_pipeline, clock);
                *cue_samples = pipeline_builder.next(&cue_pipeline, clock);
            }
        });
    });
}
