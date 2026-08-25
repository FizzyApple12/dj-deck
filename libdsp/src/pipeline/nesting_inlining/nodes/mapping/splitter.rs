use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use crate::pipeline::nesting_inlining::PipelineNode;

#[derive(Clone)]
struct SplitterParent<const CHANNELS: usize, Parent> {
    parent: Parent,
    last_result: [f32; CHANNELS],
    last_clock: usize,
}

#[derive(Clone)]
pub struct Splitter<const CHANNELS: usize, const COPIES: usize, Data, Parent>
where
    Parent: PipelineNode<CHANNELS, Data>,
{
    phantom_data: PhantomData<Data>,
    lazy_parent: Rc<RefCell<SplitterParent<CHANNELS, Parent>>>,
}

impl<const CHANNELS: usize, const COPIES: usize, Data, Parent>
    Splitter<CHANNELS, COPIES, Data, Parent>
where
    Parent: PipelineNode<CHANNELS, Data>,
{
    pub fn new(parent: Parent) -> [Self; COPIES] {
        let lazy_parent = Rc::new(RefCell::new(SplitterParent {
            parent,
            last_result: [0.0; CHANNELS],
            last_clock: usize::MAX,
        }));

        core::array::from_fn(|_| Self {
            phantom_data: PhantomData,
            lazy_parent: lazy_parent.clone(),
        })
    }
}

impl<const CHANNELS: usize, const COPIES: usize, Data, Parent> PipelineNode<CHANNELS, Data>
    for Splitter<CHANNELS, COPIES, Data, Parent>
where
    Parent: PipelineNode<CHANNELS, Data>,
{
    fn update(&mut self, data: &Data) {
        self.lazy_parent.borrow_mut().parent.update(data);
    }

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, clock: usize) -> [f32; CHANNELS] {
        if self.lazy_parent.borrow().last_clock != clock {
            let mut lazy_parent = self.lazy_parent.borrow_mut();

            lazy_parent.last_result = lazy_parent.parent.next(clock);
            lazy_parent.last_clock = clock;
        }

        self.lazy_parent.borrow().last_result
    }

    fn reset(&mut self) {
        self.lazy_parent.borrow_mut().parent.reset();
    }
}
