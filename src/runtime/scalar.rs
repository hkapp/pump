use std::io::Read;

use regex::Regex;

use crate::{error::Error, parse::Expr};

use super::{RtVal, StreamVar};

/// Runtime components that return scalar values
pub trait ExecScalar {
    fn eval(&mut self) -> Result<RtVal, Error>;
}

pub enum ScalarNode {
    RegexMatch(RegexMatch),
    ReadStreamVar(ReadStreamVar),
}

impl ExecScalar for ScalarNode {
    fn eval(&mut self) -> Result<RtVal, Error> {
        match self {
            Self::RegexMatch(r) => r.eval(),
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