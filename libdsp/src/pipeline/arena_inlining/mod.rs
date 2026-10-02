pub mod nodes;

pub trait DataExtractor<Data, Output> = Fn(&Data) -> Output;

pub trait PipelineNode<const CHANNELS: usize, Data> {
    fn update(&mut self, data: &Data);

    fn inputs_needed(&mut self, wanted: usize) -> usize;

    fn next(&mut self, input: &[[f32; CHANNELS]], clock: usize) -> [f32; CHANNELS];

    fn reset(&mut self);
}

struct PipelineNodeStorage<'a, const CHANNELS: usize, Data> {
    node: Box<dyn PipelineNode<CHANNELS, Data> + 'a>,

    inputs: Vec<usize>,
    outputs: Vec<usize>,

    last_clock: usize,

    needed_outputs: usize,
    needed_inputs: Vec<usize>,
}

pub struct PipelineEngine<'a, const CHANNELS: usize, Data> {
    nodes: Vec<PipelineNodeStorage<'a, CHANNELS, Data>>,

    output_cache: Vec<Vec<[f32; CHANNELS]>>,

    scratch_buffer: Vec<[f32; CHANNELS]>,

    last_clock: usize,
}

#[derive(Clone, Copy)]
pub struct PipelineConnector {
    node_id: Option<usize>,
}

#[derive(Clone)]
pub struct PipelineResetSpecifier {
    nodes: Vec<usize>,
}

impl PipelineResetSpecifier {
    pub fn new(connector: &PipelineConnector) -> Self {
        let mut new_self = Self { nodes: Vec::new() };

        new_self.extend(connector);

        new_self
    }

    pub fn extend(&mut self, connector: &PipelineConnector) {
        if let Some(node) = connector.node_id {
            self.nodes.push(node);
        }
    }
}

impl<Data, const CHANNELS: usize> Default for PipelineEngine<'_, CHANNELS, Data> {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),

            output_cache: Vec::new(),

            scratch_buffer: Vec::new(),

            last_clock: 0,
        }
    }
}

impl<'a, const CHANNELS: usize, Data> PipelineEngine<'a, CHANNELS, Data> {
    pub fn begin() -> PipelineConnector {
        PipelineConnector { node_id: None }
    }

    pub fn add(
        &mut self,
        inputs: &[PipelineConnector],
        node: impl PipelineNode<CHANNELS, Data> + 'a,
    ) -> PipelineConnector {
        let node_number = self.nodes.len();

        let inputs = inputs.iter().filter_map(|input| input.node_id).collect();

        for input in &inputs {
            let Some(node_store): Option<&mut PipelineNodeStorage<CHANNELS, Data>> =
                self.nodes.get_mut(*(input as &usize))
            else {
                continue;
            };

            node_store.outputs.push(node_number);
        }

        let node_store = PipelineNodeStorage::<CHANNELS, Data> {
            node: Box::new(node),

            inputs,
            outputs: Vec::new(),

            last_clock: 0,

            needed_outputs: 0,
            needed_inputs: Vec::new(),
        };

        self.nodes.push(node_store);
        self.output_cache.push(vec![[0.0; CHANNELS]]);

        PipelineConnector {
            node_id: Some(node_number),
        }
    }

    pub fn update(&mut self, data: &Data) {
        for node_store in &mut self.nodes {
            node_store.node.update(data);
        }
    }

    pub fn next(&mut self, end_node: &PipelineConnector) -> [f32; CHANNELS] {
        let Some(final_node_index) = end_node.node_id else {
            return [0.0; CHANNELS];
        };

        for node_store in &mut self.nodes {
            node_store.needed_outputs = 0;
        }

        for index in (0..self.nodes.len()).rev() {
            if index == final_node_index {
                if let Some(node_store) = self.nodes.get_mut(index) {
                    node_store.needed_outputs = 1;
                };
            } else {
                let wanted = if let Some(node_store) = self.nodes.get(index) {
                    node_store
                        .outputs
                        .iter()
                        .filter_map(|node_index| self.nodes.get(*node_index))
                        .map(|node| node.needed_outputs)
                        .max()
                        .unwrap_or(0)
                } else {
                    continue;
                };

                if wanted == 0 {
                    continue;
                }

                if let Some(node_store) = self.nodes.get_mut(index) {
                    node_store.needed_outputs = wanted;
                    node_store.needed_inputs = node_store.node.inputs_needed(wanted);
                } else {
                    continue;
                };

                if let Some(output_cache) = self.output_cache.get_mut(index) {
                    if output_cache.len() < wanted {
                        output_cache.resize(wanted, [0.0; CHANNELS]);
                    }
                }

                if self.scratch_buffer.len() < wanted {
                    self.scratch_buffer.resize(wanted, [0.0; CHANNELS]);
                }
            }
        }

        for (node_index, node_store) in self.nodes.iter_mut().enumerate() {
            if let Some(output_cache) = self.output_cache.get_mut(node_index) {
                for (output_index, output) in output_cache[0..node_store.needed_outputs]
                    .iter_mut()
                    .enumerate()
                {
                    let inputs_needed = node_store.needed_inputs;

                    // for index in &node_store.inputs {
                    //     let Some(inputs) = self.output_cache.get(*index) else {
                    //         continue;
                    //     };

                    //     let Some(input) = inputs.get(output_index) else {
                    //         continue;
                    //     };

                    //     for (target, source) in complete_input.iter_mut().zip(input) {
                    //         *target += source;
                    //     }
                    // }

                    *output = node_store
                        .node
                        .next(&self.scratch_buffer, node_store.last_clock);

                    node_store.last_clock = node_store.last_clock.wrapping_add(1);
                }
            }
        }

        let Some(final_node_output) = self.output_cache.get(final_node_index) else {
            return [0.0; CHANNELS];
        };

        let Some(final_node_output) = final_node_output.get(0) else {
            return [0.0; CHANNELS];
        };

        *final_node_output
    }

    pub fn reset(&mut self) {
        for node_store in &mut self.nodes {
            node_store.node.reset();
        }
    }

    pub fn reset_segment(&mut self, segment: &PipelineResetSpecifier) {
        for node_index in &segment.nodes {
            let Some(node_store) = self.nodes.get_mut(*node_index) else {
                continue;
            };

            node_store.node.reset();
        }
    }
}
