mod scalar;
mod stream;

use std::{cell::Cell, fmt::Display, rc::Rc};

use crate::{error::Error, parse::Expr};

pub fn exec_and_print(expr_tree: Expr) -> Result<(), Error> {
    let mut exec_tree = stream::stream_from(expr_tree);

    while let Some(rt_val) = exec_tree.next() {
        let line_to_print = rt_val?;
        println!("{}", line_to_print.str_ref().unwrap());
    }
    Ok(())
}

/* RtVal */

#[derive(Clone)]
enum RtVal {
    String(String),
    Bool(bool)
}

impl RtVal {
    fn str_ref(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(&s),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

impl Display for RtVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => s.fmt(f),
            Self::Bool(b) => b.fmt(f),
        }
    }
}

impl From<bool> for RtVal {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<String> for RtVal {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

/* StreamVar */

/// A variable that acts as a channel, read and written for each
/// value in a stream.
struct StreamVar(Rc<Cell<Option<RtVal>>>);

impl StreamVar {
    fn read(&mut self) -> Option<RtVal> {
        self.0.take()
    }
}