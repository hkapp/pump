use std::io::{self, StdinLock};

use crate::{error::Error, parse::Expr};

enum RtNode {
    Stdin(StdinState),
    // RegexMatch(regex::Regex, ParsePos),
    // Filter { filter_fn: Box<Expr>, data_source: Box<Expr> }
}

impl Exec for RtNode {
    fn next_value(&mut self) -> Option<Result<RtVal, Error>> {
        match self {
            Self::Stdin(s) => s.next_value(),
        }
    }
}

struct StdinState {
    stdin_lines: io::Lines<StdinLock<'static>>,
}

type RtVal = String;

trait Exec {
    // TODO make this an iterator if meaningful
    fn next_value(&mut self) -> Option<Result<RtVal, Error>>;
}

impl Exec for StdinState {
    fn next_value(&mut self) -> Option<Result<RtVal, Error>> {
        self.stdin_lines
            .next()
            // FIXME introduce a proper error here
            .map(|l| Ok(l.unwrap()))
    }
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
        _ => panic!("Unsupported operator: {:?}", expr),
    }
}