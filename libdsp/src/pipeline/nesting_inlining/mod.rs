pub mod nodes;

pub trait DataExtractor<Data, Output> = Fn(&Data) -> Output;

pub trait PipelineNode<const CHANNELS: usize, Data> {
    fn update(&mut self, data: &Data);

    fn next(&mut self, clock: usize) -> [f32; CHANNELS];

    fn reset(&mut self);
}

// i need two types of output,
//
// variable rate output
// fixed rate output
//
// variable rate output can be converted to fixed rate output
// but not the other way
//
// only fixed rate output can be split
