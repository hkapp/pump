mod token;

pub use token::{ParsePos, Identifier, Token};

use std::{io, iter::Peekable};

use crate::error::{self, Error, ErrCode};

pub fn parse(pgm: &str) -> Result<Expr, Error> {
    build_exp_tree(token::tokenize(pgm))
}

fn build_exp_tree<I: Iterator<Item=Token>>(token_stream: I) -> Result<Expr, Error> {
    let mut tokens =
        token_stream
            .inspect(|t| eprintln!("next token: {:?}", t))
            .peekable();

    match tokens.next() {
        Some(t) => {
            // If we can build an expression, we need to validate that the entire stream was used
            build_starting(t, &mut tokens)
                .and_then(|expr|
                    match tokens.next() {
                        // FIXME turn TooManyExprs into OrphanTokens
                        Some(trailing) => error::error(ErrCode::TooManyExprs, trailing.position),
                        None => Ok(expr)
                    })
        },
        None => error::error_no_pos(ErrCode::EmptyProgram),
    }
    /*match tokens.len() {
        1 => {
            let single_token = tokens.pop().unwrap();
            use token::Kind;
            match single_token.kind {
                Kind::Identifier(single_idn) => {

                },
                _ => panic!("Unsupported token"),
            }
        }
        0 => error::error_no_pos(ErrCode::EmptyProgram),
        _ => error::error(ErrCode::TooManyExprs, tokens[1].position),
    }*/
}

fn build_starting<I: Iterator<Item=Token>>(starting_token: Token, rem_tokens: &mut Peekable<I>) -> Result<Expr, Error> {
    use token::Kind;
    match starting_token.kind {
        Kind::Identifier(starting_idn) => {
            match rem_tokens.peek() {
                None    => resolve_identifier(starting_idn),
                Some(_) => parse_fun_call(starting_idn, rem_tokens),
            }
        }
        Kind::RegexMatch(regex) => Ok(Expr::RegexMatch(regex, starting_token.position)),
    }
}

fn resolve_identifier(starting_idn: Identifier) -> Result<Expr, Error> {
    match starting_idn.name.as_str() {
        "stdin" => Ok(Expr::Stdin),
        _       => {
            let idn_pos = starting_idn.position;
            error::error(ErrCode::CantResolve(starting_idn), idn_pos)
        },
    }
}

fn parse_fun_call<I: Iterator<Item=Token>>(starting_idn: Identifier, rem_tokens: &mut Peekable<I>) -> Result<Expr, Error> {
    // FIXME need to introduce a separate name resolution phase
    match starting_idn.name.as_str() {
        "filter" => {
            let filter_fn_tok =
                rem_tokens.next()
                    .ok_or(error::error::<Token>(ErrCode::NotEnoughArguments, starting_idn.position.right_after() ).unwrap_err())?;

            let filter_fn_pos = filter_fn_tok.position;
            let filter_fn = trivial_expr(filter_fn_tok);

            let data_source =
                rem_tokens.next()
                    .map(trivial_expr)
                    .ok_or(error::error::<Expr>(ErrCode::NotEnoughArguments, filter_fn_pos.right_after()).unwrap_err())?;
            Ok(Expr::Filter { filter_fn: Box::new(filter_fn), data_source: Box::new(data_source) })
        }
        _ => {
            let err_pos = starting_idn.position;
            error::error(ErrCode::CantResolve(starting_idn), err_pos)
        },
    }
}

fn trivial_expr(token: Token) -> Expr {
    // FIXME for now this somehow works
    build_starting(token, &mut std::iter::empty().peekable()).ok().unwrap()
}

#[derive(Debug)]
pub enum Expr {
    Stdin,
    RegexMatch(regex::Regex, ParsePos),
    Filter { filter_fn: Box<Expr>, data_source: Box<Expr> }
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
            _ => panic!("Unsupported operator: {:?}", self),
        }
    }
}