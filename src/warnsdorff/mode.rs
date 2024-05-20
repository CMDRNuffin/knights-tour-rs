use crate::{args::InputArgs, move_graph::Direction};

pub enum Mode {
    Basic(InputArgs),
    Structured(StructureMode),
    Freeform,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum StructureMode {
    Closed(bool),
    Stretched(Direction),
}