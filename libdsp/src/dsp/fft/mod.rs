pub mod fft;
pub mod real_fft;

use std::ops::{Index, IndexMut};

use num::{Complex, Float};

// Complex multiplication has edge-cases around Inf/NaN - handling those
// properly makes std::complex non-inlineable, so we use our own
#[inline]
pub fn complexMul<const CONJUGATE_SECOND: bool, V>(a: &Complex<V>, b: &Complex<V>) -> Complex<V>
where
    V: Float,
{
    if CONJUGATE_SECOND {
        Complex::<V>::new(b.re * a.re + b.im * a.im, b.re * a.im - b.im * a.re)
    } else {
        Complex::<V>::new(a.re * b.re - a.im * b.im, a.re * b.im + a.im * b.re)
    }
}

#[inline]
pub fn complexAddI<const FLIPPED: bool, V>(a: &Complex<V>, b: &Complex<V>) -> Complex<V>
where
    V: Float,
{
    if FLIPPED {
        Complex::<V>::new(a.re + b.im, a.im - b.re)
    } else {
        Complex::<V>::new(a.re - b.im, a.im + b.re)
    }
}

pub trait FFTTrait<Sample, RawBufferType, FFTBufferType>
where
    Sample: Float,
{
    type Sample;
    type Complex;

    fn fast_size_above(size: usize) -> usize;
    fn fast_size_below(size: usize) -> usize;

    fn new(size: usize, fast_direction: i32) -> Self;

    fn set_size(&mut self, size: usize) -> usize;
    fn set_fast_size_above(&mut self, size: usize) -> usize;
    fn set_fast_size_below(&mut self, size: usize) -> usize;
    fn size(&self) -> usize;

    fn fft<InputBuffer, OutputBuffer>(&mut self, input: &InputBuffer, output: &mut OutputBuffer)
    where
        InputBuffer: Index<usize, Output = RawBufferType>,
        OutputBuffer: IndexMut<usize, Output = FFTBufferType>;

    fn ifft<InputBuffer, OutputBuffer>(&mut self, input: &InputBuffer, output: &mut OutputBuffer)
    where
        InputBuffer: Index<usize, Output = FFTBufferType>,
        OutputBuffer: IndexMut<usize, Output = RawBufferType>;
}

pub struct IndexOffset<'a, Data, Element>
where
    Data: Index<usize, Output = Element>,
{
    pub data: &'a Data,
    pub offset: usize,
}

impl<'a, Data, Element> IndexOffset<'a, Data, Element>
where
    Data: Index<usize, Output = Element>,
{
    pub fn new(data: &'a Data, offset: usize) -> Self {
        IndexOffset { data, offset }
    }
}

impl<'a, Data, Element> Index<usize> for IndexOffset<'a, Data, Element>
where
    Data: Index<usize, Output = Element>,
{
    type Output = Element;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[self.offset + index]
    }
}

pub struct IndexOffsetMut<'a, Data, Element>
where
    Data: IndexMut<usize, Output = Element>,
{
    pub data: &'a mut Data,
    pub offset: usize,
}

impl<'a, Data, Element> IndexOffsetMut<'a, Data, Element>
where
    Data: IndexMut<usize, Output = Element>,
{
    pub fn new(data: &'a mut Data, offset: usize) -> Self {
        IndexOffsetMut { data, offset }
    }
}

impl<'a, Data, Element> IndexMut<usize> for IndexOffsetMut<'a, Data, Element>
where
    Data: IndexMut<usize, Output = Element>,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[self.offset + index]
    }
}

impl<'a, Data, Element> Index<usize> for IndexOffsetMut<'a, Data, Element>
where
    Data: IndexMut<usize, Output = Element>,
{
    type Output = Element;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[self.offset + index]
    }
}
