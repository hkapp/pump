use std::io::{self, StdinLock};

use crate::{error::Error, parse::Expr};

use super::{scalar::{self, ScalarNode, ExecScalar}, RtVal};

/// Any runtime component that behaves like a stream of runtime values
// Note: we can't do the other way around and derive a blanket implementation
// for Iterator from the ExecStream impl because Iterator is not a local trait
#[allow(dead_code)]
trait ExecStream: Iterator<Item=Result<RtVal, Error>> { }

// Any type that has the iterator we want is considered an ExecStream
impl<T: Iterator<Item=Result<RtVal, Error>>> ExecStream for T { }

pub(super) enum StreamNode {
    Stdin(StdinState),
    Filter(StreamFilter),
}

// TODO consider introducing a macro for this
impl Iterator for StreamNode {
    type Item = Result<RtVal, Error>;

    fn next(&mut self) -> Option<Result<RtVal, Error>> {
        match self {
            Self::Stdin(s) => s.next(),
            Self::Filter(f) => f.next(),
        }
    }
}

// TODO make this take a box
// TODO turn into a From impl
pub fn stream_from(expr: Expr) -> StreamNode {
    match expr {
        Expr::Stdin => StdinState::new_node(),
        Expr::Filter { filter_fn, data_source } => StreamFilter::new_node(filter_fn, data_source),
        // It's fine for us to panic here, as typechecking must have guaranteed that there
        // are no surprises when we arrive here
        _ => panic!("Not a stream: {:?}", expr),
    }
}

/* StdinState */

struct StdinState {
    stdin_lines: io::Lines<StdinLock<'static>>,
}

impl StdinState {
    fn new_node() -> StreamNode {
        StreamNode::Stdin(StdinState { stdin_lines: io::stdin().lines() })
    }
}

type RtRes = Result<RtVal, Error>;

impl Iterator for StdinState {
    type Item = Result<RtVal, Error>;

    fn next(&mut self) -> Option<Result<RtVal, Error>> {
        self.stdin_lines
            .next()
            // FIXME introduce a proper error here
            .map(|l| Ok(l.unwrap().into()))
    }
}

/* StreamFilter */

struct StreamFilter {
    filter_fn: Box<ScalarNode>,
    stream:    Box<StreamNode>,
}

impl StreamFilter {
    fn new_node(filter_fn: Box<Expr>, data_source: Box<Expr>) -> StreamNode {
        // Note: Box::into_inner is only available on nightly
        fn unbox<T>(boxed: Box<T>) -> T {
            *boxed
        }

        fn box_map<T, U, F: FnOnce(T) -> U>(a: Box<T>, f: F) -> Box<U> {
            let b = f(unbox(a));
            Box::new(b)
        }

        let filter = StreamFilter {
            filter_fn: box_map(filter_fn, scalar::scalar_from),
            stream:    box_map(data_source, stream_from),
        };

        StreamNode::Filter(filter)
    }
}

impl Iterator for StreamFilter {
    type Item = RtRes;

    fn next(&mut self) -> Option<Result<RtVal, Error>> {
        loop {
            match self.stream.next() {
                // End of the stream
                None => return None,

                // Stream encountered an error
                same@Some(Err(_)) => return same,

                // We actually got a value from the stream
                Some(Ok(rt_val)) => {
                    let keep = rt_val.clone();
                    let predicate_eval = match self.filter_fn.eval(rt_val) {
                        Ok(v) => v,
                        Err(e) => return Some(Err(e)),
                    };
                    if predicate_eval.as_bool().unwrap() {
                        // The value passes the filter
                        return Some(Ok(keep));
                    }
                    else {
                        // Value filtered out. Try the next one.
                        continue;
                    }
                },
            }
        }
    }
}