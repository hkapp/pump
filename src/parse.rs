use std::io;
use regex::Regex;
use lazy_static::lazy_static; // FIXME should be using LazyCell here, but couldn't get it working

use crate::error::{self, Error, ErrCode};

pub fn parse(pgm: &str) -> Result<Expr, Error> {
    build_exp_tree(tokenize(pgm))
}

pub struct Identifier {
    pub name: String,
    position: ParsePos
}

impl<'h> From<regex::Match<'h>> for Identifier {
    fn from(m: regex::Match) -> Self {
        Identifier {
            name:     m.as_str().into(),
            position: ParsePos {
                start: m.start(),
                len:   m.as_str().len()
            },
        }
    }
}

lazy_static! {
    static ref IDN_REGEX: Regex = Regex::new("[a-zA-Z][0-9a-zA-Z]*").unwrap();
}

fn tokenize(s: &str) -> impl Iterator<Item=Identifier> + '_ {
    IDN_REGEX.find_iter(s)
        .map(Into::into)
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

#[derive(Clone, Copy)]
pub struct ParsePos {
    pub start: usize,
    pub len:   usize
}

impl ParsePos {
    pub fn format<W: io::Write>(&self, source: &str, buf: &mut W) -> io::Result<()> {
        // TODO support multi-line programs
        writeln!(buf, "{}", source)?;
        write!(buf, "{}{}", str::repeat(" ", self.start), str::repeat("^", self.len))
    }
}