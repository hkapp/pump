mod scalar;
mod stream;

use std::{cell::Cell, fmt::{Debug, Display}, rc::Rc};

use crate::error::Error;
use crate::compile::Expr;

pub fn exec_and_print(expr_tree: Expr) -> Result<(), Error> {
    let mut exec_tree = stream::stream_from(expr_tree);

    while let Some(rt_val) = exec_tree.next() {
        let line_to_print = rt_val?;
        println!("{}", line_to_print.format());
    }
    Ok(())
}

/* RtVal */

#[derive(Clone)]
enum RtVal {
    String(String),
    Number(Number),
    Bool(bool)
}

type Number = f64;

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

    fn format(&self) -> String {
        match self {
            Self::String(s) => s.clone(),
            Self::Number(n) => n.to_string(),
            Self::Bool(b) => b.to_string(),
        }
    }
}

impl Display for RtVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => Display::fmt(s, f),
            Self::Number(n) => Display::fmt(n, f),
            Self::Bool(b) => Display::fmt(b, f),
        }
    }
}

impl From<String> for RtVal {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<Number> for RtVal {
    fn from(value: Number) -> Self {
        Self::Number(value)
    }
}

impl From<bool> for RtVal {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

/* StreamVar */

/// A variable that acts as a channel, read and written for each
/// value in a stream.
pub struct StreamVar(Rc<Cell<Option<RtVal>>>);

impl StreamVar {
    // We don't have the concept of Reader and Writer yet,
    // but the typical use case will anyway be to have one of each
    fn new_pair() -> (Self, Self) {
        let cell = Cell::default();
        let rc = Rc::new(cell);

        let reader_rc = Rc::clone(&rc);
        let writer_rc = rc;

        let reader = StreamVar(reader_rc);
        let writer = StreamVar(writer_rc);

        (reader, writer)
    }

    fn read(&mut self) -> Option<RtVal> {
        self.0.take()
    }

    fn write(&mut self, new_value: RtVal) {
        let old_value = self.0.replace(Some(new_value));

        // For now, we require that every written value is read exactly once
        assert!(old_value.is_none());
    }
}

impl Debug for StreamVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StreamVar({:?})", self.0.as_ptr())
    }
}