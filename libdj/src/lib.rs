#![feature(unboxed_closures, fn_traits)]

use std::path::PathBuf;

use rkyv::{
    Deserialize, Place,
    rancor::{Fallible, Source},
    ser::Writer,
    string::{ArchivedString, StringResolver},
    with::{ArchiveWith, DeserializeWith, SerializeWith},
};

pub mod audio_system;
pub mod math;
pub mod playback;
pub mod types;

// pub const PLAYBACK_SAMPLE_RATE: usize = 44100;
// pub const MAX_BUFFER_SIZE: usize = 128;

pub const MIXER_CHANNELS: usize = 4;
pub const AUDIO_CHANNELS: usize = 2;

pub const BASE_GAIN_DB: f32 = -14.0;

pub const MIXER_MIN_FREQUENCY: f32 = 0.0;
pub const MIXER_MAX_FREQUENCY: f32 = 22000.0;

// pub const MIXER_EQ_LOW_CUTOFF: f32 = 880.0;
// pub const MIXER_EQ_LOW_CUTOFF: f32 = 250.0;
pub const MIXER_EQ_LOW_CUTOFF: f64 = 100.0;
pub const MIXER_EQ_LOW_OCTAVES: f64 = 2.0;

// pub const MIXER_EQ_MID_CENTER: f32 = 2000.0;
// pub const MIXER_EQ_MID_CENTER: f32 = 1000.0;
pub const MIXER_EQ_MID_CENTER: f64 = 1000.0;
pub const MIXER_EQ_MID_Q: f64 = 0.7;

// pub const MIXER_EQ_HIGH_CUTOFF: f32 = 5000.0;
// pub const MIXER_EQ_HIGH_CUTOFF: f32 = 3000.0;
pub const MIXER_EQ_HIGH_CUTOFF: f64 = 10000.0;
pub const MIXER_EQ_HIGH_OCTAVES: f64 = 1.899_968_626_952_991_6;

pub const MIXER_FILTER_LOW_PASS_OCTAVES: f64 = 1.899_968_626_952_991_6;
pub const MIXER_FILTER_HIGH_PASS_OCTAVES: f64 = 1.899_968_626_952_991_6;

pub struct PathBufAsString;

impl ArchiveWith<PathBuf> for PathBufAsString {
    type Archived = ArchivedString;
    type Resolver = StringResolver;

    fn resolve_with(field: &PathBuf, resolver: Self::Resolver, place: Place<ArchivedString>) {
        let path_string = field.to_string_lossy();

        ArchivedString::resolve_from_str(&path_string, resolver, place);
    }
}

impl<S> SerializeWith<PathBuf, S> for PathBufAsString
where
    S: Fallible + ?Sized + Writer,
    S::Error: Source,
{
    fn serialize_with(field: &PathBuf, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        let path_string = field.to_string_lossy();

        ArchivedString::serialize_from_str(&path_string, serializer)
    }
}

impl<D> DeserializeWith<ArchivedString, PathBuf, D> for PathBufAsString
where
    D: Fallible + ?Sized,
{
    fn deserialize_with(field: &ArchivedString, deserializer: &mut D) -> Result<PathBuf, D::Error> {
        let deserialized_string = ArchivedString::deserialize(field, deserializer)?;

        Ok(PathBuf::from(deserialized_string))
    }
}
