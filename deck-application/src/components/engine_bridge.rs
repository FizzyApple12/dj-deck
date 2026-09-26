// these are unavoidable when using cxx_qt :sob:
#![allow(clippy::float_cmp, clippy::needless_pass_by_value)]

#[cxx_qt::bridge]
mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qimage.h");
        type QImage = cxx_qt_lib::QImage;
    }

    #[derive(Debug, PartialEq)]
    pub enum BrowserPage {
        Closed,
        Device,
        Playlist,
        Search,
    }

    #[derive(Debug, PartialEq)]
    pub enum TempoRange {
        SixPercent,
        TenPercent,
        SixteenPercent,
        OneHundredPercent,
    }

    #[derive(Debug, PartialEq)]
    pub enum PlayState {
        Stop,
        Play,
        Cue,
    }

    #[derive(Debug, PartialEq)]
    pub enum CrossFaderSide {
        A,
        B,
        None,
    }

    #[derive(Debug, PartialEq)]
    pub enum BeatSyncMode {
        Off,
        BPMSync,
        BeatSync,
    }

    #[derive(Debug, PartialEq)]
    pub enum BeatLoopAdjustMode {
        None,
        In,
        Out,
    }

    #[derive(Debug, PartialEq)]
    pub enum ChannelFXEffect {
        None,
        Space,
        DubEcho,
        Bitcrush,
        Pitch,
        Noise,
        Filter,
    }

    #[derive(Debug, PartialEq)]
    pub struct EngineBridgePlayer {
        pub waveform_image: QImage,
        pub waveform_image_stride: f32,

        pub preview_waveform_image: QImage,

        // pub current_track: Option<(usize, Track)>,
        // pub current_track_analysis: Option<TrackAnalysis>,
        pub is_loading: bool,

        pub beat_sync: BeatSyncMode,
        pub key_sync: bool,

        pub quanitze: bool,

        pub jog_hold: bool,
        pub jog_wait: bool,
        pub jog_velocity: f32,

        pub play_state: PlayState,
        pub time: i64, // timecode us
        pub cue_time_set: bool,
        pub cue_time: i64, // timecode us
        pub touch_cue_time_set: bool,
        pub touch_cue_time: i64, // timecode us

        pub reverse_enabled: bool,

        pub tempo_range: TempoRange,
        pub tempo_reset: bool,  // tempo reset enabled
        pub tempo_percent: f32, // tempo percent
        pub tempo_slider_position: f32,
        pub tempo_slider_is_accurate: bool,
        pub master_tempo: bool, // master tempo enabled

        pub slip: bool,         // slip enabled
        pub slip_playing: bool, // slip playing
        pub slip_time: i64,     // timecode us

        pub beat_loop_start_set: bool,
        pub beat_loop_start: i64, // timecode us
        pub beat_loop_end_set: bool,
        pub beat_loop_end: i64, // timecode us
        pub last_beat_loop_set: bool,
        pub last_beat_loop_start: i64, // timecode us
        pub last_beat_loop_end: i64,   // timecode us
        pub beat_loop_adjust_mode: BeatLoopAdjustMode,

        pub keyshift: f32, // semitones
    }

    #[derive(Debug, PartialEq)]
    pub struct EngineBridgeChannel {
        pub player: EngineBridgePlayer,

        pub gain: f32,    // decibels
        pub eq_low: f32,  // decibels
        pub eq_mid: f32,  // decibels
        pub eq_high: f32, // decibels
        pub fx: f32,      // percent

        pub cue: bool, // cue enabled

        pub fade: f32, // percent

        pub cross_fader_side: CrossFaderSide,
    }

    #[derive(Debug, PartialEq)]
    pub struct EngineBridgeDeck {
        pub mixer_channels: [EngineBridgeChannel; 4],

        pub master_channel_set: bool,
        pub master_channel: usize,

        // pub master_fx: MasterFX,
        pub crossfade: f32,

        pub master_cue: bool,
        pub master_gain: f32, // decibels

        pub channel_fx_effect: ChannelFXEffect,
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, source_open)]
        #[qproperty(BrowserPage, browser_page)]
        #[qproperty(bool, device_selected)]
        #[qproperty(usize, active_device)]
        #[qproperty(EngineBridgeDeck, deck_state)]
        #[qproperty(f64, waveform_pixels_per_second)]
        #[qproperty(bool, mixer_channel_fx_proximity)]
        #[qproperty(bool, mixer_master_fx_proximity)]
        type EngineBridge = super::EngineBridgeRust;

        #[qinvokable]
        fn before_frame(self: Pin<&mut EngineBridge>);

        #[qinvokable]
        fn load_track(self: Pin<&mut EngineBridge>, player: usize, device: usize, track_id: u32);

        #[qinvokable]
        fn eject_track(self: Pin<&mut EngineBridge>, player: usize);

        // #[qinvokable]
        // fn eject_device(self: Pin<&mut EngineBridge>, device: usize);

        #[qinvokable]
        fn touch_cue_move(self: Pin<&mut EngineBridge>, player: usize, position: f32);

        #[qinvokable]
        fn touch_cue_release(self: Pin<&mut EngineBridge>, player: usize);
    }

    impl cxx_qt::Threading for EngineBridge {}

    impl cxx_qt::Constructor<(), NewArguments = ()> for EngineBridge {}
}

use std::{
    pin::Pin,
    sync::{LazyLock, Mutex},
};

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QImage, QImageFormat};
use libdatabase::device_manager::DeviceManager;
use libdj::{
    engine::DJEngine,
    types::{bindings::UIControlEvent, deck::DeckState, playback::DeckUpdate},
};
use libdsp::audio_loader::TrackAudioData;
use log::warn;
use qobject::{
    BeatLoopAdjustMode, BeatSyncMode, BrowserPage, ChannelFXEffect, CrossFaderSide, EngineBridge,
    EngineBridgeChannel, EngineBridgeDeck, EngineBridgePlayer, PlayState, TempoRange,
};
use timecode::Timecode;
use tokio::runtime::Handle;

static ENGINE_CONNECTION: LazyLock<Mutex<Option<EngineConnection>>> =
    LazyLock::new(|| Mutex::new(None));

struct EngineConnection {
    tokio_handle: Handle,
    dj_engine: DJEngine,
    deck_state_receiver: tokio::sync::watch::Receiver<DeckState>,
    deck_update_sender: tokio::sync::mpsc::UnboundedSender<DeckUpdate>,
    ui_control_event_receiver: tokio::sync::mpsc::UnboundedReceiver<UIControlEvent>,
    loaded_track_sender: tokio::sync::mpsc::UnboundedSender<(usize, Option<Box<TrackAudioData>>)>,
    device_manager: DeviceManager,
}

impl Default for EngineBridgePlayer {
    fn default() -> Self {
        Self {
            // Safety: This set of data is ensured to be safe to initialize
            waveform_image: unsafe {
                QImage::from_raw_bytes(vec![0, 0, 0, 0], 1, 1, QImageFormat::Format_ARGB32)
            },
            waveform_image_stride: 1.0,

            // Safety: This set of data is ensured to be safe to initialize
            preview_waveform_image: unsafe {
                QImage::from_raw_bytes(vec![0, 0, 0, 0], 1, 1, QImageFormat::Format_ARGB32)
            },
            // current_track: None,
            // current_track_analysis: None,
            is_loading: false,

            beat_sync: BeatSyncMode::Off,
            key_sync: false,

            quanitze: true,

            jog_hold: false,
            jog_wait: false,
            jog_velocity: 0.0,

            play_state: PlayState::Stop,
            time: 0,
            cue_time_set: false,
            cue_time: 0,
            touch_cue_time_set: false,
            touch_cue_time: 0,
            reverse_enabled: false,

            tempo_range: TempoRange::TenPercent,
            tempo_reset: false,
            tempo_percent: 1.0,
            tempo_slider_position: 0.0,
            tempo_slider_is_accurate: true,
            master_tempo: false,

            slip: false,
            slip_playing: false,
            slip_time: 0,

            beat_loop_start_set: false,
            beat_loop_start: 0,
            beat_loop_end_set: false,
            beat_loop_end: 0,
            last_beat_loop_set: false,
            last_beat_loop_start: 0,
            last_beat_loop_end: 0,
            beat_loop_adjust_mode: BeatLoopAdjustMode::None,

            keyshift: 0.0,
        }
    }
}

impl Default for EngineBridgeChannel {
    fn default() -> Self {
        Self {
            player: EngineBridgePlayer::default(),

            gain: 0.0,
            eq_low: 0.0,
            eq_mid: 0.0,
            eq_high: 0.0,
            fx: 0.0,

            cue: false,

            fade: 1.0,

            cross_fader_side: CrossFaderSide::None,
        }
    }
}

impl Default for EngineBridgeDeck {
    fn default() -> Self {
        Self {
            mixer_channels: Default::default(),

            // master_fx: MasterFX::default(),
            crossfade: 0.5,

            master_channel_set: false,
            master_channel: 0,

            master_cue: false,
            master_gain: 0.0,

            channel_fx_effect: ChannelFXEffect::None,
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
pub struct EngineBridgeRust {
    pub source_open: bool,
    pub browser_page: BrowserPage,

    pub device_selected: bool,
    pub active_device: usize,

    pub deck_state: EngineBridgeDeck,

    pub tokio_handle: Handle,
    pub dj_engine: DJEngine,
    pub deck_state_receiver: tokio::sync::watch::Receiver<DeckState>,
    pub deck_update_sender: tokio::sync::mpsc::UnboundedSender<DeckUpdate>,
    pub ui_control_event_receiver: tokio::sync::mpsc::UnboundedReceiver<UIControlEvent>,
    pub loaded_track_sender:
        tokio::sync::mpsc::UnboundedSender<(usize, Option<Box<TrackAudioData>>)>,
    pub device_manager: DeviceManager,

    pub waveform_pixels_per_second: f64,

    pub mixer_channel_fx_proximity: bool,
    pub mixer_master_fx_proximity: bool,
}

impl cxx_qt::Constructor<()> for qobject::EngineBridge {
    type BaseArguments = ();
    type InitializeArguments = ();
    type NewArguments = ();

    fn route_arguments(
        args: (),
    ) -> (
        Self::NewArguments,
        Self::BaseArguments,
        Self::InitializeArguments,
    ) {
        (args, (), ())
    }

    fn new((): ()) -> EngineBridgeRust {
        let EngineConnection {
            tokio_handle,
            dj_engine,
            deck_state_receiver,
            deck_update_sender,
            ui_control_event_receiver,
            loaded_track_sender,
            device_manager,
        } = ENGINE_CONNECTION
            .lock()
            .expect("Engine connection mutex be free")
            .take()
            .expect("Expected engine to be connected to runtime");

        EngineBridgeRust {
            source_open: false,
            browser_page: BrowserPage::Closed,

            device_selected: false,
            active_device: 0,

            deck_state: EngineBridgeDeck::default(),

            tokio_handle,
            dj_engine,
            deck_state_receiver,
            deck_update_sender,
            ui_control_event_receiver,
            loaded_track_sender,
            device_manager,

            waveform_pixels_per_second: 200.0,

            mixer_channel_fx_proximity: false,
            mixer_master_fx_proximity: false,
        }
    }
}

impl EngineBridgeRust {
    pub fn register(
        tokio_handle: Handle,
        dj_engine: DJEngine,
        deck_state_receiver: tokio::sync::watch::Receiver<DeckState>,
        deck_update_sender: tokio::sync::mpsc::UnboundedSender<DeckUpdate>,
        ui_control_event_receiver: tokio::sync::mpsc::UnboundedReceiver<UIControlEvent>,
        loaded_track_sender: tokio::sync::mpsc::UnboundedSender<(
            usize,
            Option<Box<TrackAudioData>>,
        )>,
        device_manager: DeviceManager,
    ) {
        *ENGINE_CONNECTION
            .lock()
            .expect("Engine connection mutex be free") = Some(EngineConnection {
            tokio_handle,
            dj_engine,
            deck_state_receiver,
            deck_update_sender,
            ui_control_event_receiver,
            loaded_track_sender,
            device_manager,
        });
    }

    fn update_from_deck_state(&mut self, state: &libdj::types::deck::DeckState) {
        for (channel, new_channel) in self
            .deck_state
            .mixer_channels
            .iter_mut()
            .zip(&state.mixer_channels)
        {
            channel.player.is_loading = new_channel.player.is_loading;
            channel.player.beat_sync = match new_channel.player.beat_sync {
                libdj::types::deck::BeatSyncMode::Off => BeatSyncMode::Off,
                libdj::types::deck::BeatSyncMode::BPMSync => BeatSyncMode::BPMSync,
                libdj::types::deck::BeatSyncMode::BeatSync => BeatSyncMode::BeatSync,
            };
            channel.player.key_sync = new_channel.player.key_sync;
            channel.player.quanitze = new_channel.player.quanitze;
            channel.player.jog_hold = new_channel.player.jog_hold;
            channel.player.jog_wait = new_channel.player.jog_wait;
            channel.player.jog_velocity = new_channel.player.jog_velocity;
            channel.player.play_state = match new_channel.player.play_state {
                libdj::types::deck::PlayState::Stop => PlayState::Stop,
                libdj::types::deck::PlayState::Play => PlayState::Play,
                libdj::types::deck::PlayState::Cue => PlayState::Cue,
            };
            channel.player.time = new_channel.player.time.nanoseconds;
            channel.player.cue_time_set = new_channel.player.cue_time.is_some();
            channel.player.cue_time = new_channel
                .player
                .cue_time
                .map_or(0, |timecode| timecode.nanoseconds);
            channel.player.touch_cue_time_set = new_channel.player.touch_cue_time.is_some();
            channel.player.touch_cue_time = new_channel
                .player
                .touch_cue_time
                .map_or(0, |timecode| timecode.nanoseconds);
            channel.player.reverse_enabled = new_channel.player.reverse_enabled;
            channel.player.tempo_range = match new_channel.player.tempo_range {
                libdj::types::deck::TempoRange::SixPercent => TempoRange::SixPercent,
                libdj::types::deck::TempoRange::TenPercent => TempoRange::TenPercent,
                libdj::types::deck::TempoRange::SixteenPercent => TempoRange::SixteenPercent,
                libdj::types::deck::TempoRange::OneHundredPercent => TempoRange::OneHundredPercent,
            };
            channel.player.tempo_reset = new_channel.player.tempo_reset;
            channel.player.tempo_percent = new_channel.player.tempo_percent;
            channel.player.tempo_slider_position = new_channel.player.tempo_slider_position;
            channel.player.tempo_slider_is_accurate = new_channel.player.tempo_slider_is_accurate;
            channel.player.master_tempo = new_channel.player.master_tempo;
            channel.player.slip = new_channel.player.slip;
            channel.player.slip_playing = new_channel.player.slip_playing;
            channel.player.slip_time = new_channel.player.slip_time.nanoseconds;
            channel.player.beat_loop_start_set = new_channel.player.beat_loop_start.is_some();
            channel.player.beat_loop_start = new_channel
                .player
                .beat_loop_start
                .map_or(0, |timecode| timecode.nanoseconds);
            channel.player.beat_loop_end_set = new_channel.player.beat_loop_end.is_some();
            channel.player.beat_loop_end = new_channel
                .player
                .beat_loop_end
                .map_or(0, |timecode| timecode.nanoseconds);
            channel.player.last_beat_loop_set = new_channel.player.last_beat_loop.is_some();
            channel.player.last_beat_loop_start = new_channel
                .player
                .last_beat_loop
                .map_or(0, |timecode| timecode.0.nanoseconds);
            channel.player.last_beat_loop_end = new_channel
                .player
                .last_beat_loop
                .map_or(0, |timecode| timecode.1.nanoseconds);
            channel.player.beat_loop_adjust_mode = match new_channel.player.beat_loop_adjust_mode {
                libdj::types::deck::BeatLoopAdjustMode::None => BeatLoopAdjustMode::None,
                libdj::types::deck::BeatLoopAdjustMode::In => BeatLoopAdjustMode::In,
                libdj::types::deck::BeatLoopAdjustMode::Out => BeatLoopAdjustMode::Out,
            };
            channel.player.keyshift = new_channel.player.keyshift;

            channel.gain = new_channel.gain;
            channel.eq_low = new_channel.eq_low;
            channel.eq_mid = new_channel.eq_mid;
            channel.eq_high = new_channel.eq_high;
            channel.fx = new_channel.fx;
            channel.cue = new_channel.cue;
            channel.fade = new_channel.fade;
            channel.cross_fader_side = match new_channel.cross_fader_side {
                libdj::types::deck::CrossFaderSide::A => CrossFaderSide::A,
                libdj::types::deck::CrossFaderSide::B => CrossFaderSide::B,
                libdj::types::deck::CrossFaderSide::None => CrossFaderSide::None,
            };
        }

        self.deck_state.master_channel_set = state.master_channel.is_some();
        self.deck_state.master_channel = state.master_channel.unwrap_or(0);
        self.deck_state.crossfade = state.crossfade;
        self.deck_state.master_cue = state.master_cue;
        self.deck_state.master_gain = state.master_gain;
        self.deck_state.channel_fx_effect = match state.channel_fx_effect {
            libdj::types::deck::ChannelFXEffect::None => ChannelFXEffect::None,
            libdj::types::deck::ChannelFXEffect::Space => ChannelFXEffect::Space,
            libdj::types::deck::ChannelFXEffect::DubEcho => ChannelFXEffect::DubEcho,
            libdj::types::deck::ChannelFXEffect::Bitcrush => ChannelFXEffect::Bitcrush,
            libdj::types::deck::ChannelFXEffect::Pitch => ChannelFXEffect::Pitch,
            libdj::types::deck::ChannelFXEffect::Noise => ChannelFXEffect::Noise,
            libdj::types::deck::ChannelFXEffect::Filter => ChannelFXEffect::Filter,
        };
    }
}

impl qobject::EngineBridge {
    fn before_frame(mut self: Pin<&mut EngineBridge>) {
        let mut engine_bridge = self.as_mut().rust_mut();

        if let Ok(changed) = engine_bridge.deck_state_receiver.has_changed()
            && changed
        {
            let new_deck_state = engine_bridge
                .deck_state_receiver
                .borrow_and_update()
                .clone();

            engine_bridge.update_from_deck_state(&new_deck_state);
        }

        while let Ok(event) = engine_bridge.ui_control_event_receiver.try_recv() {
            match event {
                UIControlEvent::USBEjectPress { slot } => {
                    warn!("not implemented: begin device eject for {slot}");
                }
                UIControlEvent::USBEjectRelease { slot } => {
                    warn!("not implemented: stop device eject for {slot}");
                }
                UIControlEvent::BrowserEncoderAdjust { delta } => {
                    if delta.is_sign_positive() {
                        engine_bridge.waveform_pixels_per_second *= 2.0 * f64::from(delta);
                    } else {
                        engine_bridge.waveform_pixels_per_second /= 2.0 * f64::from(delta.abs());
                    }
                }
                UIControlEvent::BrowserEncoderPress => {}
                UIControlEvent::BrowserBackPress => {
                    if engine_bridge.source_open {
                        engine_bridge.source_open = false;
                    } else {
                        engine_bridge.browser_page = BrowserPage::Closed;
                    }
                }
                UIControlEvent::BrowserSourcePress => {
                    if engine_bridge.source_open {
                        engine_bridge.source_open = false;

                        if !engine_bridge.device_selected {
                            engine_bridge.browser_page = BrowserPage::Closed;
                        }
                    } else {
                        engine_bridge.source_open = true;
                    }
                }
                UIControlEvent::BrowserBrowsePress => {
                    engine_bridge.browser_page = BrowserPage::Device;

                    if !engine_bridge.source_open {
                        engine_bridge.source_open = true;
                    }
                }
                UIControlEvent::BrowserPlaylistPress => {
                    engine_bridge.browser_page = BrowserPage::Playlist;

                    if !engine_bridge.source_open {
                        engine_bridge.source_open = true;
                    }
                }
                UIControlEvent::BrowserSearchPress => {
                    engine_bridge.browser_page = BrowserPage::Search;

                    if !engine_bridge.source_open {
                        engine_bridge.source_open = true;
                    }
                }
                UIControlEvent::MixerChannelFXProximityPress => {
                    engine_bridge.mixer_channel_fx_proximity = true;
                }
                UIControlEvent::MixerChannelFXProximityRelease => {
                    engine_bridge.mixer_channel_fx_proximity = false;
                }
                UIControlEvent::MixerMasterFXSelectTouchPress => {
                    engine_bridge.mixer_master_fx_proximity = true;
                }
                UIControlEvent::MixerMasterFXSelectTouchRelease => {
                    engine_bridge.mixer_master_fx_proximity = false;
                }
            }
        }
    }

    fn load_track(self: Pin<&mut EngineBridge>, player: usize, device: usize, track_id: u32) {
        let loaded_track_sender = self.loaded_track_sender.clone();
        let deck_update_sender = self.deck_update_sender.clone();

        let device_manager = self.device_manager.subscribe();

        self.tokio_handle.spawn(async move {
            let mut locked_device_database = device_manager.devices.lock().await;

            if let Some(qualified_device) = locked_device_database.get_mut(&device)
                && let Some(track) = qualified_device.database.library.tracks.get(&track_id)
            {
                let _ = loaded_track_sender.send((player, None));

                let _ = deck_update_sender.send(Box::new(move |deck_state| {
                    if let Some(mixer_channel) = deck_state.mixer_channels.get_mut(player) {
                        mixer_channel.player.current_track = None;
                        mixer_channel.player.is_loading = true;

                        mixer_channel.player.time = Timecode::zero();
                        mixer_channel.player.slip_time = Timecode::zero();
                        mixer_channel.player.cue_time = None;
                        mixer_channel.player.touch_cue_time = None;

                        mixer_channel.player.beat_loop_start = None;
                        mixer_channel.player.beat_loop_end = None;

                        mixer_channel.player.keyshift = 0.0;
                    }
                }));

                let cloned_track = track.clone();

                let cloned_deck_update_sender = deck_update_sender.clone();

                std::thread::spawn(move || {
                    let audio_data = match TrackAudioData::load_from_file(&cloned_track.audio_path)
                    {
                        Ok(audio_data) => audio_data,
                        Err(err) => {
                            // todo: build a way to propagate errors to the ui
                            warn!("Track load error: {err:?}");

                            let _ = cloned_deck_update_sender.send(Box::new(move |deck_state| {
                                if let Some(mixer_channel) =
                                    deck_state.mixer_channels.get_mut(player)
                                {
                                    mixer_channel.player.is_loading = false;
                                }
                            }));

                            return;
                        }
                    };

                    let _ = loaded_track_sender.send((player, Some(Box::new(audio_data))));

                    let _ = cloned_deck_update_sender.send(Box::new(move |deck_state| {
                        if let Some(mixer_channel) = deck_state.mixer_channels.get_mut(player) {
                            mixer_channel.player.current_track = Some((device, cloned_track));
                            mixer_channel.player.is_loading = false;
                        }
                    }));
                });

                // todo: this should be moved to a new thread
                if let Ok(track_analysis) = qualified_device.database.load_analysis(track_id) {
                    let cloned_track_analysis = track_analysis.clone();

                    drop(locked_device_database);

                    let _ = deck_update_sender.send(Box::new(move |deck_state| {
                        if let Some(mixer_channel) = deck_state.mixer_channels.get_mut(player) {
                            mixer_channel.player.current_track_analysis =
                                Some(cloned_track_analysis);
                        }
                    }));
                } else {
                    drop(locked_device_database);

                    std::thread::spawn(move || {
                        // todo: perform track analysis
                    });
                }
            }
        });
    }

    fn eject_track(self: Pin<&mut EngineBridge>, player: usize) {
        let _ = self.deck_update_sender.send(Box::new(move |deck_state| {
            if let Some(mixer_channel) = deck_state.mixer_channels.get_mut(player) {
                mixer_channel.player.current_track = None;
                mixer_channel.player.current_track_analysis = None;
                mixer_channel.player.is_loading = false;

                mixer_channel.player.time = Timecode::zero();
                mixer_channel.player.slip_time = Timecode::zero();
                mixer_channel.player.cue_time = None;
                mixer_channel.player.touch_cue_time = None;

                mixer_channel.player.beat_loop_start = None;
                mixer_channel.player.beat_loop_end = None;

                mixer_channel.player.keyshift = 0.0;
            }
        }));

        let _ = self.loaded_track_sender.send((player, None));
    }

    // fn eject_device(self: Pin<&mut EngineBridge>, device: usize) {}

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn touch_cue_move(self: Pin<&mut EngineBridge>, player: usize, position: f32) {
        let _ = self.deck_update_sender.send(Box::new(move |deck_state| {
            if let Some(mixer_channel) = deck_state.mixer_channels.get_mut(player)
                && let Some((_, track)) = &mixer_channel.player.current_track
            {
                mixer_channel.player.touch_cue_time = Some(Timecode::from_nanoseconds(
                    (track.duration as f64 * f64::from(position)) as i64,
                ));
            }
        }));
    }

    fn touch_cue_release(self: Pin<&mut EngineBridge>, player: usize) {
        let _ = self.deck_update_sender.send(Box::new(move |deck_state| {
            if let Some(mixer_channel) = deck_state.mixer_channels.get_mut(player) {
                mixer_channel.player.touch_cue_time = None;
            }
        }));
    }
}
