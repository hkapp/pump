use std::{env, io};

fn main() {
    let mut args: Vec<String> = env::args().skip(1).collect();
    assert_eq!(args.len(), 1);
    let pgm = args.pop().unwrap();
    eprintln!("Program: {}", pgm);
    let expr_tree = parse(&pgm);
    eprintln!("Parsed program: {:?}", expr_tree);
    expr_tree.exec();
}

fn parse(pgm: &str) -> Expr {
    build_exp_tree(tokenize(pgm))
}

type Identifier = String;

fn tokenize(s: &str) -> impl Iterator<Item=Identifier> + '_ {
    s.split(" ")
        .map(Into::into)
}

fn build_exp_tree<I: Iterator<Item=Identifier>>(tokens: I) -> Expr {
    let idns: Vec<_> = tokens.collect();
    assert_eq!(idns.len(), 1);
    match idns[0].as_str() {
        "stdin" => Expr::Stdin,
        op@_  => panic!("Unsupported operator: {op}")
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