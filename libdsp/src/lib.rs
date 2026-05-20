#![allow(incomplete_features)]
#![feature(inherent_associated_types)]
#![feature(generic_const_exprs)]
#![feature(stdarch_arm_feature_detection)]

// this library is ported from:
// https://github.com/Signalsmith-Audio/linear
// https://signalsmith-audio.co.uk/code/dsp/
// https://signalsmith-audio.co.uk/code/stretch/
// thank you signalsmith for your excellent work, you've saved my ass

// TODO: ADD f64 VARIANTS

pub mod dsp;
pub mod linear;
pub mod stretch;
