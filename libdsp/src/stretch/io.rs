use std::{
    marker::PhantomData,
    ops::{Index, IndexMut, RangeFrom},
};

use num::Float;

// First Level Traits

pub trait IoBuffer<'s, Channel> {
    fn channel(&'s self, channel_number: usize) -> Channel;
}

pub trait IoBufferMut<'s, ChannelMut> {
    fn channel_mut(&'s mut self, channel_number: usize) -> ChannelMut;
}

// First Level Traits
// BufferSplitterIo

pub struct BufferSplitterIo<'a, Data, InnerData, Sample>
where
    Data: Index<RangeFrom<usize>, Output = InnerData>,
    InnerData: Index<usize, Output = Sample>,
    Sample: Float,
{
    pub data: &'a Data,
    pub phantom_data: PhantomData<InnerData>,
    pub length: usize,
}

impl<'a, Data, InnerData> BufferSplitterIo<'a, Data, InnerData, f32>
where
    Data: Index<RangeFrom<usize>, Output = InnerData>,
    InnerData: Index<usize, Output = f32>,
{
    pub fn new(data: &'a Data, length: usize) -> Self {
        BufferSplitterIo {
            data,
            phantom_data: PhantomData,
            length,
        }
    }
}

impl<'s, 'a: 's, Data, InnerData> IoBuffer<'s, InnerBufferSplitterIo<'s, InnerData, f32>>
    for BufferSplitterIo<'a, Data, InnerData, f32>
where
    Data: Index<RangeFrom<usize>, Output = InnerData>,
    InnerData: Index<usize, Output = f32>,
{
    fn channel(&'s self, channel_number: usize) -> InnerBufferSplitterIo<'s, InnerData, f32> {
        InnerBufferSplitterIo {
            data: &self.data[(channel_number * self.length)..],
            length: self.length,
        }
    }
}

// BufferSplitterIo
// InnerBufferSplitterIo

pub struct InnerBufferSplitterIo<'a, InnerData, Sample>
where
    InnerData: Index<usize, Output = Sample>,
    Sample: Float,
{
    pub data: &'a InnerData,
    pub length: usize,
}

impl<InnerData> Index<usize> for InnerBufferSplitterIo<'_, InnerData, f32>
where
    InnerData: Index<usize, Output = f32>,
{
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

// InnerBufferSplitterIo
// BufferSplitterIoMut

pub struct BufferSplitterIoMut<'a, Data, InnerData, Sample>
where
    Data: IndexMut<RangeFrom<usize>, Output = InnerData>,
    InnerData: IndexMut<usize, Output = Sample>,
    Sample: Float,
{
    pub data: &'a mut Data,
    pub length: usize,
}

impl<'a, Data, InnerData> BufferSplitterIoMut<'a, Data, InnerData, f32>
where
    Data: IndexMut<RangeFrom<usize>, Output = InnerData>,
    InnerData: IndexMut<usize, Output = f32>,
{
    pub fn new(data: &'a mut Data, length: usize) -> Self {
        BufferSplitterIoMut { data, length }
    }
}

impl<'s, 'a: 's, Data, InnerData> IoBuffer<'s, InnerBufferSplitterIo<'s, InnerData, f32>>
    for BufferSplitterIoMut<'a, Data, InnerData, f32>
where
    Data: IndexMut<RangeFrom<usize>, Output = InnerData>,
    InnerData: IndexMut<usize, Output = f32>,
{
    fn channel(&'s self, channel_number: usize) -> InnerBufferSplitterIo<'s, InnerData, f32> {
        InnerBufferSplitterIo {
            data: &self.data[(channel_number * self.length)..],
            length: self.length,
        }
    }
}

impl<'s, 'a: 's, Data, InnerData> IoBufferMut<'s, InnerBufferSplitterIoMut<'s, InnerData, f32>>
    for BufferSplitterIoMut<'a, Data, InnerData, f32>
where
    Data: IndexMut<RangeFrom<usize>, Output = InnerData>,
    InnerData: IndexMut<usize, Output = f32>,
{
    fn channel_mut(
        &'s mut self,
        channel_number: usize,
    ) -> InnerBufferSplitterIoMut<'s, InnerData, f32> {
        InnerBufferSplitterIoMut {
            data: &mut self.data[(channel_number * self.length)..],
            length: self.length,
        }
    }
}

// BufferSplitterIoMut
// InnerBufferSplitterIoMut

pub struct InnerBufferSplitterIoMut<'a, InnerData, Sample>
where
    InnerData: IndexMut<usize, Output = Sample>,
    Sample: Float,
{
    pub data: &'a mut InnerData,
    pub length: usize,
}

impl<InnerData> IndexMut<usize> for InnerBufferSplitterIoMut<'_, InnerData, f32>
where
    InnerData: IndexMut<usize, Output = f32>,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl<InnerData> Index<usize> for InnerBufferSplitterIoMut<'_, InnerData, f32>
where
    InnerData: IndexMut<usize, Output = f32>,
{
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

// InnerBufferSplitterIoMut
// // OffsetIo

// #[derive(Clone, Copy)]
// pub struct OffsetIo<'a, Data, InnerData, Sample>
// where
//     Data: Index<usize, Output = InnerData>,
//     InnerData: Index<usize, Output = Sample>,
//     Sample: Float,
// {
//     pub data: &'a Data,
//     pub phantom_data: PhantomData<InnerData>,
//     pub offset: usize,
// }

// impl<'a, Data, InnerData> OffsetIo<'a, Data, InnerData, f32>
// where
//     Data: Index<usize, Output = InnerData>,
//     InnerData: Index<usize, Output = f32>,
// {
//     pub fn new(data: &'a Data, offset: usize) -> Self {
//         OffsetIo {
//             data,
//             phantom_data: PhantomData,
//             offset,
//         }
//     }
// }

// impl<'a, Data, InnerData> Index<usize> for OffsetIo<'a, Data, InnerData, f32>
// where
//     Data: Index<usize, Output = InnerData>,
//     InnerData: Index<usize, Output = f32> + 'a,
// {
//     type Output = InnerOffsetIo<'a, InnerData, f32>;

//     fn index(&self, index: usize) -> &Self::Output {
//         &InnerOffsetIo {
//             data: &self.data[index],
//             offset: self.offset,
//         }
//     }
// }

// // OffsetIo
// // OffsetIoMut

// pub struct OffsetIoMut<'a, Data, InnerData, Sample>
// where
//     Data: IndexMut<usize, Output = InnerData>,
//     InnerData: IndexMut<usize, Output = Sample>,
//     Sample: Float,
// {
//     pub data: &'a mut Data,
//     pub phantom_data: PhantomData<InnerData>,
//     pub offset: usize,
// }

// impl<'a, Data, InnerData> OffsetIoMut<'a, Data, InnerData, f32>
// where
//     Data: IndexMut<usize, Output = InnerData>,
//     InnerData: IndexMut<usize, Output = f32>,
// {
//     pub fn new(data: &'a mut Data, offset: usize) -> Self {
//         OffsetIoMut {
//             data,
//             phantom_data: PhantomData,
//             offset,
//         }
//     }
// }

// impl<Data, InnerData> IndexMut<usize> for OffsetIoMut<'_, Data, InnerData,
// f32> where
//     Data: IndexMut<usize, Output = InnerData>,
//     InnerData: IndexMut<usize, Output = f32>,
// {
//     fn index_mut(&mut self, index: usize) -> &mut Self::Output {
//         // &mut self.data[index]
//         todo!("return InnerOffsetIoMut")
//     }
// }

// impl<Data, InnerData> Index<usize> for OffsetIoMut<'_, Data, InnerData, f32>
// where
//     Data: IndexMut<usize, Output = InnerData>,
//     InnerData: IndexMut<usize, Output = f32>,
// {
//     type Output = InnerData;

//     fn index(&self, index: usize) -> &Self::Output {
//         // &self.data[index]
//         todo!("return InnerOffsetIo")
//     }
// }

// // OffsetIoMut
// InnerOffsetIo

pub struct InnerOffsetIo<'a, InnerData, Sample>
where
    InnerData: Index<usize, Output = Sample>,
    Sample: Float,
{
    pub data: &'a InnerData,
    pub offset: usize,
}

impl<InnerData> Index<usize> for InnerOffsetIo<'_, InnerData, f32>
where
    InnerData: Index<usize, Output = f32>,
{
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index + self.offset]
    }
}

// InnerOffsetIo
// InnerOffsetIoMut

pub struct InnerOffsetIoMut<'a, InnerData, Sample>
where
    InnerData: IndexMut<usize, Output = Sample>,
    Sample: Float,
{
    pub data: &'a mut InnerData,
    pub offset: usize,
}

impl<InnerData> Index<usize> for InnerOffsetIoMut<'_, InnerData, f32>
where
    InnerData: IndexMut<usize, Output = f32>,
{
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index + self.offset]
    }
}

impl<InnerData> IndexMut<usize> for InnerOffsetIoMut<'_, InnerData, f32>
where
    InnerData: IndexMut<usize, Output = f32>,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index + self.offset]
    }
}

// InnerOffsetIoMut
// ZeroIo

pub struct ZeroIo<ZeroType> {
    pub phantom_data: PhantomData<ZeroType>,
}

impl<'s, 'a: 's, ZeroType> IoBuffer<'s, InnerZeroIo<ZeroType>> for ZeroIo<ZeroType> {
    fn channel(&'s self, _: usize) -> InnerZeroIo<ZeroType> {
        InnerZeroIo::<ZeroType> {
            phantom_data: PhantomData,
        }
    }
}

pub struct InnerZeroIo<ZeroType> {
    pub phantom_data: PhantomData<ZeroType>,
}

impl Index<usize> for InnerZeroIo<f32> {
    type Output = f32;

    fn index(&self, _: usize) -> &Self::Output {
        &0.0
    }
}

// ZeroIo
