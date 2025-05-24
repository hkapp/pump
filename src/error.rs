use std::{fmt::Display, io};

use crate::parse::{ParsePos, Identifier};

// TODO remove
pub struct Error {
    error_code: ErrCode
}

impl Error {
    fn new(err_code: ErrCode, err_pos: ParsePos) -> Self {
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
            ErrCode::TooManyArguments => None,
            ErrCode::TooManyExprs => None,
            ErrCode::CantResolve(idn) => Some(idn.position),
            ErrCode::NotEnoughArguments => None,
            ErrCode::UnrecognizedToken => None,
            ErrCode::NotAFunction(err_pos) => Some(*err_pos),
        }
    }
}

fn write_error_line<W: io::Write>(err_pos: ParsePos, buf: &mut W) -> io::Result<()> {
    // TODO support multi-line programs
    writeln!(buf, "{}{}", str::repeat(" ", err_pos.start), str::repeat("^", err_pos.len))
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
    NotAFunction(ParsePos),
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
            ErrCode::NotAFunction(_) =>
                write!(f, "Not a function"),
        }
    }
}