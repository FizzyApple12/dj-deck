use libdsp::stretch::io::{IoBuffer, IoBufferMut};

use crate::AUDIO_CHANNELS;

pub struct VecIoBuffer<'a> {
    pub data: &'a [Vec<f32>; AUDIO_CHANNELS],
}

impl<'a> VecIoBuffer<'a> {
    pub fn new(data: &'a [Vec<f32>; AUDIO_CHANNELS]) -> Self {
        VecIoBuffer { data }
    }
}

impl IoBuffer<f32> for VecIoBuffer<'_> {
    #[allow(clippy::indexing_slicing)]
    fn sample(&self, channel_number: usize, sample_number: usize) -> &f32 {
        &self.data[channel_number][sample_number]
    }
}

pub struct VecIoBufferMut<'a> {
    pub data: &'a mut [Vec<f32>; AUDIO_CHANNELS],
}

impl<'a> VecIoBufferMut<'a> {
    pub fn new(data: &'a mut [Vec<f32>; AUDIO_CHANNELS]) -> Self {
        VecIoBufferMut { data }
    }
}

impl IoBuffer<f32> for VecIoBufferMut<'_> {
    #[allow(clippy::indexing_slicing)]
    fn sample(&self, channel_number: usize, sample_number: usize) -> &f32 {
        &self.data[channel_number][sample_number]
    }
}

impl IoBufferMut<f32> for VecIoBufferMut<'_> {
    #[allow(clippy::indexing_slicing)]
    fn sample_mut(&mut self, channel_number: usize, sample_number: usize) -> &mut f32 {
        &mut self.data[channel_number][sample_number]
    }
}
