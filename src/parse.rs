mod token;

pub use token::{ParsePos, Identifier, Token};

use std::iter::Peekable;

use crate::error::{self, Error, ErrCode};

pub fn parse(pgm: &str) -> Result<Expr, Error> {
    let tokens = token::tokenize(pgm);
    let mut parsed = build_exp_tree(tokens)?;
    name_resolution(&mut parsed)?;
    Ok(parsed)
}

#[derive(Debug)]
pub enum Expr {
    /* Streams */
    Stdin,
    Filter { filter_fn: Box<Expr>, data_source: Box<Expr> },
    /* Scalars */
    RegexMatch(regex::Regex, ParsePos),
    UnresolvedIdentifier(Identifier),
}

impl Expr {
    fn children_mut(&mut self) -> Vec<&mut Self> {
        // TODO find a better way to avoid allocations
        match self {
            Self::Filter { filter_fn, data_source } =>
                vec![filter_fn, data_source],
            Self::RegexMatch(..) => Vec::new(),
            Self::Stdin => Vec::new(),
            Self::UnresolvedIdentifier(_) => Vec::new(),
        }
    }

    fn get_child_mut(&mut self, index: usize) -> Option<&mut Self> {
        match self {
            Self::Filter { filter_fn, data_source } => {
                match index {
                    0 => Some(filter_fn),
                    1 => Some(data_source),
                    _ => None,
                }
            },
            Self::RegexMatch(..) => None,
            Self::Stdin => None,
            Self::UnresolvedIdentifier(_) => None,
        }
    }
}

/* Atoms */

struct Atoms<'a> {
    token_stream: token::Tokenizer<'a>
}

impl<'a> Iterator for Atoms<'a> {
    type Item = Result<Expr, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.token_stream
            .next()
            .map(|r| r.map(trivial_expr))
    }
}

fn trivial_expr(token: Token) -> Expr {
    let pos = token.position;

    use token::Kind;
    match token.kind {
        Kind::Identifier(idn) => Expr::UnresolvedIdentifier(idn),
        Kind::RegexMatch(rm) => Expr::RegexMatch(rm, pos),
    }
}

/* Expression tree */

fn build_exp_tree<I: Iterator<Item=Result<Token, Error>>>(token_stream: I) -> Result<Expr, Error> {
    let mut tokens =
        token_stream
            .inspect(
                |t|
                    match t {
                        Ok(t) => eprintln!("next token: {:?}", t),
                        _ => {},
                    })
            .peekable();

    match tokens.next() {
        Some(t) => {
            let t = t?;
            // If we can build an expression, we need to validate that the entire stream was used
            build_starting(t, &mut tokens)
                .and_then(|expr|
                    match tokens.next() {
                        // FIXME turn TooManyExprs into OrphanTokens
                        Some(trailing) => error::error(ErrCode::TooManyExprs, trailing?.position),
                        None => Ok(expr)
                    })
        },
        None => error::error_no_pos(ErrCode::EmptyProgram),
    }
}

fn build_starting<I: Iterator<Item=Result<Token, Error>>>(starting_token: Token, rem_tokens: &mut Peekable<I>) -> Result<Expr, Error> {
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

fn parse_fun_call<I: Iterator<Item=Result<Token, Error>>>(starting_idn: Identifier, rem_tokens: &mut Peekable<I>) -> Result<Expr, Error> {
    // FIXME need to introduce a separate name resolution phase
    match starting_idn.name.as_str() {
        "filter" => {
            let filter_fn_tok =
                match rem_tokens.next() {
                    None => return error::error(ErrCode::NotEnoughArguments, starting_idn.position.right_after()),
                    Some(Err(e)) => return Err(e),
                    Some(Ok(t)) => t,
                };

            let filter_fn_pos = filter_fn_tok.position;
            let filter_fn = trivial_expr(filter_fn_tok);

            let data_source =
                match rem_tokens.next() {
                    None => return error::error(ErrCode::NotEnoughArguments, filter_fn_pos.right_after()),
                    Some(Err(e)) => return Err(e),
                    Some(Ok(t)) => trivial_expr(t),
                };

            Ok(Expr::Filter { filter_fn: Box::new(filter_fn), data_source: Box::new(data_source) })
        }
        _ => {
            let err_pos = starting_idn.position;
            error::error(ErrCode::CantResolve(starting_idn), err_pos)
        },
    }
}

/* Name resolution */

fn name_resolution(expr_tree: &mut Expr) -> Result<(), Error> {
    match expr_tree {
        Expr::UnresolvedIdentifier(idn) => {
            let dummy = Identifier { name: String::new(), position: idn.position };
            let idn = std::mem::replace(idn, dummy);
            *expr_tree = resolve_identifier(idn)?;
        }
        _ => {},
    }

    // Now resolve the children
    for subtree in expr_tree.children_mut() {
        name_resolution(subtree)?;
    }

    Ok(())
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