use std::path::PathBuf;

use rkyv::{
    Deserialize, Place,
    rancor::{Fallible, Source},
    ser::Writer,
    string::{ArchivedString, StringResolver},
    with::{ArchiveWith, DeserializeWith, SerializeWith},
};

pub mod types;

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
