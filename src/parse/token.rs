use std::io;

use regex::Regex;
use lazy_static::lazy_static; // FIXME should be using LazyCell here, but couldn't get it working

pub struct Identifier {
    pub name:     String,
    pub position: ParsePos
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

pub fn tokenize(s: &str) -> impl Iterator<Item=Identifier> + '_ {
    IDN_REGEX.find_iter(s)
        .map(Into::into)
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