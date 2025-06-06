use std::{fmt::Display, io};

use crate::compile::{ParsePos, Identifier};

pub enum Error {
    EmptyProgram,
    TooManyCliArgs,
    TooManyExprs(ParsePos),  // TODO remove
    CantResolve(Identifier),
    NotEnoughArguments(ParsePos),
    TooManyArguments(ParsePos),
    UnrecognizedToken(ParsePos),
    NotAFunction(ParsePos),
    WrongArgType { expected: String, found: String, err_pos: ParsePos },
    NonFormattable(String),
    NotANumber { str_value: String, parse_err: std::num::ParseFloatError, err_pos: ParsePos },
}

impl Error {
    pub fn format<W: io::Write>(&self, source: &str, buf: &mut W) -> io::Result<()> {
        match self.position() {
            Some(p) => {
                writeln!(buf, "{}", source)?;
                write_error_line(p, buf)?;
            },
            None => { },
        }

        write!(buf, "pump: {}", self)
    }

    fn position(&self) -> Option<ParsePos> {
        match &self {
            Error::EmptyProgram => None,
            Error::TooManyCliArgs => None,
            Error::TooManyExprs(err_pos) => Some(*err_pos),
            Error::CantResolve(idn) => Some(idn.position),
            Error::NotEnoughArguments(err_pos) => Some(*err_pos),
            Error::TooManyArguments(err_pos) => Some(*err_pos),
            Error::UnrecognizedToken(err_pos) => Some(*err_pos),
            Error::NotAFunction(err_pos) => Some(*err_pos),
            Error::WrongArgType { err_pos, .. } => Some(*err_pos),
            Error::NonFormattable(_) => None,
            Error::NotANumber { err_pos, .. } => Some(*err_pos),
        }
    }
}

fn write_error_line<W: io::Write>(err_pos: ParsePos, buf: &mut W) -> io::Result<()> {
    // TODO support multi-line programs
    writeln!(buf, "{}{}", str::repeat(" ", err_pos.start), str::repeat("^", err_pos.len))
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::EmptyProgram =>
                write!(f, "Program is empty. Provide at least one expression."),
            Error::TooManyCliArgs =>
                write!(f, "Too many command line arguments"),
            Error::TooManyExprs(_) =>
                write!(f, "Too many expressions (we only support 1 right now)"),
            Error::CantResolve(idn) =>
                write!(f, "Can't resolve identifier {:?}", idn.name),
            Error::NotEnoughArguments(_) =>
                // TODO add expectation
                write!(f, "Not enough arguments in function call"),
            Error::TooManyArguments(_) =>
                // TODO add expectation
                write!(f, "Too many arguments in function call"),
            Error::UnrecognizedToken(_) =>
                write!(f, "Unrecognized token"),
            Error::NotAFunction(_) =>
                write!(f, "Not a function"),
            Error::WrongArgType { expected, found, .. } =>
                write!(f, "Wrong argument type in function call: expected {}, found {}", expected, found),
            Error::NonFormattable(type_str) =>
                write!(f, "Top-level program type cannot be formatted: {}", type_str),
            Error::NotANumber { str_value, parse_err, .. } =>
                write!(f, "runtime value {:?} cannot be parsed as a number ({})", str_value, parse_err),
        }
    }
}