use super::backend::{Arith, Part};
use super::{Generator, RETURN_VARIABLE, error, is_produced, is_user_function};
use crate::builtins::{self, Builtin};
use crate::error::RosellaError;
use crate::syntax::{BinaryOp, Expr, OS};

impl Generator {
    // Rename Local Parameters
    pub(super) fn resolve(&self, name: &str) -> String {
        match &self.function {
            Some(scope) if scope.parameters.contains(name) => {
                self.backend.local_name(&scope.name, name)
            }
            _ => name.to_string(),
        }
    }

    // Calls Run Before The Statement
    pub(super) fn hoist(&mut self, expr: &Expr, setup: &mut String) -> Result<Expr, RosellaError> {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => Ok(Expr::Binary {
                left: Box::new(self.hoist(left, setup)?),
                operator: *operator,
                right: Box::new(self.hoist(right, setup)?),
            }),
            Expr::Call { name, args } if is_produced(name) => {
                let result = self.next_result();
                let local = self.function.is_some();
                let lines = self.produce(name, args, &result, local, setup)?;
                setup.push_str(&lines);
                Ok(Expr::Identifier(result))
            }
            Expr::Call { name, args } if is_user_function(name) => {
                let call = self.user_call(name, args, setup)?;
                setup.push_str(&call);
                if self.share_return {
                    return Ok(Expr::Identifier(RETURN_VARIABLE.to_string()));
                }
                let result = self.next_result();
                setup.push_str(&self.backend.capture(&result, self.function.is_some()));
                Ok(Expr::Identifier(result))
            }
            Expr::Call { name, args } => {
                let mut hoisted = Vec::new();
                for arg in args {
                    hoisted.push(self.hoist(arg, setup)?);
                }
                Ok(Expr::Call {
                    name: name.clone(),
                    args: hoisted,
                })
            }
            other => Ok(other.clone()),
        }
    }

    // Numbers In Text Run First
    pub(super) fn stringify(
        &mut self,
        expr: &Expr,
        setup: &mut String,
    ) -> Result<Expr, RosellaError> {
        match expr {
            Expr::Binary { .. } => {
                let result = self.next_result();
                if self.function.is_some() {
                    setup.push_str(&self.backend.declare_local(&result));
                }
                setup.push_str(&self.backend.assign_int(&result, &self.arith(expr)?));
                Ok(Expr::Identifier(result))
            }
            Expr::Call { name, args }
                if matches!(
                    builtins::find(name).map(|signature| signature.builtin),
                    Some(Builtin::Concat | Builtin::Path)
                ) =>
            {
                let mut converted = Vec::new();
                for arg in args {
                    converted.push(self.stringify(arg, setup)?);
                }
                Ok(Expr::Call {
                    name: name.clone(),
                    args: converted,
                })
            }
            other => Ok(other.clone()),
        }
    }

    pub(super) fn next_result(&mut self) -> String {
        let result = format!("rosella_result{}", self.results);
        self.results += 1;
        result
    }

    pub(super) fn arith(&self, expr: &Expr) -> Result<Arith, RosellaError> {
        Ok(Arith {
            text: self.arithmetic(expr)?,
            literal: match expr {
                Expr::Number(n) => Some(*n),
                _ => None,
            },
            grouped: matches!(expr, Expr::Binary { .. }),
            known: match expr {
                Expr::Identifier(name) => self.always_set(name),
                _ => false,
            },
        })
    }

    // Generated Names Are Always Set Before Use
    fn always_set(&self, name: &str) -> bool {
        let resolved = self.resolve(name);
        resolved.starts_with("rosella_")
            || self.counters.contains(&resolved)
            || (self.function.is_none() && self.assigned.iter().any(|other| other == name))
    }

    fn arithmetic(&self, expr: &Expr) -> Result<String, RosellaError> {
        match expr {
            Expr::Number(n) if *n < 0 => Ok(format!("({})", n)),
            Expr::Number(n) => Ok(n.to_string()),
            Expr::Identifier(name) => Ok(self.resolve(name)),
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let symbol = match operator {
                    BinaryOp::Add => "+",
                    BinaryOp::Subtract => "-",
                    BinaryOp::Multiply => "*",
                    BinaryOp::Divide => "/",
                    BinaryOp::Modulo => "%",
                    _ => {
                        return Err(error(
                            "Comparisons, && and || can only be used in if and while conditions",
                        ));
                    }
                };
                Ok(format!(
                    "{} {} {}",
                    self.arith(left)?.operand(),
                    symbol,
                    self.arith(right)?.operand()
                ))
            }
            Expr::String(s) => Err(error(format!(
                "The string \"{}\" cannot be used as an int",
                s
            ))),
            Expr::Call { name, .. } => Err(error(format!("{}() does not return an int", name))),
            Expr::Not(_) => Err(error("! can only be used in if and while conditions")),
        }
    }

    pub(super) fn value_parts(&self, expr: &Expr) -> Result<Vec<Part>, RosellaError> {
        match expr {
            Expr::String(s) => Ok(vec![Part::Text(s.clone())]),
            Expr::Number(n) => Ok(vec![Part::Text(n.to_string())]),
            Expr::Identifier(name) => Ok(vec![Part::Var(self.resolve(name))]),
            Expr::Call { name, args } => {
                match builtins::find(name).map(|signature| signature.builtin) {
                    Some(Builtin::Concat) => self.concat_parts(args),
                    Some(Builtin::Path) => self.path_parts(args),
                    Some(Builtin::GetCwd) => Ok(vec![Part::Cwd]),
                    Some(Builtin::ScriptDir) => {
                        Ok(vec![Part::Var("rosella_script_dir".to_string())])
                    }
                    Some(Builtin::Env) => match args.first() {
                        Some(Expr::String(variable)) => Ok(vec![Part::Var(variable.clone())]),
                        _ => Err(error("env() needs the variable name written in quotes")),
                    },
                    _ => Err(error(format!("{}() cannot be used as a value", name))),
                }
            }
            Expr::Binary { .. } => Err(error(
                "Strings cannot be combined with operators; use concat() to join them",
            )),
            Expr::Not(_) => Err(error("! can only be used in if and while conditions")),
        }
    }

    pub(super) fn concat_parts(&self, args: &[Expr]) -> Result<Vec<Part>, RosellaError> {
        let mut parts = Vec::new();
        for arg in args {
            parts.extend(self.value_parts(arg)?);
        }
        Ok(parts)
    }

    pub(super) fn path_parts(&self, args: &[Expr]) -> Result<Vec<Part>, RosellaError> {
        let separator = match self.os {
            OS::Windows => '\\',
            OS::Linux => '/',
        };
        let mut parts: Vec<Part> = Vec::new();

        for (index, arg) in args.iter().enumerate() {
            let mut arg_parts = self.value_parts(arg)?;
            for part in &mut arg_parts {
                if let Part::Text(text) = part {
                    *text = text.replace(['/', '\\'], &separator.to_string());
                }
            }

            let ends_with_separator =
                matches!(parts.last(), Some(Part::Text(text)) if text.ends_with(separator));
            let starts_with_separator =
                matches!(arg_parts.first(), Some(Part::Text(text)) if text.starts_with(separator));
            if index > 0 && !ends_with_separator && !starts_with_separator {
                parts.push(Part::Text(separator.to_string()));
            }

            parts.extend(arg_parts);
        }

        Ok(parts)
    }
}
