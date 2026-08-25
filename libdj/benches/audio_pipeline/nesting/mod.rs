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

#[allow(clippy::too_many_lines)]
pub fn test_nesting_inlining(
    criterion: &mut Criterion,
    deck_state: &DeckState,
    deck_update_results: &DeckUpdateResults,
    loaded_tracks: &[Rc<RefCell<Option<TrackAudioData>>>; 4],
) {
    use libdsp::pipeline::nesting_inlining::{
        PipelineNode,
        nodes::{
            filters::{
                biquad::{BiquadFilter, FilterConfiguration},
                stretcher::Stretcher,
            },
            generation::{
                reset_control::ResetInjector,
                track_reader::{TrackReader, TrackReaderSourceTimecode},
            },
            mapping::{combiner::Combiner, data_mapper::DataMapper, splitter::Splitter},
            mixing::{
                crossfader::CrossFader,
                fader::Fader,
                gain::{Gain, GainMode},
            },
            resampler::Resampler,
        },
    };

    let channels: [(
        DataMapper<2, PipelineChannelData<'_>, PipelineDeckData<'_>, _, _>,
        _,
    ); 4] = core::array::from_fn::<_, 4, _>(|index| {
        let loaded_track_data_ref = loaded_tracks.get(index).unwrap().clone();

        let master_audio = TrackReader::new(
            loaded_track_data_ref.clone(),
            |data: &PipelineChannelData| {
                Some(TrackReaderSourceTimecode {
                    start_time: data.channel_update_results.player.playback_frame_start_time,
                    end_time: data.channel_update_results.player.playback_frame_end_time,

                    wrap_times: data.channel_update_results.player.playback_wrap_times,
                })
            },
        );
        let master_audio = Stretcher::new(
            master_audio,
            |data: &PipelineChannelData| data.track_sample_rate,
            |data: &PipelineChannelData| data.track_sample_rate,
            |_: &PipelineChannelData| 0.0,
        );
        let master_audio = Resampler::new(
            master_audio,
            |data: &PipelineChannelData| data.track_sample_rate,
            |data: &PipelineChannelData| data.output_sample_rate,
        );
        let master_audio = Gain::new(
            master_audio,
            GainMode::<_, DummyExtractorF32, _>::Decibel(|data: &PipelineChannelData| {
                data.channel_state.gain
            }),
        );
        let master_audio = BiquadFilter::new(
            master_audio,
            FilterConfiguration::<_, _, _, DummyExtractorF64, _, DummyExtractorF64>::LowShelf(
                |data: &PipelineChannelData| {
                    MIXER_EQ_LOW_CUTOFF / f64::from(data.output_sample_rate)
                },
                |data: &PipelineChannelData| f64::from(data.channel_state.eq.0 + 1.0),
                |_: &PipelineChannelData| MIXER_EQ_LOW_OCTAVES,
            ),
        );
        let master_audio = BiquadFilter::new(
            master_audio,
            FilterConfiguration::<_, _, DummyExtractorF64, _, _, DummyExtractorF64>::PeakQ(
                |data: &PipelineChannelData| {
                    MIXER_EQ_MID_CENTER / f64::from(data.output_sample_rate)
                },
                |data: &PipelineChannelData| f64::from(data.channel_state.eq.1 + 1.0),
                |_: &PipelineChannelData| MIXER_EQ_MID_Q,
            ),
        );
        let master_audio = BiquadFilter::new(
            master_audio,
            FilterConfiguration::<_, _, _, DummyExtractorF64, _, DummyExtractorF64>::HighShelf(
                |data: &PipelineChannelData| {
                    MIXER_EQ_HIGH_CUTOFF / f64::from(data.output_sample_rate)
                },
                |data: &PipelineChannelData| f64::from(data.channel_state.eq.2 + 1.0),
                |_: &PipelineChannelData| MIXER_EQ_HIGH_OCTAVES,
            ),
        );

        let [master_audio_a, master_audio_b]: [Splitter<2, 2, PipelineChannelData, _>; 2] =
            Splitter::new(master_audio);

        let channel_master = Fader::new(master_audio_a, |data: &PipelineChannelData| {
            data.channel_state.fade
        });
        let channel_master = CrossFader::new(
            channel_master,
            |data: &PipelineChannelData| data.crossfade,
            |data: &PipelineChannelData| data.channel_state.cross_fader_side,
        );

        let touch_cue_audio =
            TrackReader::new(loaded_track_data_ref, |data: &PipelineChannelData| {
                data.channel_update_results
                    .player
                    .touch_cue_playback_times
                    .map(|(start_time, end_time)| TrackReaderSourceTimecode {
                        start_time,
                        end_time,

                        wrap_times: None,
                    })
            });
        let touch_cue_audio = Stretcher::new(
            touch_cue_audio,
            |data: &PipelineChannelData| data.track_sample_rate,
            |data: &PipelineChannelData| data.track_sample_rate,
            |_: &PipelineChannelData| 0.0,
        );
        let touch_cue_audio = Resampler::new(
            touch_cue_audio,
            |data: &PipelineChannelData| data.track_sample_rate,
            |data: &PipelineChannelData| data.output_sample_rate,
        );
        let touch_cue_audio = Gain::new(
            touch_cue_audio,
            GainMode::<_, DummyExtractorF32, _>::Decibel(|data: &PipelineChannelData| {
                data.channel_state.gain
            }),
        );
        let touch_cue_audio = BiquadFilter::new(
            touch_cue_audio,
            FilterConfiguration::<_, _, _, DummyExtractorF64, _, DummyExtractorF64>::LowShelf(
                |data: &PipelineChannelData| {
                    MIXER_EQ_LOW_CUTOFF / f64::from(data.output_sample_rate)
                },
                |data: &PipelineChannelData| f64::from(data.channel_state.eq.0 + 1.0),
                |_: &PipelineChannelData| MIXER_EQ_LOW_OCTAVES,
            ),
        );
        let touch_cue_audio = BiquadFilter::new(
            touch_cue_audio,
            FilterConfiguration::<_, _, DummyExtractorF64, _, _, DummyExtractorF64>::PeakQ(
                |data: &PipelineChannelData| {
                    MIXER_EQ_MID_CENTER / f64::from(data.output_sample_rate)
                },
                |data: &PipelineChannelData| f64::from(data.channel_state.eq.1 + 1.0),
                |_: &PipelineChannelData| MIXER_EQ_MID_Q,
            ),
        );
        let touch_cue_audio = BiquadFilter::new(
            touch_cue_audio,
            FilterConfiguration::<_, _, _, DummyExtractorF64, _, DummyExtractorF64>::HighShelf(
                |data: &PipelineChannelData| {
                    MIXER_EQ_HIGH_CUTOFF / f64::from(data.output_sample_rate)
                },
                |data: &PipelineChannelData| f64::from(data.channel_state.eq.2 + 1.0),
                |_: &PipelineChannelData| MIXER_EQ_HIGH_OCTAVES,
            ),
        );

        let channel_cue = Combiner::<2, _, (_, _)>::new((master_audio_b, touch_cue_audio));

        let channel_cue = ResetInjector::new(channel_cue, |data: &PipelineChannelData| {
            data.track_change_signal.try_recv().is_ok()
        });

        (
            DataMapper::new(channel_master, move |data: &PipelineDeckData| {
                PipelineChannelData {
                    output_sample_rate: data.output_sample_rate,

                    track_sample_rate: *data.track_sample_rates.get(index).unwrap(),
                    track_change_signal: data.track_change_signals.get(index).unwrap(),

                    channel_state: data.deck_state.mixer_channels.get(index).unwrap(),
                    channel_update_results: data.deck_update_results.channels.get(index).unwrap(),

                    crossfade: data.deck_state.crossfade,
                }
            }),
            DataMapper::new(channel_cue, move |data: &PipelineDeckData| {
                PipelineChannelData {
                    output_sample_rate: data.output_sample_rate,

                    track_sample_rate: *data.track_sample_rates.get(index).unwrap(),
                    track_change_signal: data.track_change_signals.get(index).unwrap(),

                    channel_state: data.deck_state.mixer_channels.get(index).unwrap(),
                    channel_update_results: data.deck_update_results.channels.get(index).unwrap(),

                    crossfade: data.deck_state.crossfade,
                }
            }),
        )
    });

    let [
        (master_a, cue_a),
        (master_b, cue_b),
        (master_c, cue_c),
        (master_d, cue_d),
    ]: [(
        DataMapper<2, PipelineChannelData<'_>, PipelineDeckData<'_>, _, _>,
        _,
    ); 4] = channels;

    let mut master_pipeline =
        Combiner::<2, _, (_, _, _, _)>::new((master_a, master_b, master_c, master_d));
    let mut cue_pipeline = Combiner::<2, _, (_, _, _, _)>::new((cue_a, cue_b, cue_c, cue_d));

    let source_sample_rates: [u32; 4] = core::array::from_fn::<u32, 4, _>(|index| {
        loaded_tracks
            .get(index)
            .unwrap()
            .borrow()
            .as_ref()
            .map_or(0, |track_audio_data| track_audio_data.sample_rate)
    });

    let (sender_a, receiver_a) = std::sync::mpsc::channel();
    let (sender_b, receiver_b) = std::sync::mpsc::channel();
    let (sender_c, receiver_c) = std::sync::mpsc::channel();
    let (sender_d, receiver_d) = std::sync::mpsc::channel();

    let _ = sender_a.send(());
    let _ = sender_b.send(());
    let _ = sender_c.send(());
    let _ = sender_d.send(());

    master_pipeline.reset();
    cue_pipeline.reset();

    let data = PipelineDeckData {
        output_sample_rate: OUTPUT_SAMPLE_RATE,

        track_sample_rates: source_sample_rates,
        track_change_signals: [&receiver_a, &receiver_b, &receiver_c, &receiver_d],

        deck_state,
        deck_update_results,
    };

    master_pipeline.update(&data);
    cue_pipeline.update(&data);

    let mut master_output_data = Vec::<[f32; 2]>::with_capacity(VIRTUAL_BATCH_SIZE);
    let mut cue_output_data = Vec::<[f32; 2]>::with_capacity(VIRTUAL_BATCH_SIZE);

    criterion.bench_function("nesting inlining", |b| {
        b.iter(|| {
            for (clock, (master_samples, cue_samples)) in master_output_data
                .iter_mut()
                .zip(&mut cue_output_data)
                .enumerate()
            {
                *master_samples = master_pipeline.next(clock);
                *cue_samples = cue_pipeline.next(clock);
            }
        });
    });
}
