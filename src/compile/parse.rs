mod token;

pub use token::{ParsePos, Identifier, Token, RegexSubst};

use std::{fmt::Display, iter::Peekable, ops::DerefMut};

use crate::{error::Error, runtime};

use super::Position;

pub fn parse(pgm: &str) -> Result<Expr, Error> {
    let tokens = token::tokenize(pgm);
    let mut parsed = build_exp_tree(tokens)?;
    name_resolution(&mut parsed)?;
    Ok(parsed)
}

/* Expr */

#[derive(Debug)]
pub enum Expr {
    /* Streams */
    Stdin,
    Filter { filter_fn: Box<Expr>, data_source: Box<Expr> },
    Map { map_fn: Box<Expr>, data_source: Box<Expr> },
    /* Scalars */
    RegexMatch(regex::Regex, ParsePos),
    RegexSubst(token::RegexSubst),
    UnresolvedIdentifier(Identifier),
    FunCall(FunCall),
    ReadVar(runtime::StreamVar),
}

impl Expr {
    fn children_mut(&mut self) -> Vec<&mut Self> {
        // TODO find a better way to avoid allocations
        match self {
            Self::Filter { filter_fn, data_source } =>
                vec![filter_fn, data_source],
            Self::Map { map_fn, data_source } =>
                vec![map_fn, data_source],
            Self::RegexMatch(..) => Vec::new(),
            Self::RegexSubst(..) => Vec::new(),
            Self::Stdin => Vec::new(),
            Self::UnresolvedIdentifier(_) => Vec::new(),
            Self::FunCall(fcall) => {
                let mut children = vec![fcall.function.deref_mut()];
                children.extend(fcall.arguments.iter_mut());
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

    // This is a weird trick to get println statements to look decent
    pub fn pretty_print(&self) -> &Self {
        self
    }
}

impl Position for Expr {
    fn position(&self) -> ParsePos {
        match self {
            Expr::FunCall(fcall) =>
                // TODO introduce a parse pos merging
                fcall.function.position(),
            Expr::UnresolvedIdentifier(idn) =>
                idn.position,
            Expr::RegexMatch(_, pos) => *pos,
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
        Kind::RegexSubst(subst) => Expr::RegexSubst(subst),
    }
}

/* Parsing an expression tree */

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
        return Err(Error::EmptyProgram);
    }
    else {
        let final_tree = build_next_tree(&mut tokens)?;

        match tokens.next() {
            Some(Ok(trailing)) =>
                // We have more tokens, but we should have reached the end of the stream
                // FIXME turn TooManyExprs into OrphanTokens
                return Err(Error::TooManyExprs(trailing.position)),
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

    let fun_call = FunCall::new_expr(first_token, args);
    Ok(fun_call)
}

/* FunCall */

#[derive(Debug)]
pub struct FunCall {
    pub function:  Box<Expr>,
    pub arguments: Vec<Expr>,
}

impl FunCall {
    fn new_expr(function: Expr, arguments: Vec<Expr>) -> Expr {
        Self::new_expr_boxed(Box::new(function), arguments)
    }

    pub fn new_expr_boxed(function: Box<Expr>, arguments: Vec<Expr>) -> Expr {
        let me = Self { function, arguments };
        Expr::FunCall(me)
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
        // Here we cheat a bit
        Expr::FunCall(fcall) => {
            *expr_tree = resolve_fun_call(fcall)?;
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
        _       => Err(Error::CantResolve(starting_idn)),
    }
}

fn resolve_fun_call(fcall: &mut FunCall) -> Result<Expr, Error> {
    match fcall.function.as_mut() {
        Expr::UnresolvedIdentifier(idn) => {
            match idn.name.as_str() {
                "filter" => filter_from_fun_call(std::mem::take(&mut fcall.arguments), idn.position),
                "map"    => map_from_fun_call(std::mem::take(&mut fcall.arguments), idn.position),
                _        => Err(Error::NotAFunction(idn.position)),
            }
        }
        _ => Err(Error::NotAFunction(fcall.function.position())),
    }
}

fn filter_from_fun_call(mut args: Vec<Expr>, fn_pos: ParsePos) -> Result<Expr, Error> {
    match args.len() {
        0 => {
            // Not enough arguments
            Err(Error::NotEnoughArguments(fn_pos.right_after()))
        },
        1 => {
            // Not enough arguments
            Err(Error::NotEnoughArguments(args[0].position().right_after()))
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
            Err(Error::TooManyArguments(args[2].position()))
        }
    }
}

fn map_from_fun_call(mut args: Vec<Expr>, fn_pos: ParsePos) -> Result<Expr, Error> {
    match args.len() {
        0 => {
            // Not enough arguments
            Err(Error::NotEnoughArguments(fn_pos.right_after()))
        },
        1 => {
            // Not enough arguments
            Err(Error::NotEnoughArguments(args[0].position().right_after()))
        },
        2 => {
            // Right number of arguments: build the Expr
            // Note: we get the arguments in reverse order because of pop()
            let data_source = args.pop().unwrap();
            let map_fn = args.pop().unwrap();

            Ok(Expr::Map { map_fn: Box::new(map_fn), data_source: Box::new(data_source) })
        },
        _ => {
            // Too many arguments
            Err(Error::TooManyArguments(args[2].position()))
        }
    }
}

/* Pretty printing */

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Stdin => write!(f, "stdin"),
            Expr::Filter { filter_fn, data_source } => {
                write!(f, "filter {} {}", filter_fn, data_source)
            },
            Expr::Map { map_fn, data_source } => {
                write!(f, "map {} {}", map_fn, data_source)
            },
            Expr::RegexMatch(re, _) => {
                write!(f, "m/{}/", re.as_str())
            },
            Expr::RegexSubst(subst) => {
                write!(f, "s/{}/{}/", subst.search.as_str(), subst.replace)
            },
            Expr::UnresolvedIdentifier(identifier) => {
                write!(f, "?:{}:?", identifier.name)
            },
            Expr::FunCall(fcall) => {
                write!(f, "{}", fcall.function)?;
                for arg in &fcall.arguments {
                    write!(f, " {}", arg)?;
                }
                Ok(())
            },
            Expr::ReadVar(stream_var) => {
                write!(f, "(read {:?})", stream_var)
            },
        }
    }
}