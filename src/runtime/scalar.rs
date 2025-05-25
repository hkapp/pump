use regex::Regex;

use crate::{error::Error, parse::{self, Expr}};

use super::{RtVal, StreamVar};

/// Runtime components that return scalar values
pub trait ExecScalar {
    fn eval(&mut self) -> Result<RtVal, Error>;
}

pub enum ScalarNode {
    RegexMatch(RegexMatch),
    RegexSubst(RegexSubst),
    ReadStreamVar(ReadStreamVar),
}

impl ExecScalar for ScalarNode {
    fn eval(&mut self) -> Result<RtVal, Error> {
        match self {
            Self::RegexMatch(r) => r.eval(),
            Self::RegexSubst(subst) => subst.eval(),
            Self::ReadStreamVar(rsv) => rsv.eval(),
        }
    }
}

// TODO make this take a box
// TODO convert into From impl
pub fn scalar_from(expr: Expr) -> ScalarNode {
    match expr {
        Expr::FunCall { function, arguments } =>
            scalar_fun_call(*function, arguments),

        Expr::ReadVar(var) => ReadStreamVar::new_node(var),

        Expr::RegexMatch(..) => panic!("We don't expect a RegexMatch outside of a FunCall anymore"),

        // It's fine for us to panic here, as typechecking must have guaranteed that
        // we have what our caller expects here
        _ => panic!("Not a scalar: {:?}", expr),
    }
}

fn scalar_fun_call(function: Expr, mut arguments: Vec<Expr>) -> ScalarNode {
    match function {
        Expr::RegexMatch(regex, pos) => {
            assert_eq!(arguments.len(), 1);
            let single_arg = arguments.pop().unwrap();
            RegexMatch::new_node(regex, single_arg)
        }
        Expr::RegexSubst(subst) => {
            assert_eq!(arguments.len(), 1);
            let single_arg = arguments.pop().unwrap();
            RegexSubst::new_node(subst, single_arg)
        }
        _ => todo!(),
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
    fn new_node(subst: parse::RegexSubst, arg: Expr) -> ScalarNode {
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