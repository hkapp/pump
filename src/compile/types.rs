use crate::Error;

use super::{Expr, Position};

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
            Expr::Stdin =>
                Ok(Type::stream(Type::String)),

            Expr::Filter { filter_fn, data_source } => {
                let source_type = data_source.typecheck()?;
                let source_items: &Type =
                    match &source_type {
                        Type::Stream(item_type) => item_type,
                        _ => return Err(Error::WrongArgType(data_source.position())),
                    };

                // The filter function must go from the data source's item type to boolean
                let fn_type = filter_fn.typecheck()?;
                let expected_fn_type = Type::function(vec![source_items.clone()], Type::Bool);
                if fn_type != expected_fn_type {
                    return Err(Error::WrongArgType(filter_fn.position()));
                }

                Ok(source_type)
            },

            Expr::Map { map_fn, data_source } => {
                let source_type = data_source.typecheck()?;
                let source_items: &Type =
                    match &source_type {
                        Type::Stream(item_type) => item_type,
                        _ => return Err(Error::WrongArgType(data_source.position())),
                    };

                // The mapping function must take the data source's item type as argument
                let fn_type = map_fn.typecheck()?;
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
                                    return Err(Error::WrongArgType(map_fn.position()));
                                }
                            }
                            else {
                                return Err(Error::WrongArgType(map_fn.position()));
                            }
                        }
                        _ => { return Err(Error::WrongArgType(map_fn.position())); }
                    };

                Ok(Type::Stream(mapped_to))
            },

            Expr::RegexMatch(..) =>
                Ok(Type::function(vec![Type::String], Type::Bool)),

            Expr::RegexSubst(..) =>
                Ok(Type::function(vec![Type::String], Type::String)),

            Expr::UnresolvedIdentifier(identifier) =>
                // We should not reach here with some identifiers still being unresolved
                // This is a logic/programming error
                panic!("Unexpected unresolved identifier during typechecking: {:?}", identifier.name),

            Expr::FunCall { function, arguments } => {
                match function.typecheck()? {
                    Type::Function { parameters, return_type } => {
                        for (param_type, arg)
                        in parameters.into_iter()
                            .zip(arguments.iter_mut())
                        {
                            let arg_type = arg.typecheck()?;
                            if arg_type != param_type {
                                return Err(Error::WrongArgType(arg.position()));
                            }
                        }

                        // Typecheck suceeded
                        Ok(*return_type)
                    }
                    _ => Err(Error::NotAFunction(function.position())),
                }
            },

            Expr::ReadVar(_stream_var) => todo!(),
        }
    }
}