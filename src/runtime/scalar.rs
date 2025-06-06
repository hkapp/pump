use regex::Regex;

use crate::error::Error;
use crate::compile::{self, Builtin, Expr, ParsePos};
use crate::Position;

use super::{RtVal, StreamVar, Number};

/// Runtime components that return scalar values
pub trait ExecScalar {
    fn eval(&mut self) -> Result<RtVal, Error>;
}

#[allow(private_interfaces)]
pub enum ScalarNode {
    RegexMatch(RegexMatch),
    RegexSubst(RegexSubst),
    ReadStreamVar(ReadStreamVar),
    ToNumber(ToNumber)
}

impl ExecScalar for ScalarNode {
    fn eval(&mut self) -> Result<RtVal, Error> {
        match self {
            Self::RegexMatch(r) => r.eval(),
            Self::RegexSubst(subst) => subst.eval(),
            Self::ReadStreamVar(rsv) => rsv.eval(),
            Self::ToNumber(n) => n.eval(),
        }
    }
}

// TODO convert into From impl
pub fn scalar_from(expr: Expr) -> ScalarNode {
    match expr {
        Expr::FunCall(fcall) => scalar_fun_call(fcall),

        Expr::ReadVar(var) => ReadStreamVar::new_node(var),

        // It's fine for us to panic here, as typechecking must have guaranteed that
        // we have what our caller expects here
        _ => panic!("Not a scalar: {:?}", expr),
    }
}

fn scalar_fun_call(mut fcall: compile::FunCall) -> ScalarNode {
    match *fcall.function {
        Expr::Builtin(b, pos) => {
            match b {
                Builtin::RegexMatch(regex) => {
                    assert_eq!(fcall.arguments.len(), 1);
                    let single_arg = fcall.arguments.pop().unwrap();
                    RegexMatch::new_node(regex, single_arg)
                }
                Builtin::RegexSubst(subst) => {
                    assert_eq!(fcall.arguments.len(), 1);
                    let single_arg = fcall.arguments.pop().unwrap();
                    RegexSubst::new_node(subst, single_arg)
                }
                Builtin::ToNumber => {
                    assert_eq!(fcall.arguments.len(), 1);
                    let single_arg = fcall.arguments.pop().unwrap();
                    ToNumber::new_node(single_arg, pos)
                }
                _ => panic!("Not a scalar builtin: {:?}", b),
            }
        }
        _ => panic!("Not a scalar expression: {:?}", fcall),
    }
}

/* RegexMatch */

struct RegexMatch {
    regex:    Regex,
    argument: Box<ScalarNode>,
}

impl RegexMatch {
    fn new_node(regex: Regex, arg: Expr) -> ScalarNode {
        let rt_arg = scalar_from(arg);
        let argument = Box::new(rt_arg);
        let me = RegexMatch { regex , argument };
        ScalarNode::RegexMatch(me)
    }
}

impl ExecScalar for RegexMatch {
    fn eval(&mut self) -> Result<RtVal, Error> {
        let input = self.argument.eval()?;
        let is_match = self.regex.is_match(&input.str_ref().unwrap());
        let rt_val = is_match.into();
        Ok(rt_val)
    }
}

/* RegexSubst */

struct RegexSubst {
    search:   Regex,
    replace:  String,
    argument: Box<ScalarNode>,
}

use std::cell::LazyCell;
const REGEX_GROUP_ID: LazyCell<regex::Regex> =
    LazyCell::new(|| regex::Regex::new(r"\\(\d)").unwrap());

impl RegexSubst {
    fn new_node(subst: compile::RegexSubst, arg: Expr) -> ScalarNode {
        let rt_arg = scalar_from(arg);
        let argument = Box::new(rt_arg);

        let search = subst.search;

        // The user-provided input contains sequences like "\1" to refer to named capture groups
        // We replace these with "$1" to be able to use the native replace capabilities of
        // the regex crate.
        let replace =
            REGEX_GROUP_ID.replace_all(&subst.replace, |rec: &regex::Captures| {
                // Generate "${1}" to be 100% clear to the regex crate
                format!("${{{}}}", rec.get(1).unwrap().as_str())
            })
            .into_owned();
        // DEBUG
        eprintln!("RegexSubst::new_node: {:?}", replace);

        let me = RegexSubst { search, replace, argument };
        ScalarNode::RegexSubst(me)
    }
}

impl ExecScalar for RegexSubst {
    fn eval(&mut self) -> Result<RtVal, Error> {
        let input = self.argument.eval()?;
        let str_input = input.str_ref().unwrap();

        let replaced_str =
            self.search
                .replace(str_input, &self.replace)
                .into_owned();

        let rt_val = replaced_str.into();
        Ok(rt_val)
    }
}

/* ToNumber */

struct ToNumber {
    argument: Box<ScalarNode>,
    src_pos:  ParsePos,
}

impl ToNumber {
    fn new_node(arg: Expr, to_num_pos: ParsePos) -> ScalarNode {
        let rt_arg = scalar_from(arg);
        let argument = Box::new(rt_arg);

        let me = ToNumber { argument, src_pos: to_num_pos };
        ScalarNode::ToNumber(me)
    }
}

impl ExecScalar for ToNumber {
    fn eval(&mut self) -> Result<RtVal, Error> {
        let input = self.argument.eval()?;
        let str_input = input.str_ref().unwrap();

        use std::str::FromStr;
        let num_value =
            Number::from_str(str_input)
                .map_err(|parse_err|
                    Error::NotANumber {
                        str_value: str_input.into(),
                        parse_err,
                        err_pos: self.src_pos
                    })?;

        let rt_val = num_value.into();
        Ok(rt_val)
    }
}

/* ReadStreamVar */
struct ReadStreamVar {
    var: StreamVar,
}

impl ReadStreamVar {
    fn new_node(var: StreamVar) -> ScalarNode {
        let me = Self { var };
        ScalarNode::ReadStreamVar(me)
    }
}

impl ExecScalar for ReadStreamVar {
    fn eval(&mut self) -> Result<RtVal, Error> {
        let var_content = self.var.read().unwrap();
        Ok(var_content)
    }
}