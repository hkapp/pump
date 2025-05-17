use regex::Regex;

use crate::{error::Error, parse::Expr};

use super::RtVal;

/// Runtime components that return scalar values
pub trait ExecScalar {
    // FIXME remove the extra input argument
    fn eval(&mut self, input: RtVal) -> Result<RtVal, Error>;
}

pub enum ScalarNode {
    RegexMatch(RegexMatch),
}

impl ExecScalar for ScalarNode {
    fn eval(&mut self, input: RtVal) -> Result<RtVal, Error> {
        match self {
            Self::RegexMatch(r) => r.eval(input),
        }
    }
}

// TODO make this take a box
// TODO convert into From impl
pub fn scalar_from(expr: Expr) -> ScalarNode {
    match expr {
        Expr::RegexMatch(regex, pos) => RegexMatch::new_node(regex),
        // It's fine for us to panic here, as typechecking must have guaranteed that
        // we have what our caller expects here
        _ => panic!("Not a scalar: {:?}", expr),
    }
}

/* RegexMatch */

struct RegexMatch {
    regex: Regex,
}

impl RegexMatch {
    fn new_node(regex: Regex) -> ScalarNode {
        let me = RegexMatch { regex };
        ScalarNode::RegexMatch(me)
    }
}

impl ExecScalar for RegexMatch {
    fn eval(&mut self, input: RtVal) -> Result<RtVal, Error> {
        // FIXME
        let is_match = self.regex.is_match(&input.str_ref().unwrap());
        let rt_val = is_match.into();
        Ok(rt_val)
    }
}