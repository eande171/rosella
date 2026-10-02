use super::backend::{Arith, Part};
use super::{Generator, error};
use crate::builtins::{self, Builtin};
use crate::error::RosellaError;
use crate::syntax::{BinaryOp, Expr, OS};

impl Generator {
    // Rename Local Parameters
    pub(super) fn resolve(&self, name: &str) -> String {
        match &self.function {
            Some((function, parameters)) if parameters.contains(name) => {
                self.backend.local_name(function, name)
            }
            _ => name.to_string(),
        }
    }

    pub(super) fn arith(&self, expr: &Expr) -> Result<Arith, RosellaError> {
        Ok(Arith {
            text: self.arithmetic(expr)?,
            literal: match expr {
                Expr::Number(n) => Some(*n as i64),
                _ => None,
            },
            grouped: matches!(expr, Expr::Binary { .. }),
        })
    }

    fn arithmetic(&self, expr: &Expr) -> Result<String, RosellaError> {
        match expr {
            Expr::Number(n) if *n < 0.0 => Ok(format!("({})", integer(*n)?)),
            Expr::Number(n) => integer(*n),
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
                    _ => {
                        return Err(error(
                            "Comparisons can only be used in if and while conditions",
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
        }
    }

    pub(super) fn value_parts(&self, expr: &Expr) -> Result<Vec<Part>, RosellaError> {
        match expr {
            Expr::String(s) => Ok(vec![Part::Text(s.clone())]),
            Expr::Number(n) => Ok(vec![Part::Text(format_number(*n))]),
            Expr::Identifier(name) => Ok(vec![Part::Var(self.resolve(name))]),
            Expr::Call { name, args } => {
                match builtins::find(name).map(|signature| signature.builtin) {
                    Some(Builtin::Concat) => self.concat_parts(args),
                    Some(Builtin::Path) => self.path_parts(args),
                    Some(Builtin::GetCwd) => Ok(vec![Part::Cwd]),
                    _ => Err(error(format!("{}() cannot be used as a value", name))),
                }
            }
            Expr::Binary { .. } => Err(error(
                "Strings cannot be combined with operators; use concat() to join them",
            )),
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

fn integer(n: f64) -> Result<String, RosellaError> {
    // Batch Uses 32 Bit Integers
    if n.fract() != 0.0 || n.abs() > i32::MAX as f64 {
        return Err(error(format!(
            "{} is not a whole number that int can hold",
            n
        )));
    }
    Ok(format!("{}", n as i64))
}

fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        n.to_string()
    }
}
