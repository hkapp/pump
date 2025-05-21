mod token;

pub use token::{ParsePos, Identifier, Token};

use std::{iter::Peekable, ops::DerefMut};

use crate::{error::{self, ErrCode, Error}, runtime};

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
    FunCall { function: Box<Expr>, arguments: Vec<Expr> },
    ReadVar(runtime::StreamVar),
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
            Self::FunCall { function, arguments } => {
                let mut children = vec![function.deref_mut()];
                children.extend(arguments.iter_mut());
                children
            },
            Self::ReadVar(_) => Vec::new(),
        }
    }

    /*fn get_child_mut(&mut self, index: usize) -> Option<&mut Self> {
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
    }*/

    // TODO turn into a trait
    fn position(&self) -> ParsePos {
        match self {
            Expr::FunCall { function, .. } =>
                // TODO introduce a parse pos merging
                function.position(),
            Expr::UnresolvedIdentifier(idn) =>
                idn.position,
            _ =>
                // FIXME this is terrible
                todo!(),
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

    if tokens.peek().is_none() {
        return error::error_no_pos(ErrCode::EmptyProgram);
    }
    else {
        let final_tree = build_next_tree(&mut tokens)?;

        match tokens.next() {
            Some(Ok(trailing)) =>
                // We have more tokens, but we should have reached the end of the stream
                // FIXME turn TooManyExprs into OrphanTokens
                return error::error(ErrCode::TooManyExprs, trailing.position),
            Some(Err(e)) =>
                // The tokenizer had an issue, just pass it along
                return Err(e),
            None =>
                // We reached the end of the stream (as expected)
                Ok(final_tree),
        }
    }
}

// Pre-condition: the token stream has been peeked and is known to have at least one more token
fn build_next_tree<I: Iterator<Item=Result<Token, Error>>>(tokens: &mut Peekable<I>) -> Result<Expr, Error> {
    let first_token = tokens.next().unwrap()?;
    let first_atom = trivial_expr(first_token);

    if tokens.peek().is_some() {
        // There are more tokens: we assume a function call
        parse_fun_call(first_atom, tokens)
    }
    else {
        // No more token: that's all we have
        Ok(first_atom)
    }
}

fn parse_fun_call<I: Iterator<Item=Result<Token, Error>>>(first_token: Expr, rem_tokens: &mut Peekable<I>) -> Result<Expr, Error> {
    let mut args = Vec::new();

    for token_res in rem_tokens {
        let token = token_res?;
        let this_arg = trivial_expr(token);
        args.push(this_arg);
    }

    let fun_call = Expr::FunCall { function: Box::new(first_token), arguments: args };
    Ok(fun_call)
}

/* Name resolution */

fn name_resolution(expr_tree: &mut Expr) -> Result<(), Error> {
    match expr_tree {
        Expr::UnresolvedIdentifier(idn) => {
            let dummy = Identifier { name: String::new(), position: idn.position };
            let idn = std::mem::replace(idn, dummy);
            *expr_tree = resolve_identifier(idn)?;
        }
        // Here we cheat a bit
        Expr::FunCall { function, arguments } => {
            *expr_tree = resolve_fun_call(function, arguments)?;
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

fn resolve_fun_call(function: &mut Expr, arguments: &mut Vec<Expr>) -> Result<Expr, Error> {
    match function {
        Expr::UnresolvedIdentifier(idn) => {
            match idn.name.as_str() {
                "filter" => filter_from_fun_call(std::mem::take(arguments), idn.position),
                _ => error::error(ErrCode::CantResolve(idn.take()), idn.position),
            }
        }
        _ => error::error(ErrCode::NotAFunction, function.position()),
    }
}

fn filter_from_fun_call(mut args: Vec<Expr>, fn_pos: ParsePos) -> Result<Expr, Error> {
    match args.len() {
        0 => {
            // Not enough arguments
            error::error(ErrCode::NotEnoughArguments, fn_pos.right_after())
        },
        1 => {
            // Not enough arguments
            error::error(ErrCode::NotEnoughArguments, args[0].position().right_after())
        },
        2 => {
            // Right number of arguments: build the Expr
            // Note: we get the arguments in reverse order because of pop()
            let data_source = args.pop().unwrap();
            let filter_fn = args.pop().unwrap();

            Ok(Expr::Filter { filter_fn: Box::new(filter_fn), data_source: Box::new(data_source) })
        },
        _ => {
            // Too many arguments
            error::error(ErrCode::TooManyArguments, args[2].position())
        }
    }
}