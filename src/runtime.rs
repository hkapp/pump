use std::{fmt::Display, io::{self, StdinLock}};

use regex::Regex;

use crate::{error::Error, parse::Expr};

enum RtNode {
    Stdin(StdinState),
    RegexMatch(RegexMatch),
    Filter(StreamFilter),
}

impl Exec for RtNode {
    fn next_value(&mut self) -> Option<Result<RtVal, Error>> {
        match self {
            Self::Stdin(s) => s.next_value(),
            Self::RegexMatch(r) => r.next_value(),
            Self::Filter(f) => f.next_value(),
        }
    }

    fn eval(&mut self, input: RtVal) -> Result<RtVal, Error> {
        match self {
            Self::Stdin(s) => s.eval(input),
            Self::RegexMatch(r) => r.eval(input),
            Self::Filter(f) => f.eval(input),
        }
    }
}

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

trait Exec {
    // TODO make this an iterator if meaningful
    fn next_value(&mut self) -> Option<Result<RtVal, Error>>;
    fn eval(&mut self, input: RtVal) -> Result<RtVal, Error>;
}

pub fn exec_and_print(expr_tree: Expr) -> Result<(), Error> {
    let mut exec_tree = executable_form(expr_tree);

    while let Some(rt_val) = exec_tree.next_value() {
        let line_to_print = rt_val?;
        println!("{}", line_to_print);
    }
    Ok(())
}

// TODO make this take a box
fn executable_form(expr: Expr) -> RtNode {
    match expr {
        Expr::Stdin => RtNode::Stdin(StdinState { stdin_lines: io::stdin().lines() }),
        Expr::RegexMatch(regex, pos) => RegexMatch::new_node(regex),
        Expr::Filter { filter_fn, data_source } => StreamFilter::new_node(filter_fn, data_source),
    }
}

struct StdinState {
    stdin_lines: io::Lines<StdinLock<'static>>,
}

impl Exec for StdinState {
    fn next_value(&mut self) -> Option<Result<RtVal, Error>> {
        self.stdin_lines
            .next()
            // FIXME introduce a proper error here
            .map(|l| Ok(l.unwrap().into()))
    }

    fn eval(&mut self, _input: RtVal) -> Result<RtVal, Error> {
        panic!("Unsupported operation");
    }
}

struct RegexMatch {
    regex: Regex,
}

impl RegexMatch {
    fn new_node(regex: Regex) -> RtNode {
        let me = RegexMatch { regex };
        RtNode::RegexMatch(me)
    }
}

impl Exec for RegexMatch {
    fn next_value(&mut self) -> Option<Result<RtVal, Error>> {
        panic!("Unsupported operation")
    }

    fn eval(&mut self, input: RtVal) -> Result<RtVal, Error> {
        // FIXME
        let is_match = self.regex.is_match(&input.str_ref().unwrap());
        let rt_val = is_match.into();
        Ok(rt_val)
    }
}

struct StreamFilter {
    filter_fn: Box<RtNode>,
    stream:    Box<RtNode>,
}

impl StreamFilter {
    fn new_node(filter_fn: Box<Expr>, data_source: Box<Expr>) -> RtNode {
        // Note: Box::into_inner is only available on nightly
        fn unbox<T>(boxed: Box<T>) -> T {
            *boxed
        }

        fn box_map<T, U, F: FnOnce(T) -> U>(a: Box<T>, f: F) -> Box<U> {
            let b = f(unbox(a));
            Box::new(b)
        }

        let filter = StreamFilter {
            filter_fn: box_map(filter_fn, executable_form),
            stream:    box_map(data_source, executable_form),
        };

        RtNode::Filter(filter)
    }
}

impl Exec for StreamFilter {
    fn next_value(&mut self) -> Option<Result<RtVal, Error>> {
        loop {
            match self.stream.next_value() {
                None => return None, // End of the stream
                same@Some(Err(_)) => return same, // Stream encountered an error
                Some(Ok(rt_val)) => {
                    // FIXME
                    let keep = rt_val.clone();
                    let predicate_eval = match self.filter_fn.eval(rt_val) {
                        Ok(v) => v,
                        Err(e) => return Some(Err(e)),
                    };
                    if predicate_eval.as_bool().unwrap() {
                        return Some(Ok(keep));
                    }
                    else {
                        continue;
                    }
                },
            }
        }
    }

    fn eval(&mut self, _input: RtVal) -> Result<RtVal, Error> {
        panic!("Unsupported operation")
    }
}