use std::cmp::{min, max};

use crate::dsu::Unite;

#[derive(Debug, Clone, Copy)]
pub enum Link {
    Colored(usize, usize),
    Uncolored(usize, usize),
    Complete(usize),
}

impl Unite for Link {
    fn unite(self, oth: Self, mut u: usize, mut v: usize) -> Option<Self> {
        match self {
            Link::Uncolored(u1, u2) => {
                u = if u == u1 { u2 } else { u1 };
                match oth {
                    Link::Uncolored(v1, v2) => {
                        v = if v == v1 { v2 } else { v1 };
                        Some(Link::Uncolored(min(u, v), max(u, v)))
                    },
                    Link::Colored(color, _) => Some(Link::Colored(color, u)),
                    _ => None
                }
            },
            Link::Colored(color, _) => {
                match oth {
                    Link::Uncolored(v1, v2) => {
                        v = if v == v1 { v2 } else { v1 };
                        Some(Link::Colored(color, v))
                    },
                    Link::Colored(oth, _) if color == oth => Some(Link::Complete(color)),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}