#![feature(vec_try_remove)]

pub mod ipc;
pub mod types;

use godot::prelude::*;

struct MyExtension;

/// safety: godot forces this to be unsafe, but this should be safe in all
/// reasonable contexts
#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
