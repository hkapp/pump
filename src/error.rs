use std::{fmt::Display, io};

use crate::parse::{ParsePos, Identifier};

// TODO remove
pub struct Error {
    error_code: ErrCode
}

impl Error {
    fn new(err_code: ErrCode) -> Self {
        Error {
            error_code: err_code
        }
    }

    pub fn format<W: io::Write>(&self, source: &str, buf: &mut W) -> io::Result<()> {
        match self.position() {
            Some(p) => {
                writeln!(buf, "{}", source)?;
                write_error_line(p, buf)?;
            },
            None => { },
        }

        write!(buf, "pump: {}", self.error_code)
    }

    fn position(&self) -> Option<ParsePos> {
        match &self.error_code {
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

/// Will always return Err
// TODO remove
pub fn error<T>(err_code: ErrCode) -> Result<T, Error> {
    Err(Error::new(err_code))
}

/// Will always return Err
// FIXME introduce the actual concept of "no position"
pub fn error_no_pos<T>(err_code: ErrCode) -> Result<T, Error> {
    Err(Error::new(err_code))
}

pub enum ErrCode {
    EmptyProgram,
    TooManyCliArgs,
    TooManyExprs(ParsePos),  // TODO remove
    CantResolve(Identifier),
    NotEnoughArguments(ParsePos),
    TooManyArguments(ParsePos),
    UnrecognizedToken(ParsePos),
    NotAFunction(ParsePos),
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