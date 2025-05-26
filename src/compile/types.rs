use std::fmt::Display;

use crate::Error;

use super::{Builtin, Expr, FunCall, Position};

pub fn typecheck(program: &mut Expr) -> Result<(), Error> {
    program.typecheck()?;
    Ok(())
}

/* Type */

#[derive(Clone, PartialEq, Eq)]
enum Type {
    Bool,
    String,
    Stream(Box<Type>),
    Function { parameters: Vec<Type>, return_type: Box<Type> },
}

impl Type {
    fn stream(of_what: Type) -> Self {
        Self::Stream(Box::new(of_what))
    }

    fn function(parameters: Vec<Type>, return_type: Type) -> Self {
        Self::Function { parameters, return_type: Box::new(return_type) }
    }
}

/* Typecheck trait and logic */

trait Typecheck {
    fn typecheck(&mut self) -> Result<Type, Error>;
}

impl Typecheck for Expr {
    fn typecheck(&mut self) -> Result<Type, Error> {
        match self {
            Expr::Builtin(b, _pos) =>
                b.typecheck(),

            Expr::UnresolvedIdentifier(identifier) =>
                // We should not reach here with some identifiers still being unresolved
                // This is a logic/programming error
                panic!("Unexpected unresolved identifier during typechecking: {:?}", identifier.name),

            Expr::FunCall(fcall) =>
                fcall.typecheck(),

            Expr::ReadVar(_stream_var) => todo!(),
        }
    }
}

impl Typecheck for FunCall {
    fn typecheck(&mut self) -> Result<Type, Error> {
        match self.function.typecheck()? {
            Type::Function { parameters, return_type } => {
                for (param_type, arg)
                in parameters.into_iter()
                    .zip(self.arguments.iter_mut())
                {
                    let arg_type = arg.typecheck()?;
                    if arg_type != param_type {
                        return Err(Error::WrongArgType {
                            expected: param_type.to_string(),
                            found:    arg_type.to_string(),
                            err_pos:  arg.position()
                        });
                    }
                }

                // Typecheck suceeded
                Ok(*return_type)
            }
            _ => Err(Error::NotAFunction(self.function.position())),
        }
    }
}

impl Typecheck for Builtin {
    fn typecheck(&mut self) -> Result<Type, Error> {
        match self {
            Builtin::Stdin =>
                Ok(Type::stream(Type::String)),
            Builtin::RegexMatch(..) =>
                Ok(Type::function(vec![Type::String], Type::Bool)),
            Builtin::RegexSubst(..) =>
                Ok(Type::function(vec![Type::String], Type::String)),


            Builtin::Filter { filter_fn, data_source } => {
                let source_type = data_source.typecheck()?;
                let source_items: &Type =
                    match &source_type {
                        Type::Stream(item_type) => item_type,
                        _ => return Err(Error::WrongArgType {
                            expected: "any stream type".into(),
                            found:    source_type.to_string(),
                            err_pos:  data_source.position()
                        }),
                    };

                // The filter function must go from the data source's item type to boolean
                let fn_type = filter_fn.typecheck()?;
                let expected_fn_type = Type::function(vec![source_items.clone()], Type::Bool);
                if fn_type != expected_fn_type {
                    return Err(Error::WrongArgType {
                        expected: expected_fn_type.to_string(),
                        found:    fn_type.to_string(),
                        err_pos:  filter_fn.position()
                    });
                }

                Ok(source_type)
            },

            Builtin::Map { map_fn, data_source } => {
                let source_type = data_source.typecheck()?;
                let source_items: &Type =
                    match &source_type {
                        Type::Stream(item_type) => item_type,
                        _ => return Err(Error::WrongArgType {
                            expected: "any stream type".into(),
                            found:    source_type.to_string(),
                            err_pos:  data_source.position()
                        }),
                    };

                // The mapping function must take the data source's item type as argument
                let fn_type = map_fn.typecheck()?;
                let fn_type_str = fn_type.to_string();  // used to beat the borrow checker
                // TODO should we remember this mapped-to type in the Map node?
                let mapped_to =
                    match fn_type {
                        Type::Function { parameters, return_type } => {
                            if parameters.len() == 1 {
                                let single_param = parameters.first().unwrap();
                                if single_param == source_items {
                                    // Typecheck ok
                                    // Give back the function's return type so we can build the final type out of it
                                    return_type
                                }
                                else {
                                    return Err(Error::WrongArgType {
                                        expected: format!("fn ({}) -> anything", source_items.to_string()),
                                        found:    single_param.to_string(),
                                        err_pos:  map_fn.position()
                                    });
                                }
                            }
                            else {
                                return Err(Error::WrongArgType {
                                    expected: "a function of a single argument".into(),
                                    found:    fn_type_str,
                                    err_pos:  map_fn.position()
                                });
                            }
                        }
                        _ => {
                            return Err(Error::WrongArgType {
                                expected: "any function type".into(),
                                found:    fn_type_str,
                                err_pos:  map_fn.position()
                            });
                        }
                    };

                Ok(Type::Stream(mapped_to))
            },
        }
    }
}

/* Pretty printing */

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Stream(item) => write!(f, "stream of {}", item),
            Type::Function { parameters, return_type } => {
                write!(f, "fn (")?;
                for (idx, param) in parameters.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", return_type)
            }
        }
    }
}