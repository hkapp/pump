mod parse;
//mod types;

use crate::Error;

pub use parse::{ParsePos, Identifier, Expr, RegexSubst};

pub fn compile(pgm: &str) -> Result<Expr, Error> {
    eprintln!("Program: {}", pgm);
    let expr_tree = parse::parse(&pgm)?;
    eprintln!("Parsed program: {}", expr_tree.pretty_print());
    Ok(expr_tree)
}