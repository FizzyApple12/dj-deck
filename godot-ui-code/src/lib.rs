pub mod browser;
pub mod ipc;
pub mod menu;
pub mod player;
pub mod types;

use godot::prelude::*;

struct MyExtension;

/// safety: godot forces this to be unsafe, but this should be safe in all
/// reasonable contexts
#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
