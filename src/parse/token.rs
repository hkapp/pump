use std::io;

use regex::Regex;

pub struct Identifier {
    pub name:     String,
    pub position: ParsePos
}

impl Identifier {
    fn from(m: &regex::Match) -> Self {
        Identifier {
            name:     m.as_str().into(),
            position: ParsePos {
                start: m.start(),
                len:   m.as_str().len()
            },
        }
    }

    fn len(&self) -> usize {
        self.position.len
    }
}

pub fn tokenize<'a>(s: &'a str) -> Tokenizer<'a> {
    Tokenizer {
        source:   s,
        curr_pos: 0,
        regexes:  TOKEN_RXS.iter().map(TokenRx::from).collect(),
    }
}

pub struct Tokenizer<'a> {
    source:   &'a str,
    curr_pos: usize,
    regexes:  Vec<TokenRx>,
}

type Token = Identifier;

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.trim_leading_whitespaces();

        if self.at_end() {
            // Reached the end of the source
            return None;
        }

        match self.regexes[0].try_at(self.source, self.curr_pos) {
            Some(token) => {
                self.curr_pos += token.len();
                Some(token)
            },
            None => {
                // We could not parse the next token
                // FIXME return Err instead here
                panic!("Unrecognized token");
            }
        }
    }
}

impl<'a> Tokenizer<'a> {
    fn at_end(&self) -> bool {
        self.curr_pos == self.source.len()
    }

    fn trim_leading_whitespaces(&mut self) {
        // TODO use a constant regex
        let rem_source = &self.source[self.curr_pos..];
        let rem_no_ws = rem_source.trim_start();
        let len_diff = rem_source.len() - rem_no_ws.len();

        if len_diff == 0 {
            // Considering how the tokenizer works, this can never happen before the end of the source program
            assert!(self.at_end() || self.curr_pos == 0);
        }
        else {
            self.curr_pos += len_diff;
        }
    }
}

// Note: this definition forbids using the From trait implementation
type BuildFn = fn(&regex::Match) -> Token;

struct TokenRx {
    regex:    Regex,
    build_fn: BuildFn,
}

type TRDef = (&'static str, BuildFn);

impl From<&TRDef> for TokenRx {
    fn from(trdef: &TRDef) -> Self {
        TokenRx {
            regex:    Regex::new(trdef.0).unwrap(),
            build_fn: trdef.1
        }
    }
}

const TOKEN_RXS: [TRDef; 1] = [
    // WARNING the ordering matters here
    ("[a-zA-Z][0-9a-zA-Z]*", Identifier::from)
];

impl TokenRx {
    fn try_at(&self, source: &str, start: usize) -> Option<Token> {
        self.regex
            .find_at(source, start)
            .map(|m| (self.build_fn)(&m))
    }
}

#[derive(Clone, Copy)]
pub struct ParsePos {
    pub start: usize,
    pub len:   usize
}

impl ParsePos {
    // TODO move this function into crate::error
    pub fn format<W: io::Write>(&self, source: &str, buf: &mut W) -> io::Result<()> {
        // TODO support multi-line programs
        writeln!(buf, "{}", source)?;
        write!(buf, "{}{}", str::repeat(" ", self.start), str::repeat("^", self.len))
    }
}