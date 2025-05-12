use std::{env, fmt::Display, io};

fn main() {
    match submain() {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error! {}", e);
            std::process::exit(1); // TODO use the error to get a return code
        }
    }
}

fn submain() -> Result<(), Error> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    assert_eq!(args.len(), 1);
    let pgm = args.pop().unwrap();
    eprintln!("Program: {}", pgm);
    let expr_tree = parse(&pgm)?;
    eprintln!("Parsed program: {:?}", expr_tree);
    expr_tree.exec();
    Ok(())
}

fn parse(pgm: &str) -> Result<Expr, Error> {
    build_exp_tree(tokenize(pgm))
}

type Identifier = String;

fn tokenize(s: &str) -> impl Iterator<Item=Identifier> + '_ {
    s.split(" ")
        .map(Into::into)
}

fn build_exp_tree<I: Iterator<Item=Identifier>>(tokens: I) -> Result<Expr, Error> {
    let idns: Vec<_> = tokens.collect();

    match idns.len() {
        1 => {
            match idns[0].as_str() {
                "stdin" => Ok(Expr::Stdin),
                op@_  => panic!("Unsupported operator: {op}")
            }
        }
        0 => Err(Error::EmptyProgram),
        _ => Err(Error::TooManyExprs),
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

enum Error {
    EmptyProgram,
    TooManyExprs,  // TODO remove
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::EmptyProgram =>
                write!(f, "Program is empty. Provide at least one expression."),
            Error::TooManyExprs =>
                write!(f, "Too many expressions (we only support 1 right now)"),
        }
    }
}