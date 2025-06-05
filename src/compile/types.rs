use std::fmt::Display;

use crate::Error;

use super::{Builtin, Expr, FunCall, Position};

/// Type checks an expression tree as a full program
/// The top-level type is guaranteed to be formattable
pub fn typecheck_program(program: &mut Expr) -> Result<(), Error> {
    let top_level_type = program.typecheck()?;

    if !is_formattable(&top_level_type) {
        Err(Error::NonFormattable(format!("{}", top_level_type)))
    }
    else {
        Ok(())
    }
}

fn is_formattable(typ: &Type) -> bool {
    // For now, only streams of a base type are formattable
    match typ.stream_item() {
        Some(Type::Bool)   => true,
        Some(Type::String) => true,
        _                  => false,
    }
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

    fn stream_item(&self) -> Option<&Type> {
        match self {
            Type::Stream(item) => Some(item),
            _ => None,
        }
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
        let fn_type =
            match self.function.as_mut() {
                // TODO we would need to introduce full-fledged type equations here
                Expr::Builtin(Builtin::Filter, _pos) =>
                    typecheck_filter(&mut self.arguments)?,
                Expr::Builtin(Builtin::Map, _pos) =>
                    typecheck_map(&mut self.arguments)?,
                _ => self.function.typecheck()?,
            };

        match fn_type {
            Type::Function { parameters, return_type } => {
                let n_args = self.arguments.len();
                let n_params = parameters.len();

                // Start by checking the number of arguments provided to the function
                assert_ne!(n_params, 0);
                if n_args < n_params {
                    return Err(Error::NotEnoughArguments(self.arguments.last().unwrap().position().right_after()));
                }
                else if n_args > n_params {
                    return Err(Error::TooManyArguments(self.arguments[n_params].position()));
                }

                // Check the types of the arguments
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


            Builtin::Filter | Builtin::Map =>
                todo!("We don't support full type equations yet"),
        }
    }
}

fn mut_pair<T>(data: &mut [T]) -> Option<(&mut T, &mut T)> {
    if data.len() != 2 {
        return None;
    }

    let mut iter = data.into_iter();
    let first = iter.next().unwrap();
    let second = iter.next().unwrap();
    Some((first, second))
}

fn typecheck_filter(arguments: &mut [Expr]) -> Result<Type, Error> {
    let (filter_fn, data_source) =
        match mut_pair(arguments) {
            Some(pair) => pair,
            None => {
                // Ugly hack: return a pointless function of 2 arguments that
                // will fail in the calling FunCall typechecking
                return Ok(Type::function(vec![Type::Bool, Type::Bool], Type::Bool));
            }
        };

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

    let return_type = Type::function(vec![expected_fn_type, source_type.clone()], source_type);
    Ok(return_type)
}

fn typecheck_map(arguments: &mut [Expr]) -> Result<Type, Error> {
    let (map_fn, data_source) =
        match mut_pair(arguments) {
            Some(pair) => pair,
            None => {
                // Ugly hack: return a pointless function of 2 arguments that
                // will fail in the calling FunCall typechecking
                return Ok(Type::function(vec![Type::Bool, Type::Bool], Type::Bool));
            }
        };

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
    // TODO should we remember this mapped-to type in the Map node?
    let mapped_to =
        match &fn_type {
            Type::Function { parameters, return_type } => {
                if parameters.len() == 1 {
                    let single_param = parameters.first().unwrap();
                    if single_param == source_items {
                        // Typecheck ok
                        // Give back the function's return type so we can build the final type out of it
                        return_type.clone()
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
                        found:    fn_type.to_string(),
                        err_pos:  map_fn.position()
                    });
                }
            }
            _ => {
                return Err(Error::WrongArgType {
                    expected: "any function type".into(),
                    found:    fn_type.to_string(),
                    err_pos:  map_fn.position()
                });
            }
        };

    let return_type = Type::function(vec![fn_type, source_type], Type::Stream(mapped_to));
    Ok(return_type)
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