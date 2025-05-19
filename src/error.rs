use std::{fmt::Display, io};

use crate::parse::{ParsePos, Identifier};

pub struct Error {
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

    pub fn format<W: io::Write>(&self, source: &str, buf: &mut W) -> io::Result<()> {
        self.position.format(source, buf)?;
        writeln!(buf)?;
        write!(buf, "pump: {}", self.error_code)
    }
}

/// Will always return Err
pub fn error<T>(err_code: ErrCode, err_pos: ParsePos) -> Result<T, Error> {
    Err(Error::new(err_code, err_pos))
}

/// Will always return Err
// FIXME introduce the actual concept of "no position"
pub fn error_no_pos<T>(err_code: ErrCode) -> Result<T, Error> {
    Err(Error::new(err_code, ParsePos{ start: 0, len: 1 }))
}

pub enum ErrCode {
    EmptyProgram,
    TooManyArguments,
    TooManyExprs,  // TODO remove
    CantResolve(Identifier),
    NotEnoughArguments,
    UnrecognizedToken,
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
            ErrCode::NotEnoughArguments =>
                write!(f, "Not enough arguments in function call"),
            ErrCode::UnrecognizedToken =>
                write!(f, "Unrecognized token"),
        }
    }
}