mod token;

pub use token::{ParsePos, Identifier};

use std::io;

use crate::error::{self, Error, ErrCode};

pub fn parse(pgm: &str) -> Result<Expr, Error> {
    build_exp_tree(token::tokenize(pgm))
}

fn build_exp_tree<I: Iterator<Item=Identifier>>(tokens: I) -> Result<Expr, Error> {
    let mut idns: Vec<_> = tokens.collect();

    match idns.len() {
        1 => {
            let single_idn = idns.pop().unwrap();
            match single_idn.name.as_str() {
                "stdin" => Ok(Expr::Stdin),
                _       => {
                    let idn_pos = single_idn.position;
                    error::error(ErrCode::CantResolve(single_idn), idn_pos)
                },
            }
        }
        0 => error::error_no_pos(ErrCode::EmptyProgram),
        _ => error::error(ErrCode::TooManyExprs, idns[1].position),
    }
}

#[derive(Debug)]
pub enum Expr {
    Stdin
}

impl Expr {
    pub fn exec(&self) {
        match self {
            Expr::Stdin => {
                // Here we simply read everything from stdin and pipe it out
                io::stdin()
                    .lines()
                    .for_each(|l| println!("{}", l.unwrap()))
            }
        }
    }
}