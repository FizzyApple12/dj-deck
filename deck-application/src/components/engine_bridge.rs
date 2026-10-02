// this file is horrific, thanks cxx_qt

// these are unavoidable when using cxx_qt :sob:
#![allow(
    clippy::float_cmp,
    clippy::needless_pass_by_value,
    clippy::unnecessary_box_returns
)]

#[cxx_qt::bridge]
mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qimage.h");
        type QImage = cxx_qt_lib::QImage;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
        include!("cxx-qt-lib/qbytearray.h");
        type QByteArray = cxx_qt_lib::QByteArray;
        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
        include!("cxx-qt-lib/qvector.h");
        type QVector_i32 = cxx_qt_lib::QVector<i32>;
    }

    unsafe extern "C++Qt" {
        include!(<QtCore/QAbstractListModel>);
        #[qobject]
        type QAbstractListModel;

        include!("cxx-qt-lib/qobject.h");
        include!("gpu_texture_source.h");

        #[qobject]
        #[base = QObject]
        type GpuTextureSource;

        #[rust_name = "set_image"]
        fn setImage(self: Pin<&mut GpuTextureSource>, new_image: &QImage);
    }

    #[namespace = "rust::cxxqtlib1"]
    unsafe extern "C++" {
        include!("cxx-qt-lib/common.h");
        include!("gpu_texture_source.h");

        #[cxx_name = "new_ptr"]
        fn new_gpu_texture_source() -> *mut GpuTextureSource;
    }

    #[qml_element]
    qnamespace!("BrowserPage");
    #[qenum]
    #[namespace = "BrowserPage"]
    pub enum BrowserPage {
        Closed,
        Device,
        Playlist,
        Search,
    }

    #[qml_element]
    qnamespace!("TempoRange");
    #[qenum]
    #[namespace = "TempoRange"]
    pub enum TempoRange {
        SixPercent,
        TenPercent,
        SixteenPercent,
        OneHundredPercent,
    }

    #[qml_element]
    qnamespace!("PlayState");
    #[qenum]
    #[namespace = "PlayState"]
    pub enum PlayState {
        Stop,
        Play,
        Cue,
    }

    #[qml_element]
    qnamespace!("CrossFaderSide");
    #[qenum]
    #[namespace = "CrossFaderSide"]
    pub enum CrossFaderSide {
        A,
        B,
        None,
    }

    #[qml_element]
    qnamespace!("BeatSyncMode");
    #[qenum]
    #[namespace = "BeatSyncMode"]
    pub enum BeatSyncMode {
        Off,
        BPMSync,
        BeatSync,
    }

    #[qml_element]
    qnamespace!("BeatLoopAdjustMode");
    #[qenum]
    #[namespace = "BeatLoopAdjustMode"]
    pub enum BeatLoopAdjustMode {
        None,
        In,
        Out,
    }

    #[qml_element]
    qnamespace!("ChannelFXEffect");
    #[qenum]
    #[namespace = "ChannelFXEffect"]
    pub enum ChannelFXEffect {
        None,
        Space,
        DubEcho,
        Bitcrush,
        Pitch,
        Noise,
        Filter,
    }

    #[qml_element]
    qnamespace!("BrowserEntryType");
    #[qenum]
    #[namespace = "BrowserEntryType"]
    pub enum BrowserEntryType {
        Track,
        Playlist,
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[base = QAbstractListModel]
        pub type SourceListModel = super::SourceListModelRust;

        #[qinvokable]
        #[cxx_override]
        #[rust_name = "row_count"]
        fn rowCount(self: &SourceListModel, parent: &QModelIndex) -> i32;

        #[inherit]
        fn index(
            self: &SourceListModel,
            row: i32,
            column: i32,
            parent: &QModelIndex,
        ) -> QModelIndex;

        #[qinvokable]
        #[cxx_override]
        #[rust_name = "role_names"]
        fn roleNames(self: &SourceListModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        #[cxx_override]
        fn data(self: &SourceListModel, index: &QModelIndex, role: i32) -> QVariant;

        #[inherit]
        #[rust_name = "begin_insert_rows"]
        fn beginInsertRows(
            self: Pin<&mut SourceListModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );
        #[inherit]
        #[rust_name = "end_insert_rows"]
        fn endInsertRows(self: Pin<&mut SourceListModel>);

        #[inherit]
        #[rust_name = "begin_remove_rows"]
        fn beginRemoveRows(
            self: Pin<&mut SourceListModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );

        #[inherit]
        #[rust_name = "end_remove_rows"]
        fn endRemoveRows(self: Pin<&mut SourceListModel>);

        #[inherit]
        #[rust_name = "begin_reset_model"]
        fn beginResetModel(self: Pin<&mut SourceListModel>);
        #[inherit]
        #[rust_name = "end_reset_model"]
        fn endResetModel(self: Pin<&mut SourceListModel>);

        #[inherit]
        #[qsignal]
        #[rust_name = "data_changed"]
        fn dataChanged(
            self: Pin<&mut SourceListModel>,
            top_left: &QModelIndex,
            bottom_right: &QModelIndex,
            roles: &QVector_i32,
        );

        #[qsignal]
        unsafe fn called(self: Pin<&mut SourceListModel>, inner: *mut SourceListModel);
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[base = QAbstractListModel]
        pub type BrowserListModel = super::BrowserListModelRust;

        #[qinvokable]
        #[cxx_override]
        #[rust_name = "row_count"]
        fn rowCount(self: &BrowserListModel, parent: &QModelIndex) -> i32;

        #[inherit]
        fn index(
            self: &BrowserListModel,
            row: i32,
            column: i32,
            parent: &QModelIndex,
        ) -> QModelIndex;

        #[qinvokable]
        #[cxx_override]
        #[rust_name = "role_names"]
        fn roleNames(self: &BrowserListModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        #[cxx_override]
        fn data(self: &BrowserListModel, index: &QModelIndex, role: i32) -> QVariant;

        #[inherit]
        #[rust_name = "begin_insert_rows"]
        fn beginInsertRows(
            self: Pin<&mut BrowserListModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );
        #[inherit]
        #[rust_name = "end_insert_rows"]
        fn endInsertRows(self: Pin<&mut BrowserListModel>);

        #[inherit]
        #[rust_name = "begin_remove_rows"]
        fn beginRemoveRows(
            self: Pin<&mut BrowserListModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );

        #[inherit]
        #[rust_name = "end_remove_rows"]
        fn endRemoveRows(self: Pin<&mut BrowserListModel>);

        #[inherit]
        #[rust_name = "begin_reset_model"]
        fn beginResetModel(self: Pin<&mut BrowserListModel>);
        #[inherit]
        #[rust_name = "end_reset_model"]
        fn endResetModel(self: Pin<&mut BrowserListModel>);

        #[inherit]
        #[qsignal]
        #[rust_name = "data_changed"]
        fn dataChanged(
            self: Pin<&mut BrowserListModel>,
            top_left: &QModelIndex,
            bottom_right: &QModelIndex,
            roles: &QVector_i32,
        );

        #[qsignal]
        unsafe fn called(self: Pin<&mut BrowserListModel>, inner: *mut BrowserListModel);
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(*mut GpuTextureSource, waveform_texture_source)]
        #[qproperty(f32, waveform_texture_stride)]
        #[qproperty(f32, waveform_length_seconds)]
        #[qproperty(*mut GpuTextureSource, preview_waveform_texture_source)]
        #[qproperty(bool, track_loaded)]
        #[qproperty(QString, track_name)]
        #[qproperty(QString, track_artist)]
        #[qproperty(i64, track_length)]
        #[qproperty(f32, current_bpm)]
        #[qproperty(bool, is_loading)]
        #[qproperty(BeatSyncMode, beat_sync)]
        #[qproperty(bool, key_sync)]
        #[qproperty(bool, quanitze)]
        #[qproperty(bool, jog_hold)]
        #[qproperty(bool, jog_wait)]
        #[qproperty(f32, jog_velocity)]
        #[qproperty(bool, display_remaining)]
        #[qproperty(PlayState, play_state)]
        #[qproperty(i64, time)]
        #[qproperty(bool, cue_time_set)]
        #[qproperty(i64, cue_time)]
        #[qproperty(bool, touch_cue_time_set)]
        #[qproperty(i64, touch_cue_time)]
        #[qproperty(bool, reverse_enabled)]
        #[qproperty(TempoRange, tempo_range)]
        #[qproperty(bool, tempo_reset)]
        #[qproperty(f32, tempo_percent)]
        #[qproperty(f32, tempo_slider_position)]
        #[qproperty(bool, tempo_slider_is_accurate)]
        #[qproperty(bool, master_tempo)]
        #[qproperty(bool, slip)]
        #[qproperty(bool, slip_playing)]
        #[qproperty(i64, slip_time)]
        #[qproperty(bool, beat_loop_start_set)]
        #[qproperty(i64, beat_loop_start)]
        #[qproperty(bool, beat_loop_end_set)]
        #[qproperty(i64, beat_loop_end)]
        #[qproperty(bool, last_beat_loop_set)]
        #[qproperty(i64, last_beat_loop_start)]
        #[qproperty(i64, last_beat_loop_end)]
        #[qproperty(BeatLoopAdjustMode, beat_loop_adjust_mode)]
        #[qproperty(f32, keyshift)]
        type EngineBridgePlayer = super::EngineBridgePlayerRust;

        #[qobject]
        #[qml_element]
        #[qproperty(*mut EngineBridgePlayer, player)]
        #[qproperty(f32, gain)]
        #[qproperty(f32, eq_low)]
        #[qproperty(f32, eq_mid)]
        #[qproperty(f32, eq_high)]
        #[qproperty(f32, fx)]
        #[qproperty(bool, cue)]
        #[qproperty(f32, fade)]
        #[qproperty(CrossFaderSide, cross_fader_side)]
        type EngineBridgeChannel = super::EngineBridgeChannelRust;

        #[qobject]
        #[qml_element]
        #[qproperty(bool, master_channel_set)]
        #[qproperty(i32, master_channel)]
        #[qproperty(f32, crossfade)]
        #[qproperty(bool, master_cue)]
        #[qproperty(f32, master_gain)]
        #[qproperty(ChannelFXEffect, channel_fx_effect)]
        type EngineBridgeDeck = super::EngineBridgeDeckRust;

        #[qinvokable]
        fn mixer_channel(self: &EngineBridgeDeck, index: i32) -> *mut EngineBridgeChannel;

        #[qobject]
        #[qml_element]
        #[qproperty(bool, source_open)]
        #[qproperty(BrowserPage, browser_page)]
        #[qproperty(*mut SourceListModel, source_index)]
        #[qproperty(*mut BrowserListModel, browser_index)]
        #[qproperty(QString, search)]
        #[qproperty(bool, device_selected)]
        #[qproperty(i32, active_device)]
        #[qproperty(*mut EngineBridgeDeck, deck_state)]
        #[qproperty(f64, waveform_pixels_per_second)]
        #[qproperty(bool, mixer_channel_fx_proximity)]
        #[qproperty(bool, mixer_master_fx_proximity)]
        type EngineBridge = super::EngineBridgeRust;

        #[qinvokable]
        fn before_frame(self: Pin<&mut EngineBridge>);

        #[qinvokable]
        fn select_device(self: Pin<&mut EngineBridge>, device: i32);

        #[qinvokable]
        fn load_track(self: &EngineBridge, player: i32, device: i32, track_id: i32);

        #[qinvokable]
        fn eject_track(self: &EngineBridge, player: i32);

        // #[qinvokable]
        // fn eject_device(self: &EngineBridge, device: i32);

        #[qinvokable]
        fn touch_cue_move(self: &EngineBridge, player: i32, position: f32);

        #[qinvokable]
        fn touch_cue_release(self: &EngineBridge, player: i32);

        #[qsignal]
        unsafe fn called_source_list_model(
            self: Pin<&mut EngineBridge>,
            inner: *mut SourceListModel,
        );

        #[qsignal]
        unsafe fn called_browser_list_model(
            self: Pin<&mut EngineBridge>,
            inner: *mut BrowserListModel,
        );
    }

    #[namespace = "rust::cxxqtlib1"]
    unsafe extern "C++" {
        include!("cxx-qt-lib/common.h");

        #[cxx_name = "new_ptr"]
        fn new_engine_bridge_player() -> *mut EngineBridgePlayer;

        #[cxx_name = "new_ptr"]
        fn new_engine_bridge_channel() -> *mut EngineBridgeChannel;

        #[cxx_name = "new_ptr"]
        fn new_engine_bridge_deck() -> *mut EngineBridgeDeck;
    }

    impl cxx_qt::Threading for EngineBridge {}

    impl cxx_qt::Constructor<(), NewArguments = ()> for EngineBridge {}
}

use std::{pin::Pin, sync::Mutex};

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QByteArray, QHash, QImage, QModelIndex, QObjectExt, QString, QVariant};
use libdatabase::device_manager::DeviceManager;
use libdj::{
    engine::DJEngine,
    types::{bindings::UIControlEvent, deck::DeckState, playback::DeckUpdate},
};
use libdsp::audio_loader::TrackAudioData;
use log::warn;
use qobject::{
    BeatLoopAdjustMode, BeatSyncMode, BrowserEntryType, BrowserPage, ChannelFXEffect,
    CrossFaderSide, EngineBridgeChannel, EngineBridgeDeck, EngineBridgePlayer, PlayState,
    SourceListModel, TempoRange,
};
use timecode::Timecode;
use tokio::runtime::Handle;

use crate::{
    components::engine_bridge::qobject::{
        BrowserListModel, GpuTextureSource, QVector_i32, new_engine_bridge_channel,
        new_engine_bridge_deck, new_engine_bridge_player, new_gpu_texture_source,
    },
    waveform_loader::{preview_waveform_to_shader_texture, waveform_to_shader_texture},
};

static ENGINE_CONNECTION: Mutex<Option<EngineConnection>> = Mutex::new(None);

#[allow(unused)]
struct EngineConnection {
    tokio_handle: Handle,
    dj_engine: DJEngine,
    deck_state_receiver: tokio::sync::watch::Receiver<DeckState>,
    deck_update_sender: tokio::sync::mpsc::UnboundedSender<DeckUpdate>,
    ui_control_event_receiver: tokio::sync::mpsc::UnboundedReceiver<UIControlEvent>,
    loaded_track_sender: tokio::sync::mpsc::UnboundedSender<(usize, Option<Box<TrackAudioData>>)>,
    device_manager: DeviceManager,
}

#[allow(clippy::struct_excessive_bools)]
pub struct EngineBridgePlayerRust {
    pub waveform_texture_source: *mut GpuTextureSource,
    pub waveform_texture_stride: f32,
    pub waveform_length_seconds: f32,

    pub preview_waveform_texture_source: *mut GpuTextureSource,

    pub track_loaded: bool,
    pub track_name: QString,
    pub track_artist: QString,
    pub track_length: i64, // nanoseconds

    pub current_bpm: f32,

    pub is_loading: bool,

    pub beat_sync: BeatSyncMode,
    pub key_sync: bool,

    pub quanitze: bool,

    pub jog_hold: bool,
    pub jog_wait: bool,
    pub jog_velocity: f32,

    pub display_remaining: bool,

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

impl Default for EngineBridgePlayerRust {
    fn default() -> Self {
        Self {
            waveform_texture_source: new_gpu_texture_source(),
            waveform_texture_stride: 1.0,
            waveform_length_seconds: 0.0,

            preview_waveform_texture_source: new_gpu_texture_source(),

            track_loaded: false,
            track_name: QString::from(""),
            track_artist: QString::from(""),
            track_length: 0,

            current_bpm: 0.0,

            is_loading: false,

            beat_sync: BeatSyncMode::Off,
            key_sync: false,

            quanitze: true,

            jog_hold: false,
            jog_wait: false,
            jog_velocity: 0.0,

            display_remaining: true,

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

pub struct EngineBridgeChannelRust {
    pub player: *mut EngineBridgePlayer,

    pub gain: f32,    // decibels
    pub eq_low: f32,  // decibels
    pub eq_mid: f32,  // decibels
    pub eq_high: f32, // decibels
    pub fx: f32,      // percent

    pub cue: bool, // cue enabled

    pub fade: f32, // percent

    pub cross_fader_side: CrossFaderSide,
}

impl Default for EngineBridgeChannelRust {
    fn default() -> Self {
        Self {
            player: new_engine_bridge_player(),

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

pub struct EngineBridgeDeckRust {
    pub mixer_channels: [*mut EngineBridgeChannel; 4],

    pub master_channel_set: bool,
    pub master_channel: i32,

    pub crossfade: f32,

    pub master_cue: bool,
    pub master_gain: f32, // decibels

    pub channel_fx_effect: ChannelFXEffect,
}

impl Default for EngineBridgeDeckRust {
    fn default() -> Self {
        Self {
            mixer_channels: [
                new_engine_bridge_channel(),
                new_engine_bridge_channel(),
                new_engine_bridge_channel(),
                new_engine_bridge_channel(),
            ],

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

impl qobject::EngineBridgeDeck {
    #[allow(clippy::cast_sign_loss, clippy::indexing_slicing)]
    fn mixer_channel(&self, index: i32) -> *mut EngineBridgeChannel {
        usize::try_from(index)
            .ok()
            .and_then(|i| self.mixer_channels.get(i).copied())
            .unwrap_or(std::ptr::null_mut())
    }
}

#[allow(clippy::struct_excessive_bools)]
pub struct EngineBridgeRust {
    pub source_open: bool,
    pub browser_page: BrowserPage,

    pub source_index: *mut SourceListModel,

    pub browser_index: *mut BrowserListModel,

    pub search: QString,

    pub device_selected: bool,
    pub active_device: i32,

    pub deck_state: *mut EngineBridgeDeck,

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
        EngineBridgeRust {
            source_open: false,
            browser_page: BrowserPage::Closed,

            source_index: std::ptr::null_mut(),

            browser_index: std::ptr::null_mut(),

            search: QString::from(""),

            device_selected: false,
            active_device: 0,

            deck_state: new_engine_bridge_deck(),

            waveform_pixels_per_second: 200.0,

            mixer_channel_fx_proximity: false,
            mixer_master_fx_proximity: false,
        }
    }

    fn initialize(mut self: core::pin::Pin<&mut Self>, (): Self::InitializeArguments) {
        // Safety: pray to god
        self.as_mut()
            .on_called_source_list_model(|mut root_object, child_object| unsafe {
                root_object.as_mut().called_source_list_model(child_object);
            })
            .release();

        // Safety: pray even harder
        self.as_mut()
            .on_called_browser_list_model(|mut root_object, child_object| unsafe {
                root_object.as_mut().called_browser_list_model(child_object);
            })
            .release();

        // Safety: Probably not
        let mut deck_state_pin = unsafe {
            let Some(deck_state) = self.deck_state.as_mut() else {
                return;
            };

            Pin::new_unchecked(&mut *deck_state)
        };

        deck_state_pin.as_mut().set_parent(self.as_mut());

        for channel in deck_state_pin.as_mut().mixer_channels {
            // Safety: Probably not
            let mut mixer_channel_pin = unsafe {
                let Some(mixer_channel) = channel.as_mut() else {
                    return;
                };

                Pin::new_unchecked(&mut *mixer_channel)
            };

            mixer_channel_pin
                .as_mut()
                .set_parent(deck_state_pin.as_mut());

            // Safety: Probably not
            let mut player_pin = unsafe {
                let Some(player) = mixer_channel_pin.player.as_mut() else {
                    return;
                };

                Pin::new_unchecked(&mut *player)
            };

            player_pin.as_mut().set_parent(mixer_channel_pin.as_mut());

            let mut waveform_texture_source_pin = unsafe {
                let Some(waveform_texture_source) = player_pin.waveform_texture_source.as_mut()
                else {
                    return;
                };

                Pin::new_unchecked(&mut *waveform_texture_source)
            };

            waveform_texture_source_pin
                .as_mut()
                .set_parent(player_pin.as_mut());

            let mut preview_waveform_texture_source_pin = unsafe {
                let Some(preview_waveform_texture_source) =
                    player_pin.preview_waveform_texture_source.as_mut()
                else {
                    return;
                };

                Pin::new_unchecked(&mut *preview_waveform_texture_source)
            };

            preview_waveform_texture_source_pin
                .as_mut()
                .set_parent(player_pin.as_mut());
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
}

impl qobject::EngineBridge {
    fn before_frame(mut self: Pin<&mut Self>) {
        let Ok(mut engine_connection_lock) = ENGINE_CONNECTION.lock() else {
            return;
        };
        let Some(engine_connection) = engine_connection_lock.as_mut() else {
            return;
        };

        let mut source_index_updated = false;

        while let Some(event) = engine_connection.device_manager.try_recv() {
            source_index_updated = true;

            match event {
                libdatabase::device_manager::DeviceManagerEvent::DeviceConnected(number) => {
                    self.as_mut().device_connected(engine_connection, number);
                }
                libdatabase::device_manager::DeviceManagerEvent::DeviceDisconnected(number) => {
                    self.as_mut().device_disconnected(number);
                }
            }
        }

        if source_index_updated {
            self.as_mut().rebuild_browse_index(engine_connection);
        }

        if let Ok(changed) = engine_connection.deck_state_receiver.has_changed()
            && changed
        {
            let new_deck_state = engine_connection
                .deck_state_receiver
                .borrow_and_update()
                .clone();

            self.as_mut().update_from_deck_state(&new_deck_state);
        }

        while let Ok(event) = engine_connection.ui_control_event_receiver.try_recv() {
            match event {
                UIControlEvent::USBEjectPress { slot } => {
                    warn!("not implemented: begin device eject for {slot}");
                }
                UIControlEvent::USBEjectRelease { slot } => {
                    warn!("not implemented: stop device eject for {slot}");
                }
                UIControlEvent::BrowserEncoderAdjust { delta } => {
                    let new_pixels_per_second = if delta.is_sign_positive() {
                        self.waveform_pixels_per_second * 2.0 * f64::from(delta)
                    } else {
                        self.waveform_pixels_per_second / (2.0 * f64::from(delta.abs()))
                    };

                    self.as_mut()
                        .set_waveform_pixels_per_second(new_pixels_per_second);
                }
                UIControlEvent::BrowserEncoderPress => {}
                UIControlEvent::BrowserBackPress => {
                    if self.source_open {
                        self.as_mut().set_source_open(false);

                        if !self.device_selected {
                            self.as_mut().set_browser_page(BrowserPage::Closed);
                        }
                    } else {
                        self.as_mut().set_browser_page(BrowserPage::Closed);
                    }

                    self.as_mut().rebuild_browse_index(engine_connection);
                }
                UIControlEvent::BrowserSourcePress => {
                    if self.source_open {
                        self.as_mut().set_source_open(false);

                        if !self.device_selected {
                            self.as_mut().set_browser_page(BrowserPage::Closed);
                        }
                    } else {
                        self.as_mut().set_source_open(true);
                    }

                    self.as_mut().rebuild_browse_index(engine_connection);
                }
                UIControlEvent::BrowserBrowsePress => {
                    if self.browser_page == BrowserPage::Closed {
                        self.as_mut().set_browser_page(BrowserPage::Device);

                        if !self.device_selected {
                            self.as_mut().set_source_open(true);
                        }
                    } else if self.source_open {
                        self.as_mut().set_source_open(false);

                        if !self.device_selected {
                            self.as_mut().set_browser_page(BrowserPage::Closed);
                        }
                    } else {
                        self.as_mut().set_browser_page(BrowserPage::Closed);
                    }

                    self.as_mut().rebuild_browse_index(engine_connection);
                }
                UIControlEvent::BrowserPlaylistPress => {
                    if self.browser_page == BrowserPage::Closed {
                        self.as_mut().set_browser_page(BrowserPage::Playlist);

                        if !self.device_selected {
                            self.as_mut().set_source_open(true);
                        }
                    } else if self.source_open {
                        self.as_mut().set_source_open(false);

                        if !self.device_selected {
                            self.as_mut().set_browser_page(BrowserPage::Closed);
                        }
                    } else {
                        self.as_mut().set_browser_page(BrowserPage::Closed);
                    }

                    self.as_mut().rebuild_browse_index(engine_connection);
                }
                UIControlEvent::BrowserSearchPress => {
                    if self.as_mut().browser_page == BrowserPage::Closed {
                        self.as_mut().set_browser_page(BrowserPage::Search);

                        if !self.as_mut().device_selected {
                            self.as_mut().set_source_open(true);
                        }
                    } else if self.as_mut().source_open {
                        self.as_mut().set_source_open(false);

                        if !self.as_mut().device_selected {
                            self.as_mut().set_browser_page(BrowserPage::Closed);
                        }
                    } else {
                        self.as_mut().set_browser_page(BrowserPage::Closed);
                    }

                    self.as_mut().rebuild_browse_index(engine_connection);
                }
                UIControlEvent::MixerChannelFXProximityPress => {
                    self.as_mut().set_mixer_channel_fx_proximity(true);
                }
                UIControlEvent::MixerChannelFXProximityRelease => {
                    self.as_mut().set_mixer_channel_fx_proximity(false);
                }
                UIControlEvent::MixerMasterFXSelectTouchPress => {
                    self.as_mut().set_mixer_master_fx_proximity(true);
                }
                UIControlEvent::MixerMasterFXSelectTouchRelease => {
                    self.as_mut().set_mixer_master_fx_proximity(false);
                }
            }
        }
    }

    fn select_device(mut self: Pin<&mut Self>, device: i32) {
        self.as_mut().set_active_device(device);
        self.as_mut().set_device_selected(true);

        let Ok(mut engine_connection_lock) = ENGINE_CONNECTION.lock() else {
            return;
        };
        let Some(engine_connection) = engine_connection_lock.as_mut() else {
            return;
        };

        self.as_mut().rebuild_browse_index(engine_connection);

        self.as_mut().set_source_open(false);

        if *self.as_ref().browser_page() == BrowserPage::Closed {
            self.as_mut().set_browser_page(BrowserPage::Device);
        }
    }

    #[allow(clippy::cast_sign_loss)]
    fn load_track(&self, player: i32, device: i32, track_id: i32) {
        let player = player as usize;
        let device = device as usize;
        let track_id = track_id as u32;

        let Ok(mut engine_connection_lock) = ENGINE_CONNECTION.lock() else {
            return;
        };
        let Some(engine_connection) = engine_connection_lock.as_mut() else {
            return;
        };

        let loaded_track_sender = engine_connection.loaded_track_sender.clone();
        let deck_update_sender = engine_connection.deck_update_sender.clone();

        let device_manager = engine_connection.device_manager.subscribe();

        let qt_executor_track_details = self.qt_thread();
        let qt_executor_preview_waveform = self.qt_thread();
        let qt_executor_full_waveform = self.qt_thread();

        engine_connection.tokio_handle.spawn(async move {
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

                let cloned_track_title = track.title.clone();
                let cloned_track_artist = if let Some(artist) = qualified_device
                    .database
                    .library
                    .artists
                    .get(&track.artist_id)
                {
                    artist.name.clone()
                } else {
                    "Unknown".to_string()
                };
                let cloned_track_duration = track.duration;

                let _ = qt_executor_track_details.queue(move |engine_bridge| {
                    // Safety: Probably not
                    let deck_state_pin = unsafe {
                        let Some(deck_state) = engine_bridge.deck_state.as_mut() else {
                            return;
                        };

                        Pin::new_unchecked(&mut *deck_state)
                    };

                    if let Some(mixer_channel) = deck_state_pin.mixer_channels.get(player) {
                        // Safety: Probably not
                        let mixer_channel_pin = unsafe {
                            let Some(mixer_channel) = mixer_channel.as_mut() else {
                                return;
                            };

                            Pin::new_unchecked(&mut *mixer_channel)
                        };

                        // Safety: Probably not
                        let mut player_pin = unsafe {
                            let Some(player) = mixer_channel_pin.player.as_mut() else {
                                return;
                            };

                            Pin::new_unchecked(&mut *player)
                        };

                        player_pin
                            .as_mut()
                            .set_track_name(QString::from(cloned_track_title));
                        player_pin
                            .as_mut()
                            .set_track_artist(QString::from(cloned_track_artist));
                        player_pin.set_track_length(cloned_track_duration);
                    }
                });

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

                    let _ = deck_update_sender.send(Box::new(move |deck_state| {
                        if let Some(mixer_channel) = deck_state.mixer_channels.get_mut(player) {
                            mixer_channel.player.current_track_analysis =
                                Some(cloned_track_analysis);
                        }
                    }));
                } else {
                    std::thread::spawn(move || {
                        // todo: perform track analysis
                    });
                }

                if let Ok(preview_waveform) = qualified_device.database.load_preview_waveform(
                    track_id,
                    libdj::types::analysis::WaveformType::ThreeBand,
                ) {
                    let cloned_preview_waveform = preview_waveform.clone();

                    std::thread::spawn(move || {
                        let image = preview_waveform_to_shader_texture(&cloned_preview_waveform);

                        let _ = qt_executor_preview_waveform.queue(move |engine_bridge| {
                            // Safety: Probably not
                            let deck_state_pin = unsafe {
                                let Some(deck_state) = engine_bridge.deck_state.as_mut() else {
                                    return;
                                };

                                Pin::new_unchecked(&mut *deck_state)
                            };

                            if let Some(mixer_channel) = deck_state_pin.mixer_channels.get(player) {
                                // Safety: Probably not
                                let mixer_channel_pin = unsafe {
                                    let Some(mixer_channel) = mixer_channel.as_mut() else {
                                        return;
                                    };

                                    Pin::new_unchecked(&mut *mixer_channel)
                                };

                                // Safety: Probably not
                                let mut player_pin = unsafe {
                                    let Some(player) = mixer_channel_pin.player.as_mut() else {
                                        return;
                                    };

                                    Pin::new_unchecked(&mut *player)
                                };

                                // Safety: Probably not
                                let mut preview_waveform_texture_source_pin = unsafe {
                                    let Some(preview_waveform_texture_source) =
                                        player_pin.preview_waveform_texture_source.as_mut()
                                    else {
                                        return;
                                    };

                                    Pin::new_unchecked(&mut *preview_waveform_texture_source)
                                };

                                preview_waveform_texture_source_pin
                                    .as_mut()
                                    .set_image(&image);
                            }
                        });
                    });
                }

                if let Ok(waveform) = qualified_device
                    .database
                    .load_waveform(track_id, libdj::types::analysis::WaveformType::ThreeBand)
                {
                    let cloned_waveform = waveform.clone();

                    std::thread::spawn(move || {
                        let image = waveform_to_shader_texture(&cloned_waveform);

                        let _ = qt_executor_full_waveform.queue(move |engine_bridge| {
                            // Safety: Probably not
                            let deck_state_pin = unsafe {
                                let Some(deck_state) = engine_bridge.deck_state.as_mut() else {
                                    return;
                                };

                                Pin::new_unchecked(&mut *deck_state)
                            };

                            if let Some(mixer_channel) = deck_state_pin.mixer_channels.get(player) {
                                // Safety: Probably not
                                let mixer_channel_pin = unsafe {
                                    let Some(mixer_channel) = mixer_channel.as_mut() else {
                                        return;
                                    };

                                    Pin::new_unchecked(&mut *mixer_channel)
                                };

                                // Safety: Probably not
                                let mut player_pin = unsafe {
                                    let Some(player) = mixer_channel_pin.player.as_mut() else {
                                        return;
                                    };

                                    Pin::new_unchecked(&mut *player)
                                };

                                // Safety: Probably not
                                let mut waveform_texture_source_pin = unsafe {
                                    let Some(waveform_texture_source) =
                                        player_pin.waveform_texture_source.as_mut()
                                    else {
                                        return;
                                    };

                                    Pin::new_unchecked(&mut *waveform_texture_source)
                                };

                                waveform_texture_source_pin.as_mut().set_image(&image.0);
                                player_pin.as_mut().set_waveform_texture_stride(image.1);
                                player_pin.as_mut().set_waveform_length_seconds(image.2);
                            }
                        });
                    });
                }
            }

            drop(locked_device_database);
        });
    }

    #[allow(clippy::cast_sign_loss, clippy::unused_self)]
    fn eject_track(&self, player: i32) {
        let player = player as usize;

        let Ok(mut engine_connection_lock) = ENGINE_CONNECTION.lock() else {
            return;
        };
        let Some(engine_connection) = engine_connection_lock.as_mut() else {
            return;
        };

        let _ = engine_connection
            .deck_update_sender
            .send(Box::new(move |deck_state| {
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

        let _ = engine_connection.loaded_track_sender.send((player, None));
    }

    // fn eject_device(self: Pin<&mut EngineBridge>, device: i32) {}

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::unused_self
    )]
    fn touch_cue_move(&self, player: i32, position: f32) {
        let player = player as usize;

        let Ok(mut engine_connection_lock) = ENGINE_CONNECTION.lock() else {
            return;
        };
        let Some(engine_connection) = engine_connection_lock.as_mut() else {
            return;
        };

        let _ = engine_connection
            .deck_update_sender
            .send(Box::new(move |deck_state| {
                if let Some(mixer_channel) = deck_state.mixer_channels.get_mut(player)
                    && let Some((_, track)) = &mixer_channel.player.current_track
                {
                    mixer_channel.player.touch_cue_time = Some(Timecode::from_nanoseconds(
                        (track.duration as f64 * f64::from(position)) as i64,
                    ));
                }
            }));
    }

    #[allow(clippy::cast_sign_loss, clippy::unused_self)]
    fn touch_cue_release(&self, player: i32) {
        let player = player as usize;

        let Ok(mut engine_connection_lock) = ENGINE_CONNECTION.lock() else {
            return;
        };
        let Some(engine_connection) = engine_connection_lock.as_mut() else {
            return;
        };

        let _ = engine_connection
            .deck_update_sender
            .send(Box::new(move |deck_state| {
                if let Some(mixer_channel) = deck_state.mixer_channels.get_mut(player) {
                    mixer_channel.player.touch_cue_time = None;
                }
            }));
    }

    #[allow(clippy::cast_possible_truncation)]
    fn update_from_deck_state(self: Pin<&mut Self>, state: &libdj::types::deck::DeckState) {
        // Safety: Probably not
        let mut deck_state_pin = unsafe {
            let Some(deck_state) = self.deck_state.as_mut() else {
                return;
            };

            Pin::new_unchecked(&mut *deck_state)
        };

        for (channel, new_channel) in deck_state_pin
            .mixer_channels
            .iter()
           	// Safety: Probably not
            .map(|channel| unsafe {
                let channel = channel.as_mut()?;

                Some(Pin::new_unchecked(&mut *channel))
            })
            .zip(&state.mixer_channels)
        {
            let Some(mut channel) = channel else {
                continue;
            };

            // Safety: Probably not
            let mut player_pin = unsafe {
                let Some(player) = channel.player.as_mut() else {
                    return;
                };

                Pin::new_unchecked(&mut *player)
            };

            player_pin
                .as_mut()
                .set_track_loaded(new_channel.player.current_track.is_some());

            if let Some(bpm) = new_channel.player.get_current_bpm() {
                player_pin.as_mut().set_current_bpm(bpm);
            } else {
                player_pin.as_mut().set_current_bpm(0.0);
            }

            player_pin
                .as_mut()
                .set_is_loading(new_channel.player.is_loading);
            player_pin
                .as_mut()
                .set_beat_sync(match new_channel.player.beat_sync {
                    libdj::types::deck::BeatSyncMode::Off => BeatSyncMode::Off,
                    libdj::types::deck::BeatSyncMode::BPMSync => BeatSyncMode::BPMSync,
                    libdj::types::deck::BeatSyncMode::BeatSync => BeatSyncMode::BeatSync,
                });
            player_pin
                .as_mut()
                .set_key_sync(new_channel.player.key_sync);
            player_pin
                .as_mut()
                .set_quanitze(new_channel.player.quanitze);
            player_pin
                .as_mut()
                .set_jog_hold(new_channel.player.jog_hold);
            player_pin
                .as_mut()
                .set_jog_wait(new_channel.player.jog_wait);
            player_pin
                .as_mut()
                .set_jog_velocity(new_channel.player.jog_velocity);
            player_pin
                .as_mut()
                .set_play_state(match new_channel.player.play_state {
                    libdj::types::deck::PlayState::Stop => PlayState::Stop,
                    libdj::types::deck::PlayState::Play => PlayState::Play,
                    libdj::types::deck::PlayState::Cue => PlayState::Cue,
                });
            player_pin
                .as_mut()
                .set_time(new_channel.player.time.nanoseconds);
            player_pin
                .as_mut()
                .set_cue_time_set(new_channel.player.cue_time.is_some());
            player_pin.as_mut().set_cue_time(
                new_channel
                    .player
                    .cue_time
                    .map_or(0, |timecode| timecode.nanoseconds),
            );
            player_pin
                .as_mut()
                .set_touch_cue_time_set(new_channel.player.touch_cue_time.is_some());
            player_pin.as_mut().set_touch_cue_time(
                new_channel
                    .player
                    .touch_cue_time
                    .map_or(0, |timecode| timecode.nanoseconds),
            );
            player_pin
                .as_mut()
                .set_reverse_enabled(new_channel.player.reverse_enabled);
            player_pin
                .as_mut()
                .set_tempo_range(match new_channel.player.tempo_range {
                    libdj::types::deck::TempoRange::SixPercent => TempoRange::SixPercent,
                    libdj::types::deck::TempoRange::TenPercent => TempoRange::TenPercent,
                    libdj::types::deck::TempoRange::SixteenPercent => TempoRange::SixteenPercent,
                    libdj::types::deck::TempoRange::OneHundredPercent => {
                        TempoRange::OneHundredPercent
                    }
                });
            player_pin
                .as_mut()
                .set_tempo_reset(new_channel.player.tempo_reset);
            player_pin
                .as_mut()
                .set_tempo_percent(new_channel.player.tempo_percent);
            player_pin
                .as_mut()
                .set_tempo_slider_position(new_channel.player.tempo_slider_position);
            player_pin
                .as_mut()
                .set_tempo_slider_is_accurate(new_channel.player.tempo_slider_is_accurate);
            player_pin
                .as_mut()
                .set_master_tempo(new_channel.player.master_tempo);
            player_pin.as_mut().set_slip(new_channel.player.slip);
            player_pin
                .as_mut()
                .set_slip_playing(new_channel.player.slip_playing);
            player_pin
                .as_mut()
                .set_slip_time(new_channel.player.slip_time.nanoseconds);
            player_pin
                .as_mut()
                .set_beat_loop_start_set(new_channel.player.beat_loop_start.is_some());
            player_pin.as_mut().set_beat_loop_start(
                new_channel
                    .player
                    .beat_loop_start
                    .map_or(0, |timecode| timecode.nanoseconds),
            );
            player_pin
                .as_mut()
                .set_beat_loop_end_set(new_channel.player.beat_loop_end.is_some());
            player_pin.as_mut().set_beat_loop_end(
                new_channel
                    .player
                    .beat_loop_end
                    .map_or(0, |timecode| timecode.nanoseconds),
            );
            player_pin
                .as_mut()
                .set_last_beat_loop_set(new_channel.player.last_beat_loop.is_some());
            player_pin.as_mut().set_last_beat_loop_start(
                new_channel
                    .player
                    .last_beat_loop
                    .map_or(0, |timecode| timecode.0.nanoseconds),
            );
            player_pin.as_mut().set_last_beat_loop_end(
                new_channel
                    .player
                    .last_beat_loop
                    .map_or(0, |timecode| timecode.1.nanoseconds),
            );
            player_pin.as_mut().set_beat_loop_adjust_mode(
                match new_channel.player.beat_loop_adjust_mode {
                    libdj::types::deck::BeatLoopAdjustMode::None => BeatLoopAdjustMode::None,
                    libdj::types::deck::BeatLoopAdjustMode::In => BeatLoopAdjustMode::In,
                    libdj::types::deck::BeatLoopAdjustMode::Out => BeatLoopAdjustMode::Out,
                },
            );
            player_pin
                .as_mut()
                .set_keyshift(new_channel.player.keyshift);

            channel.as_mut().set_gain(new_channel.gain);
            channel.as_mut().set_eq_low(new_channel.eq_low);
            channel.as_mut().set_eq_mid(new_channel.eq_mid);
            channel.as_mut().set_eq_high(new_channel.eq_high);
            channel.as_mut().set_fx(new_channel.fx);
            channel.as_mut().set_cue(new_channel.cue);
            channel.as_mut().set_fade(new_channel.fade);
            channel
                .as_mut()
                .set_cross_fader_side(match new_channel.cross_fader_side {
                    libdj::types::deck::CrossFaderSide::A => CrossFaderSide::A,
                    libdj::types::deck::CrossFaderSide::B => CrossFaderSide::B,
                    libdj::types::deck::CrossFaderSide::None => CrossFaderSide::None,
                });
        }

        deck_state_pin
            .as_mut()
            .set_master_channel_set(state.master_channel.is_some());
        deck_state_pin
            .as_mut()
            .set_master_channel(state.master_channel.unwrap_or(0) as i32);
        deck_state_pin.as_mut().set_crossfade(state.crossfade);
        deck_state_pin.as_mut().set_master_cue(state.master_cue);
        deck_state_pin.as_mut().set_master_gain(state.master_gain);
        deck_state_pin
            .as_mut()
            .set_channel_fx_effect(match state.channel_fx_effect {
                libdj::types::deck::ChannelFXEffect::None => ChannelFXEffect::None,
                libdj::types::deck::ChannelFXEffect::Space => ChannelFXEffect::Space,
                libdj::types::deck::ChannelFXEffect::DubEcho => ChannelFXEffect::DubEcho,
                libdj::types::deck::ChannelFXEffect::Bitcrush => ChannelFXEffect::Bitcrush,
                libdj::types::deck::ChannelFXEffect::Pitch => ChannelFXEffect::Pitch,
                libdj::types::deck::ChannelFXEffect::Noise => ChannelFXEffect::Noise,
                libdj::types::deck::ChannelFXEffect::Filter => ChannelFXEffect::Filter,
            });
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap
    )]
    fn device_connected(
        self: Pin<&mut Self>,
        engine_connection: &mut EngineConnection,
        number: usize,
    ) {
        let label = engine_connection
            .device_manager
            .devices
            .blocking_lock()
            .get(&number)
            .map_or("Unknown".to_string(), |device| device.name.clone());

        // Safety: Probably not
        let mut source_index_pin = unsafe {
            let Some(source_index) = self.source_index.as_mut() else {
                return;
            };

            Pin::new_unchecked(&mut *source_index)
        };

        if let Some((updated_item, existing_device)) = source_index_pin
            .as_mut()
            .rust_mut()
            .items
            .iter_mut()
            .enumerate()
            .find(|(_, device)| device.device_number == (number as i32))
        {
            existing_device.label = QString::from(label);

            let updated_item =
                source_index_pin
                    .as_mut()
                    .index(updated_item as i32, 0, &QModelIndex::default());

            source_index_pin.as_mut().data_changed(
                &updated_item,
                &updated_item,
                &QVector_i32::from(vec![SOURCE_LIST_MODEL_LABEL_ROLE]),
            );
        } else {
            let updated_item = source_index_pin.items.len() as i32;

            source_index_pin.as_mut().begin_insert_rows(
                &QModelIndex::default(),
                updated_item,
                updated_item,
            );

            source_index_pin
                .as_mut()
                .rust_mut()
                .items
                .push(SourceListEntry {
                    device_number: number as i32,
                    label: QString::from(label),
                });

            source_index_pin.as_mut().end_insert_rows();
        }
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap
    )]
    fn device_disconnected(mut self: Pin<&mut Self>, number: usize) {
        if self.device_selected {
            self.as_mut().set_device_selected(false);
            self.as_mut().set_active_device(0);
        }

        if self.browser_page != BrowserPage::Closed {
            self.as_mut().set_source_open(true);
        }

        // Safety: Probably not
        let mut source_index_pin = unsafe {
            let Some(source_index) = self.source_index.as_mut() else {
                return;
            };

            Pin::new_unchecked(&mut *source_index)
        };

        source_index_pin.as_mut().begin_reset_model();

        let mut source_index_rust = source_index_pin.as_mut().rust_mut();

        while let Some(index) = source_index_rust
            .items
            .iter()
            .position(|device| device.device_number == (number as i32))
        {
            let _ = source_index_rust.items.remove(index);
        }

        source_index_pin.as_mut().end_reset_model();
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap
    )]
    fn rebuild_browse_index(mut self: Pin<&mut Self>, engine_connection: &mut EngineConnection) {
        // Safety: Probably not
        let mut browser_index_pin = unsafe {
            let Some(browser_index) = self.browser_index.as_mut() else {
                return;
            };

            Pin::new_unchecked(&mut *browser_index)
        };

        if !self.device_selected {
            self.as_mut().set_active_device(0);

            browser_index_pin.as_mut().begin_reset_model();

            browser_index_pin.as_mut().rust_mut().items.clear();

            browser_index_pin.as_mut().end_reset_model();

            return;
        }

        let locked_device_manager = engine_connection.device_manager.devices.blocking_lock();

        let Some(device) = locked_device_manager.get(&(self.active_device as usize)) else {
            drop(locked_device_manager);

            self.as_mut().set_device_selected(false);
            self.as_mut().set_active_device(0);

            browser_index_pin.as_mut().begin_reset_model();

            browser_index_pin.as_mut().rust_mut().items.clear();

            browser_index_pin.as_mut().end_reset_model();

            return;
        };

        browser_index_pin.as_mut().begin_reset_model();

        match self.browser_page {
            BrowserPage::Device => {
                browser_index_pin.as_mut().rust_mut().items.clear();

                for (index, track) in device.database.library.tracks.values().enumerate() {
                    browser_index_pin
                        .as_mut()
                        .rust_mut()
                        .items
                        .push(BrowserListEntry {
                            entry_type: BrowserEntryType::Track,
                            entry_number: index as i32,
                            node_id: track.id as i32,
                            title: QString::from(track.title.clone()),
                            artist: QString::from(
                                device
                                    .database
                                    .library
                                    .artists
                                    .get(&track.artist_id)
                                    .map_or("Unknown".to_string(), |artist| artist.name.clone()),
                            ),
                            duration: track.duration,
                            bpm: track.bpm,
                        });
                }
            }
            BrowserPage::Playlist => {
                browser_index_pin.as_mut().rust_mut().items.clear();
            }
            BrowserPage::Search => {
                browser_index_pin.as_mut().rust_mut().items.clear();

                let search = &self.search.to_string();

                for (index, track) in device
                    .database
                    .library
                    .tracks
                    .values()
                    .filter(|track| {
                        // todo: build a better index and fuzzy match
                        track.title.contains(search)
                    })
                    .enumerate()
                {
                    browser_index_pin
                        .as_mut()
                        .rust_mut()
                        .items
                        .push(BrowserListEntry {
                            entry_type: BrowserEntryType::Track,
                            entry_number: index as i32,
                            node_id: track.id as i32,
                            title: QString::from(track.title.clone()),
                            artist: QString::from(
                                device
                                    .database
                                    .library
                                    .artists
                                    .get(&track.artist_id)
                                    .map_or("Unknown".to_string(), |artist| artist.name.clone()),
                            ),
                            duration: track.duration,
                            bpm: track.bpm,
                        });
                }
            }
            _ => {}
        }

        browser_index_pin.as_mut().end_reset_model();

        drop(locked_device_manager);
    }
}

const SOURCE_LIST_MODEL_DEVICE_NUMBER_ROLE: i32 = 0;
const SOURCE_LIST_MODEL_LABEL_ROLE: i32 = 1;

#[derive(PartialEq)]
pub struct SourceListEntry {
    pub device_number: i32,
    pub label: QString,
}

#[derive(Default)]
pub struct SourceListModelRust {
    pub items: Vec<SourceListEntry>,
}

impl qobject::SourceListModel {
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    pub fn row_count(&self, _: &QModelIndex) -> i32 {
        self.items.len() as i32
    }

    #[allow(clippy::unused_self)]
    pub fn role_names(&self) -> cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray> {
        let mut roles = QHash::default();

        roles.insert(
            SOURCE_LIST_MODEL_DEVICE_NUMBER_ROLE,
            QByteArray::from("device_number"),
        );
        roles.insert(SOURCE_LIST_MODEL_LABEL_ROLE, QByteArray::from("label"));

        roles
    }

    #[allow(clippy::cast_sign_loss)]
    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        match self.items.get(index.row() as usize) {
            Some(item) if role == SOURCE_LIST_MODEL_DEVICE_NUMBER_ROLE => {
                QVariant::from(&item.device_number)
            }
            Some(item) if role == SOURCE_LIST_MODEL_LABEL_ROLE => QVariant::from(&item.label),
            _ => QVariant::default(),
        }
    }
}

const BROWSER_LIST_MODEL_ENTRY_TYPE_ROLE: i32 = 0;
const BROWSER_LIST_MODEL_ENTRY_NUMBER_ROLE: i32 = 1;
const BROWSER_LIST_MODEL_NODE_ID_ROLE: i32 = 2;
const BROWSER_LIST_MODEL_TITLE_ROLE: i32 = 3;
const BROWSER_LIST_MODEL_ARTIST_ROLE: i32 = 4;
const BROWSER_LIST_MODEL_DURATION_ROLE: i32 = 5;
const BROWSER_LIST_MODEL_BPM_ROLE: i32 = 6;

#[derive(PartialEq)]
pub struct BrowserListEntry {
    pub entry_type: BrowserEntryType,
    pub entry_number: i32,
    pub node_id: i32,
    pub title: QString,
    pub artist: QString,
    pub duration: i64, // nanoseconds
    pub bpm: f32,
}

#[derive(Default)]
pub struct BrowserListModelRust {
    pub items: Vec<BrowserListEntry>,
}

impl qobject::BrowserListModel {
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    pub fn row_count(&self, _: &QModelIndex) -> i32 {
        self.items.len() as i32
    }

    #[allow(clippy::unused_self)]
    pub fn role_names(&self) -> cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray> {
        let mut roles = QHash::default();

        roles.insert(
            BROWSER_LIST_MODEL_ENTRY_TYPE_ROLE,
            QByteArray::from("entry_type"),
        );
        roles.insert(
            BROWSER_LIST_MODEL_ENTRY_NUMBER_ROLE,
            QByteArray::from("entry_number"),
        );
        roles.insert(BROWSER_LIST_MODEL_NODE_ID_ROLE, QByteArray::from("node_id"));
        roles.insert(BROWSER_LIST_MODEL_TITLE_ROLE, QByteArray::from("title"));
        roles.insert(BROWSER_LIST_MODEL_ARTIST_ROLE, QByteArray::from("artist"));
        roles.insert(
            BROWSER_LIST_MODEL_DURATION_ROLE,
            QByteArray::from("duration"),
        );
        roles.insert(BROWSER_LIST_MODEL_BPM_ROLE, QByteArray::from("bpm"));

        roles
    }

    #[allow(clippy::cast_sign_loss)]
    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        match self.items.get(index.row() as usize) {
            Some(item) if role == BROWSER_LIST_MODEL_ENTRY_TYPE_ROLE => {
                QVariant::from(&item.entry_type.repr)
            }
            Some(item) if role == BROWSER_LIST_MODEL_ENTRY_NUMBER_ROLE => {
                QVariant::from(&item.entry_number)
            }
            Some(item) if role == BROWSER_LIST_MODEL_NODE_ID_ROLE => QVariant::from(&item.node_id),
            Some(item) if role == BROWSER_LIST_MODEL_TITLE_ROLE => QVariant::from(&item.title),
            Some(item) if role == BROWSER_LIST_MODEL_ARTIST_ROLE => QVariant::from(&item.artist),
            Some(item) if role == BROWSER_LIST_MODEL_DURATION_ROLE => {
                QVariant::from(&item.duration)
            }
            Some(item) if role == BROWSER_LIST_MODEL_BPM_ROLE => QVariant::from(&item.bpm),
            _ => QVariant::default(),
        }
    }
}
