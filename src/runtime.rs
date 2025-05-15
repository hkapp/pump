use std::io::{self, StdinLock};

use regex::Regex;

use crate::{error::Error, parse::Expr};

enum RtNode {
    Stdin(StdinState),
    RegexMatch(RegexMatch),
    // Filter { filter_fn: Box<Expr>, data_source: Box<Expr> }
}

impl Exec for RtNode {
    fn next_value(&mut self) -> Option<Result<RtVal, Error>> {
        match self {
            Self::Stdin(s) => s.next_value(),
            Self::RegexMatch(r) => r.next_value(),
        }
    }

    fn eval(&mut self, input: RtVal) -> Result<RtVal, Error> {
        match self {
            Self::Stdin(s) => s.eval(input),
            Self::RegexMatch(r) => r.eval(input),
        }
    }
}

type RtVal = String;

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

fn executable_form(expr: Expr) -> RtNode {
    match expr {
        Expr::Stdin => RtNode::Stdin(StdinState { stdin_lines: io::stdin().lines() }),
        Expr::RegexMatch(regex, pos) => RegexMatch::new_node(regex),
        _ => todo!(),
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
            .map(|l| Ok(l.unwrap()))
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
        let is_match = self.regex.is_match(&input);
        let rt_val = is_match.to_string();
        Ok(rt_val)
    }
}