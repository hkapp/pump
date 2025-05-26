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

// TODO move to its own module under compile
#[derive(Debug)]
pub enum Expr {
    Builtin(Builtin, ParsePos),
    UnresolvedIdentifier(Identifier),
    FunCall(FunCall),
    ReadVar(runtime::StreamVar),
}

#[derive(Debug)]
pub enum Builtin {
    /* Streams */
    Stdin,
    Filter,
    Map,
    /* Scalars */
    RegexMatch(regex::Regex),
    RegexSubst(token::RegexSubst),
}

impl Expr {
    fn children_mut(&mut self) -> Vec<&mut Self> {
        // TODO find a better way to avoid allocations
        match self {
            Self::Builtin(..) =>
                // The Builtin expression is just a marker
                // As such, it can never have children
                Vec::new(),
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
            Expr::Builtin(_, pos) =>
                *pos,
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
        Kind::Identifier(idn) =>
            Expr::UnresolvedIdentifier(idn),
        Kind::RegexMatch(rm) =>
            Expr::Builtin(Builtin::RegexMatch(rm), pos),
        Kind::RegexSubst(subst) =>
            Expr::Builtin(Builtin::RegexSubst(subst), pos),
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
    pub fn new_expr(function: Expr, arguments: Vec<Expr>) -> Expr {
        Self::new_expr_boxed(Box::new(function), arguments)
    }

    fn new_expr_boxed(function: Box<Expr>, arguments: Vec<Expr>) -> Expr {
        let me = Self { function, arguments };
        Expr::FunCall(me)
    }
}

/* Name resolution */

fn name_resolution(expr_tree: &mut Expr) -> Result<(), Error> {
    match expr_tree {
        Expr::UnresolvedIdentifier(idn) => {
            let pos = idn.position;
            let builtin = resolve_builtin(idn.take())?;
            *expr_tree = Expr::Builtin(builtin, pos);
        }
        _ => {},
    }

    // Now resolve the children
    for subtree in expr_tree.children_mut() {
        name_resolution(subtree)?;
    }

    Ok(())
}

fn resolve_builtin(starting_idn: Identifier) -> Result<Builtin, Error> {
    // Note: Rust playground's MIR emit shows us that a constant string
    // match is basically turned into cascading if-elses. For performance,
    // we would want to use a constant hash map here.
    match starting_idn.name.as_str() {
        "stdin"  => Ok(Builtin::Stdin),
        "filter" => Ok(Builtin::Filter),
        "map"    => Ok(Builtin::Map),
        _        => Err(Error::CantResolve(starting_idn)),
    }
}

/* Pretty printing */

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Builtin(b, _pos) =>
                write!(f, "{}", b),
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

impl Display for Builtin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Builtin::Stdin =>
                write!(f, "stdin"),
            Builtin::RegexMatch(re) =>
                write!(f, "m/{}/", re.as_str()),
            Builtin::RegexSubst(subst) =>
                write!(f, "s/{}/{}/", subst.search.as_str(), subst.replace),
            Builtin::Filter =>
                write!(f, "filter"),
            Builtin::Map =>
                write!(f, "map"),
        }
    }
}