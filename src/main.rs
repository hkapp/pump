use std::{env, fmt::Display, io};

use regex::Regex;
use lazy_static::lazy_static; // FIXME should be using LazyCell here, but couldn't get it working

fn main() {
    let pgm = retrieve_program();
    match pgm {
        Ok(source) => {
            match submain(&source) {
                Ok(_) => (),
                Err(e) => {
                    e.format(&source, &mut std::io::stderr()).unwrap();
                    eprintln!();
                    std::process::exit(1); // TODO use the error to get a return code
                }
            }
        }
        // This arm is different because we don't have an input source to use for proper error formatting
        Err(e) => {
            eprintln!("pump: {}", e);
            std::process::exit(1); // TODO use the error to get a return code
        }
    }
}

fn retrieve_program() -> Result<String, ErrCode> {
    let mut args: Vec<String> = env::args().skip(1).collect();

    match args.len() {
        0 => Err(ErrCode::EmptyProgram),
        1 => Ok(args.pop().unwrap()),
        _ => Err(ErrCode::TooManyArguments),
    }
}

fn submain(pgm: &str) -> Result<(), Error> {
    eprintln!("Program: {}", pgm);
    let expr_tree = parse(&pgm)?;
    eprintln!("Parsed program: {:?}", expr_tree);
    expr_tree.exec();
    Ok(())
}

fn parse(pgm: &str) -> Result<Expr, Error> {
    build_exp_tree(tokenize(pgm))
}

struct Identifier {
    name:     String,
    position: ParsePos
}

impl<'h> From<regex::Match<'h>> for Identifier {
    fn from(m: regex::Match) -> Self {
        Identifier {
            name:     m.as_str().into(),
            position: ParsePos(m.start()),
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
                    error(ErrCode::CantResolve(single_idn), idn_pos)
                },
            }
        }
        0 => error_no_pos(ErrCode::EmptyProgram),
        _ => error(ErrCode::TooManyExprs, idns[1].position),
    }
}

#[derive(Debug)]
enum Expr {
    Stdin
}

impl Expr {
    fn exec(&self) {
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

struct Error {
    position:   ParsePos,
    error_code: ErrCode
}

impl Error {
    fn new(err_code: ErrCode, err_pos: ParsePos) -> Self {
        Error {
            position:   err_pos,
            error_code: err_code
        }
    }

    fn format<W: io::Write>(&self, source: &str, buf: &mut W) -> io::Result<()> {
        self.position.format(source, buf)?;
        writeln!(buf)?;
        write!(buf, "pump: {}", self.error_code)
    }
}

/// Will always return Err
fn error<T>(err_code: ErrCode, err_pos: ParsePos) -> Result<T, Error> {
    Err(Error::new(err_code, err_pos))
}

/// Will always return Err
// FIXME introduce the actual concept of "no position"
fn error_no_pos<T>(err_code: ErrCode) -> Result<T, Error> {
    Err(Error::new(err_code, ParsePos(0)))
}

enum ErrCode {
    EmptyProgram,
    TooManyArguments,
    TooManyExprs,  // TODO remove
    CantResolve(Identifier),
}

impl Display for ErrCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrCode::EmptyProgram =>
                write!(f, "Program is empty. Provide at least one expression."),
            ErrCode::TooManyArguments =>
                write!(f, "Too many command line arguments"),
            ErrCode::TooManyExprs =>
                write!(f, "Too many expressions (we only support 1 right now)"),
            ErrCode::CantResolve(idn) =>
                write!(f, "Can't resolve identifier {:?}", idn.name),
        }
    }
}

#[derive(Clone, Copy)]
struct ParsePos(usize);

impl ParsePos {
    fn format<W: io::Write>(&self, source: &str, buf: &mut W) -> io::Result<()> {
        // TODO support multi-line programs
        writeln!(buf, "{}", source)?;
        write!(buf, "{}^", str::repeat(" ", self.0))
    }
}