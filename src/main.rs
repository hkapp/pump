pub mod error;
pub mod parse;
pub mod runtime;

use std::env;

use error::Error;

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

fn retrieve_program() -> Result<String, Error> {
    let mut args: Vec<String> = env::args().skip(1).collect();

    match args.len() {
        0 => Err(Error::EmptyProgram),
        1 => Ok(args.pop().unwrap()),
        _ => Err(Error::TooManyCliArgs),
    }
}

fn submain(pgm: &str) -> Result<(), Error> {
    eprintln!("Program: {}", pgm);
    let expr_tree = parse::parse(&pgm)?;
    eprintln!("Parsed program: {}", expr_tree.pretty_print());
    runtime::exec_and_print(expr_tree)
}