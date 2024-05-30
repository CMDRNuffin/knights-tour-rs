use std::fmt::Display;

use crate::{args::InputArgs, move_graph::Direction};

pub enum Mode {
    Basic(InputArgs),
    Structured(StructureMode),
    Freeform,
}

impl Display for Mode{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Basic(_) => write!(f, "basic"),
            Self::Structured(StructureMode::Closed(_)) => write!(f, "closed"),
            Self::Structured(StructureMode::Stretched(d)) => write!(f, "stretched {}", if d.is_vertical() { "vertical" } else { "horizontal" }),
            Self::Freeform => write!(f, "Freeform mode"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum StructureMode {
    Closed(bool),
    Stretched(Direction),
}