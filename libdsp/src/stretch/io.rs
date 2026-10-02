use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use num::Float;

// First Level Traits

pub trait IoBuffer<Sample> {
    fn sample(&self, channel_number: usize, sample_number: usize) -> &Sample;
}

pub trait IoBufferMut<Sample>: IoBuffer<Sample> {
    fn sample_mut(&mut self, channel_number: usize, sample_number: usize) -> &mut Sample;
}

// First Level Traits
// BufferSplitterIo

pub struct BufferSplitterIo<'a, Data, Sample>
where
    Data: Index<usize, Output = Sample>,
    Sample: Float,
{
    pub data: &'a Data,
    pub length: usize,
}

impl<'a, Data> BufferSplitterIo<'a, Data, f32>
where
    Data: Index<usize, Output = f32>,
{
    pub fn new(data: &'a Data, length: usize) -> Self {
        BufferSplitterIo { data, length }
    }
}

impl<Data> IoBuffer<f32> for BufferSplitterIo<'_, Data, f32>
where
    Data: Index<usize, Output = f32>,
{
    fn sample(&self, channel_number: usize, sample_number: usize) -> &f32 {
        &self.data[(channel_number * self.length) + sample_number]
    }
}

// BufferSplitterIo
// BufferSplitterIoMut

pub struct BufferSplitterIoMut<'a, Data, Sample>
where
    Data: IndexMut<usize, Output = Sample>,
    Sample: Float,
{
    pub data: &'a mut Data,
    pub length: usize,
}

impl<'a, Data> BufferSplitterIoMut<'a, Data, f32>
where
    Data: IndexMut<usize, Output = f32>,
{
    pub fn new(data: &'a mut Data, length: usize) -> Self {
        BufferSplitterIoMut { data, length }
    }
}

impl<Data> IoBuffer<f32> for BufferSplitterIoMut<'_, Data, f32>
where
    Data: IndexMut<usize, Output = f32>,
{
    fn sample(&self, channel_number: usize, sample_number: usize) -> &f32 {
        &self.data[(channel_number * self.length) + sample_number]
    }
}

impl<Data> IoBufferMut<f32> for BufferSplitterIoMut<'_, Data, f32>
where
    Data: IndexMut<usize, Output = f32>,
{
    fn sample_mut(&mut self, channel_number: usize, sample_number: usize) -> &mut f32 {
        &mut self.data[(channel_number * self.length) + sample_number]
    }
}

// BufferSplitterIoMut
// IoBufferOffsetIo

#[derive(Clone, Copy)]
pub struct IoBufferOffsetIo<'a, Data, Sample>
where
    Data: IoBuffer<Sample>,
    Sample: Float,
{
    pub data: &'a Data,
    pub phantom_data: PhantomData<Sample>,
    pub offset: usize,
}

impl<'a, Data> IoBufferOffsetIo<'a, Data, f32>
where
    Data: IoBuffer<f32>,
{
    pub fn new(data: &'a Data, offset: usize) -> Self {
        IoBufferOffsetIo {
            data,
            phantom_data: PhantomData,
            offset,
        }
    }
}

impl<Data> IoBuffer<f32> for IoBufferOffsetIo<'_, Data, f32>
where
    Data: IoBuffer<f32>,
{
    fn sample(&self, channel_number: usize, sample_number: usize) -> &f32 {
        self.data
            .sample(channel_number, sample_number + self.offset)
    }
}

// IoBufferOffsetIo
// IoBufferOffsetIo

pub struct IoBufferOffsetIoMut<'a, Data, Sample>
where
    Data: IoBufferMut<Sample>,
    Sample: Float,
{
    pub data: &'a mut Data,
    pub phantom_data: PhantomData<Sample>,
    pub offset: usize,
}

impl<'a, Data> IoBufferOffsetIoMut<'a, Data, f32>
where
    Data: IoBufferMut<f32>,
{
    pub fn new(data: &'a mut Data, offset: usize) -> Self {
        IoBufferOffsetIoMut {
            data,
            phantom_data: PhantomData,
            offset,
        }
    }
}

impl<Data> IoBuffer<f32> for IoBufferOffsetIoMut<'_, Data, f32>
where
    Data: IoBufferMut<f32>,
{
    fn sample(&self, channel_number: usize, sample_number: usize) -> &f32 {
        self.data
            .sample(channel_number, sample_number + self.offset)
    }
}

impl<Data> IoBufferMut<f32> for IoBufferOffsetIoMut<'_, Data, f32>
where
    Data: IoBufferMut<f32>,
{
    fn sample_mut(&mut self, channel_number: usize, sample_number: usize) -> &mut f32 {
        self.data
            .sample_mut(channel_number, sample_number + self.offset)
    }
}

// IoBufferOffsetIo
// ZeroIo

pub struct ZeroIo<ZeroType> {
    pub phantom_data: PhantomData<ZeroType>,
}

impl Default for ZeroIo<f32> {
    fn default() -> Self {
        ZeroIo {
            phantom_data: PhantomData,
        }
    }
}

impl IoBuffer<f32> for ZeroIo<f32> {
    fn sample(&self, _: usize, _: usize) -> &f32 {
        &0.0
    }
}

// ZeroIo
