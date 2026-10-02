use super::backend::{Arg, Arith, Backend, Condition, Part, Test, Transfer};
use super::indent;
use crate::error::RosellaError;
use crate::syntax::BinaryOp;

pub struct Bash;

impl Backend for Bash {
    fn program(&mut self, body: String) -> String {
        format!("#!/bin/bash\n{}", body)
    }

    fn empty_body(&self) -> &'static str {
        ":\n"
    }

    fn let_int(&self, name: &str, value: &Arith) -> String {
        match value.literal {
            Some(literal) => format!("{}={}\n", name, literal),
            None => format!("{}=$(( {} ))\n", name, value.text),
        }
    }

    fn let_str(&self, name: &str, value: &[Part]) -> Result<String, RosellaError> {
        Ok(format!("{}={}\n", name, quote(value, false)))
    }

    fn condition(&mut self, test: Test) -> Result<Condition, RosellaError> {
        let test = match test {
            Test::Int {
                left,
                operator,
                right,
            } => format!(
                "(( {} {} {} ))",
                left.operand(),
                symbol(operator),
                right.operand()
            ),
            Test::Str {
                left,
                operator,
                right,
            } => format!(
                "[[ {} {} {} ]]",
                quote(&left, false),
                symbol(operator),
                quote(&right, false)
            ),
            Test::File { negate, path } => format!(
                "[[ {}-e {} ]]",
                if negate { "! " } else { "" },
                quote(&path, false)
            ),
        };

        Ok(Condition {
            setup: String::new(),
            test,
        })
    }

    fn if_chain(&self, branches: Vec<(Condition, String)>, otherwise: Option<String>) -> String {
        let mut output = String::new();

        for (index, (condition, body)) in branches.into_iter().enumerate() {
            let keyword = if index == 0 { "if" } else { "elif" };
            output.push_str(&format!("{} {}; then\n{}", keyword, condition.test, body));
        }
        if let Some(otherwise) = otherwise {
            output.push_str(&format!("else\n{}", otherwise));
        }

        output.push_str("fi\n");
        output
    }

    fn while_loop(&mut self, condition: Condition, body: String) -> String {
        format!("while {}; do\n{}done\n", condition.test, body)
    }

    fn function(&mut self, name: &str, parameters: &[String], body: String) -> String {
        let mut output = format!("{}() {{\n", name);
        for (index, parameter) in parameters.iter().enumerate() {
            output.push_str(&indent(&format!(
                "local {}=\"${{{}}}\"\n",
                parameter,
                index + 1
            )));
        }
        output.push_str(&body);
        output.push_str("}\n");
        output
    }

    fn call(&self, name: &str, args: &[Arg]) -> Result<String, RosellaError> {
        let mut output = name.to_string();
        for arg in args {
            output.push(' ');
            output.push_str(&match arg {
                Arg::Int(value) => format!("\"$(( {} ))\"", value.text),
                Arg::Value(parts) => quote(parts, false),
            });
        }
        output.push('\n');
        Ok(output)
    }

    fn print(&self, text: &[Part]) -> Result<String, RosellaError> {
        Ok(format!("printf '%s\\n' {}\n", quote(text, false)))
    }

    fn cd(&self, path: &[Part]) -> Result<String, RosellaError> {
        Ok(format!("cd {}\n", quote(path, false)))
    }

    fn make_dir(&self, path: &[Part]) -> Result<String, RosellaError> {
        Ok(format!("mkdir -p -- {}\n", quote(path, false)))
    }

    fn remove(
        &self,
        path: &[Part],
        directory: bool,
        _depth: usize,
    ) -> Result<String, RosellaError> {
        let command = if directory { "rm -rf" } else { "rm -f" };
        Ok(format!("{} -- {}\n", command, quote(path, true)))
    }

    fn transfer(
        &self,
        transfer: Transfer,
        source: &[Part],
        destination: &[Part],
    ) -> Result<String, RosellaError> {
        let command = match transfer {
            Transfer::Copy => "cp",
            Transfer::Move => "mv",
        };
        Ok(format!(
            "{} -- {} {}\n",
            command,
            quote(source, false),
            quote(destination, false)
        ))
    }

    fn write(&self, path: &[Part], content: &[Part], append: bool) -> Result<String, RosellaError> {
        Ok(format!(
            "printf '%s\\n' {} {} {}\n",
            quote(content, false),
            if append { ">>" } else { ">" },
            quote(path, false)
        ))
    }

    fn read(&self, prompt: &[Part], variable: &str) -> Result<String, RosellaError> {
        Ok(format!(
            "read -r -p {} {}\n",
            quote(prompt, false),
            variable
        ))
    }

    fn exit(&self, code: &Arith, _depth: usize) -> String {
        match code.literal {
            Some(literal) => format!("exit {}\n", literal),
            None => format!("exit $(( {} ))\n", code.text),
        }
    }
}

fn symbol(operator: BinaryOp) -> &'static str {
    match operator {
        BinaryOp::Equal => "==",
        BinaryOp::NotEqual => "!=",
        BinaryOp::LessThan => "<",
        BinaryOp::LessThanEq => "<=",
        BinaryOp::GreaterThan => ">",
        BinaryOp::GreaterThanEq => ">=",
        BinaryOp::Add => "+",
        BinaryOp::Subtract => "-",
        BinaryOp::Multiply => "*",
        BinaryOp::Divide => "/",
    }
}

fn quote(parts: &[Part], guard_empty: bool) -> String {
    let mut output = String::from('"');

    for part in parts {
        match part {
            Part::Text(text) => {
                for ch in text.chars() {
                    match ch {
                        // Avoid Literal Line Breaks
                        '\n' => output.push_str("\"$'\\n'\""),
                        '\\' | '"' | '$' | '`' => {
                            output.push('\\');
                            output.push(ch);
                        }
                        _ => output.push(ch),
                    }
                }
            }
            // Stop On Empty Variable
            Part::Var(name) if guard_empty => output.push_str(&format!("${{{}:?}}", name)),
            Part::Var(name) => output.push_str(&format!("${{{}}}", name)),
            Part::Cwd => output.push_str("${PWD}"),
        }
    }

    output.push('"');
    output
}
