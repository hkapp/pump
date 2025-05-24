use std::{fmt::Display, io};

use crate::parse::{ParsePos, Identifier};

/// The same as Error
// TODO remove
pub type ErrCode = Error;

pub enum Error {
    EmptyProgram,
    TooManyCliArgs,
    TooManyExprs(ParsePos),  // TODO remove
    CantResolve(Identifier),
    NotEnoughArguments(ParsePos),
    TooManyArguments(ParsePos),
    UnrecognizedToken(ParsePos),
    NotAFunction(ParsePos),
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
            ErrCode::EmptyProgram => None,
            ErrCode::TooManyCliArgs => None,
            ErrCode::TooManyExprs(err_pos) => Some(*err_pos),
            ErrCode::CantResolve(idn) => Some(idn.position),
            ErrCode::NotEnoughArguments(err_pos) => Some(*err_pos),
            ErrCode::TooManyArguments(err_pos) => Some(*err_pos),
            ErrCode::UnrecognizedToken(err_pos) => Some(*err_pos),
            ErrCode::NotAFunction(err_pos) => Some(*err_pos),
        }
    }
}

fn write_error_line<W: io::Write>(err_pos: ParsePos, buf: &mut W) -> io::Result<()> {
    // TODO support multi-line programs
    writeln!(buf, "{}{}", str::repeat(" ", err_pos.start), str::repeat("^", err_pos.len))
}

impl Display for ErrCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrCode::EmptyProgram =>
                write!(f, "Program is empty. Provide at least one expression."),
            ErrCode::TooManyCliArgs =>
                write!(f, "Too many command line arguments"),
            ErrCode::TooManyExprs(_) =>
                write!(f, "Too many expressions (we only support 1 right now)"),
            ErrCode::CantResolve(idn) =>
                write!(f, "Can't resolve identifier {:?}", idn.name),
            ErrCode::NotEnoughArguments(_) =>
                write!(f, "Not enough arguments in function call"),
            ErrCode::TooManyArguments(_) =>
                write!(f, "Too many arguments in function call"),
            ErrCode::UnrecognizedToken(_) =>
                write!(f, "Unrecognized token"),
            ErrCode::NotAFunction(_) =>
                write!(f, "Not a function"),
        }
    }
}