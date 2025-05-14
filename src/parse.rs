mod token;

pub use token::{ParsePos, Identifier, Token};

use std::io;

use crate::error::{self, Error, ErrCode};

pub fn parse(pgm: &str) -> Result<Expr, Error> {
    build_exp_tree(token::tokenize(pgm))
}

fn build_exp_tree<I: Iterator<Item=Token>>(token_stream: I) -> Result<Expr, Error> {
    let mut tokens: Vec<_> = token_stream.collect();
    eprintln!("Tokens: {:?}", tokens);

    match tokens.len() {
        1 => {
            let single_token = tokens.pop().unwrap();
            use token::Kind;
            match single_token.kind {
                Kind::Identifier(single_idn) => {
                    match single_idn.name.as_str() {
                        "stdin" => Ok(Expr::Stdin),
                        _       => {
                            let idn_pos = single_idn.position;
                            error::error(ErrCode::CantResolve(single_idn), idn_pos)
                        },
                    }
                },
                _ => panic!("Unsupported token"),
            }
        }
        0 => error::error_no_pos(ErrCode::EmptyProgram),
        _ => error::error(ErrCode::TooManyExprs, tokens[1].position),
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