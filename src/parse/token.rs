use std::io;
use std::fmt::Debug;

use regex::Regex;

pub struct Identifier {
    pub name:     String,
    pub position: ParsePos
}

impl Identifier {
    fn token(m: &regex::Match) -> Token {
        println!("Identifier: {}", m.as_str());
        let pos = ParsePos::from(m);
        let idn =
            Identifier {
                name:     m.as_str().into(),
                position: pos,
            };
        Token { position: pos, kind: Kind::Identifier(idn) }
    }
}

pub fn tokenize<'a>(s: &'a str) -> Tokenizer<'a> {
    Tokenizer {
        source:   s,
        curr_pos: 0,
        token_rxs:  TOKEN_RXS.iter().map(TokenRx::from).collect(),
    }
}

pub struct Tokenizer<'a> {
    source:   &'a str,
    curr_pos: usize,
    token_rxs:  Vec<TokenRx>,
}

pub struct Token {
    pub position: ParsePos,
    pub kind:     Kind,
}

pub enum Kind {
    Identifier(Identifier),
    RegexMatch(Regex),
}

impl Token {
    fn len(&self) -> usize {
        self.position.len
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}@{}+{}", self.kind, self.position.start, self.position.len)
    }
}

impl Debug for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Identifier(idn) => write!(f, "Identifier({:?})", idn.name),
            Kind::RegexMatch(re) => write!(f, "RegexMatch({:?})", re.as_str()),
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.trim_leading_whitespaces();

        if self.at_end() {
            // Reached the end of the source
            return None;
        }

        let first_success =
            self.token_rxs
                .iter()
                .find_map(|trx| trx.try_at(self.source, self.curr_pos));

        match first_success {
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
            assert!(self.at_end() || self.curr_pos == 0,
                "No trimmed spaces at position {}, i.e. {:?}", self.curr_pos, &self.source[self.curr_pos..]);
        }
        else {
            self.curr_pos += len_diff;
        }
    }
}

// Note: this definition forbids using the From trait implementation
// TODO we can turn this into a function that returns a Kind
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

fn regex_match(m: &regex::Match) -> Token {
    println!("regex_match: {}", m.as_str());
    // FIXME we need to get a regex::Captures instead as argument
    let regex_substr = &m.as_str()[2..(m.len()-1)];
    // FIXME need to return a proper error here
    let re_match = Regex::new(regex_substr).unwrap();

    let pos = ParsePos::from(m);
    Token { position: pos, kind: Kind::RegexMatch(re_match) }
}

const TOKEN_RXS: [TRDef; 2] = [
    // WARNING the ordering matters here
    // ("m/(?:[^/]|\\/)*/\\>", regex_match),
    ("m/(?:[^/]|\\/)*/", regex_match),
    ("[a-zA-Z][0-9a-zA-Z]*", Identifier::token),
];

impl TokenRx {
    fn try_at(&self, source: &str, start: usize) -> Option<Token> {
        self.regex
            .find_at(source, start)
            .filter(|m| m.start() == start)
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

    fn from(m: &regex::Match) -> Self {
        ParsePos {
            start: m.start(),
            len:   m.as_str().len()
        }
    }
}