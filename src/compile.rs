mod parse;
mod types;

use crate::Error;

pub use parse::{ParsePos, Identifier, Expr, RegexSubst, FunCall, Builtin};

pub fn compile(pgm: &str) -> Result<Expr, Error> {
    eprintln!("Program: {}", pgm);
    let mut expr_tree = parse::parse(&pgm)?;
    eprintln!("Parsed program: {}", expr_tree.pretty_print());
    types::typecheck_program(&mut expr_tree)?;
    Ok(expr_tree)
}

pub trait Position {
    fn position(&self) -> ParsePos;
}