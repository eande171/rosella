use super::backend::{Arg, Arith, Backend, Condition, Part, ReturnCheck, Test, Transfer};
use super::{error, indent};
use crate::error::RosellaError;
use crate::syntax::BinaryOp;

#[derive(Default)]
pub struct Batch {
    subroutines: String,
    unique_index: usize,
}

impl Batch {
    fn next_index(&mut self) -> usize {
        let index = self.unique_index;
        self.unique_index += 1;
        index
    }

    // Precompute If Operands
    fn int_operand(&mut self, value: &Arith, setup: &mut String) -> String {
        if let Some(literal) = value.literal {
            return literal.to_string();
        }

        let temporary = format!("rosella_temp{}", self.next_index());
        setup.push_str(&format!("set /a \"{}={}\"\n", temporary, value.text));
        format!("!{}!", temporary)
    }
}

impl Backend for Batch {
    fn program(&mut self, body: String) -> String {
        let mut output = format!(
            "@echo off\nsetlocal enabledelayedexpansion\nset \"rosella_exit=\"\n{}",
            body
        );

        // Subroutines After Main Script
        if !self.subroutines.is_empty() {
            output.push_str("goto :eof\n\n");
            output.push_str(&self.subroutines);
        }

        // CRLF For Label Lookup
        output.replace('\n', "\r\n")
    }

    fn empty_body(&self) -> &'static str {
        "rem\n"
    }

    // No Call Stack
    fn supports_recursion(&self) -> bool {
        false
    }

    // No Local Variables
    fn local_name(&self, function: &str, parameter: &str) -> String {
        format!("rosella_{}_{}", function, parameter)
    }

    fn assign_int(&self, name: &str, value: &Arith) -> String {
        format!("set /a \"{}={}\"\n", name, value.text)
    }

    fn assign_str(&self, name: &str, value: &[Part]) -> Result<String, RosellaError> {
        Ok(format!("set \"{}={}\"\n", name, quoted(value)?))
    }

    fn condition(&mut self, test: Test, mut setup: String) -> Result<Condition, RosellaError> {
        match test {
            Test::Int {
                left,
                operator,
                right,
            } => {
                let left = self.int_operand(&left, &mut setup);
                let right = self.int_operand(&right, &mut setup);
                let operator = match operator {
                    BinaryOp::Equal => "EQU",
                    BinaryOp::NotEqual => "NEQ",
                    BinaryOp::LessThan => "LSS",
                    BinaryOp::LessThanEq => "LEQ",
                    BinaryOp::GreaterThan => "GTR",
                    _ => "GEQ",
                };
                Ok(Condition {
                    setup,
                    test: format!("{} {} {}", left, operator, right),
                })
            }
            Test::Str {
                left,
                operator,
                right,
            } => {
                let (left, right) = (quoted(&left)?, quoted(&right)?);
                let test = match operator {
                    BinaryOp::Equal => format!("\"{}\"==\"{}\"", left, right),
                    BinaryOp::NotEqual => format!("not \"{}\"==\"{}\"", left, right),
                    BinaryOp::LessThan => format!("\"{}\" LSS \"{}\"", left, right),
                    _ => format!("\"{}\" GTR \"{}\"", left, right),
                };
                Ok(Condition { setup, test })
            }
            Test::File { negate, path } => Ok(Condition {
                setup,
                test: format!(
                    "{}exist \"{}\"",
                    if negate { "not " } else { "" },
                    quoted(&path)?
                ),
            }),
        }
    }

    // Nest Else If
    fn if_chain(&self, branches: Vec<(Condition, String)>, otherwise: Option<String>) -> String {
        let mut tail = otherwise;
        let mut output = String::new();

        for (condition, body) in branches.into_iter().rev() {
            output = format!("{}if {} (\n{}", condition.setup, condition.test, body);
            if let Some(else_body) = tail {
                output.push_str(&format!(") else (\n{}", else_body));
            }
            output.push_str(")\n");
            tail = Some(indent(&output));
        }

        output
    }

    // Loops As Subroutines
    fn while_loop(&mut self, condition: Condition, body: String, check: ReturnCheck) -> String {
        let label = format!("rosella_while{}", self.next_index());

        self.subroutines.push_str(&format!(
            ":{label}\n{setup}if {test} goto :{label}_body\ngoto :eof\n:{label}_body\n{body}goto :{label}\n\n",
            label = label,
            setup = condition.setup,
            test = condition.test,
            body = body,
        ));

        let mut output = call(&label);
        output.push_str(match check {
            ReturnCheck::None => "",
            ReturnCheck::Propagate => "if defined rosella_returning goto :eof\n",
            ReturnCheck::Clear => {
                "if defined rosella_returning (set \"rosella_returning=\" & goto :eof)\n"
            }
        });
        output
    }

    // Pass Arguments Through Variables
    fn function(&mut self, name: &str, parameters: &[String], body: String) -> String {
        let mut output = format!(":{}\n", name);
        for (index, parameter) in parameters.iter().enumerate() {
            output.push_str(&indent(&format!(
                "set \"{}=!rosella_arg{}!\"\n",
                parameter,
                index + 1
            )));
        }
        output.push_str(&body);

        // Skip A Repeated Final Jump
        let jump = indent("goto :eof\n");
        if !body.ends_with(&jump) {
            output.push_str(&jump);
        }
        output.push('\n');

        self.subroutines.push_str(&output);
        String::new()
    }

    fn call(&self, name: &str, args: &[Arg]) -> Result<String, RosellaError> {
        let mut output = String::new();
        for (index, arg) in args.iter().enumerate() {
            output.push_str(&match arg {
                Arg::Int(value) => format!("set /a \"rosella_arg{}={}\"\n", index + 1, value.text),
                Arg::Value(parts) => {
                    format!("set \"rosella_arg{}={}\"\n", index + 1, quoted(parts)?)
                }
            });
        }
        output.push_str(&call(name));
        Ok(output)
    }

    fn capture(&self, temporary: &str, _local: bool) -> String {
        format!("set \"{}=!rosella_return!\"\n", temporary)
    }

    // Leave Every Enclosing Loop
    fn return_from_function(&self, nested_in_loop: bool) -> String {
        if nested_in_loop {
            "set \"rosella_returning=1\"\ngoto :eof\n".to_string()
        } else {
            "goto :eof\n".to_string()
        }
    }

    fn print(&self, text: &[Part]) -> Result<String, RosellaError> {
        Ok(split_lines(text)
            .iter()
            .map(|line| format!("echo({}\n", unquoted(line)))
            .collect())
    }

    fn cd(&self, path: &[Part]) -> Result<String, RosellaError> {
        Ok(format!("cd /d \"{}\"\n", quoted(path)?))
    }

    fn make_dir(&self, path: &[Part]) -> Result<String, RosellaError> {
        let path = quoted(path)?;
        Ok(format!("if not exist \"{}\" mkdir \"{}\"\n", path, path))
    }

    // Stop On Empty Variable
    fn remove(&self, path: &[Part], directory: bool, depth: usize) -> Result<String, RosellaError> {
        let mut output = String::new();
        for part in path {
            if let Part::Var(name) = part {
                output.push_str(&format!(
                    "if not defined {} (\n{}{})\n",
                    name,
                    indent(&format!(
                        ">&2 echo(Stopped: '{}' is empty in a path to remove\n",
                        name
                    )),
                    indent(&exit("1", depth)),
                ));
            }
        }

        let command = if directory {
            "rmdir /s /q"
        } else {
            "del /f /q"
        };
        let path = quoted(path)?;
        output.push_str(&format!("if exist \"{}\" {} \"{}\"\n", path, command, path));
        Ok(output)
    }

    fn transfer(
        &self,
        transfer: Transfer,
        source: &[Part],
        destination: &[Part],
    ) -> Result<String, RosellaError> {
        let command = match transfer {
            Transfer::Copy => "copy",
            Transfer::Move => "move",
        };
        Ok(format!(
            "{} /y \"{}\" \"{}\" >nul\n",
            command,
            quoted(source)?,
            quoted(destination)?
        ))
    }

    fn write(&self, path: &[Part], content: &[Part], append: bool) -> Result<String, RosellaError> {
        let path = quoted(path)?;
        let mut output = String::new();

        // Redirect First
        for (index, line) in split_lines(content).iter().enumerate() {
            let operator = if index == 0 && !append { ">" } else { ">>" };
            output.push_str(&format!(
                "{}\"{}\" echo({}\n",
                operator,
                path,
                unquoted(line)
            ));
        }

        Ok(output)
    }

    // Clear Before Prompt
    fn read(&self, prompt: &[Part], variable: &str, _local: bool) -> Result<String, RosellaError> {
        Ok(format!(
            "set \"{variable}=\"\nset /p \"{variable}={prompt}\"\n",
            variable = variable,
            prompt = quoted(prompt)?
        ))
    }

    fn exit(&self, code: &Arith, depth: usize) -> String {
        exit(&code.text, depth)
    }
}

// Helper Functions

fn call(label: &str) -> String {
    format!(
        "call :{}\nif defined rosella_exit exit /b !rosella_exit!\n",
        label
    )
}

// Exit Through Every Caller
fn exit(code: &str, depth: usize) -> String {
    match (depth, code.parse::<i32>()) {
        (0, Ok(code)) => format!("exit /b {}\n", code),
        (0, _) => format!("set /a \"rosella_exit={}\"\nexit /b !rosella_exit!\n", code),
        _ => format!("set /a \"rosella_exit={}\"\ngoto :eof\n", code),
    }
}

// Delayed Expansion Rescans Carets
fn has_bang(parts: &[Part]) -> bool {
    parts.iter().any(|part| match part {
        Part::Text(text) => text.contains('!'),
        Part::Var(_) | Part::Cwd => true,
    })
}

fn variable(part: &Part) -> String {
    match part {
        Part::Var(name) => format!("!{}!", name),
        _ => "!CD!".to_string(),
    }
}

fn quoted(parts: &[Part]) -> Result<String, RosellaError> {
    let bang = has_bang(parts);
    let mut output = String::new();

    for part in parts {
        let Part::Text(text) = part else {
            output.push_str(&variable(part));
            continue;
        };

        for ch in text.chars() {
            match ch {
                '"' => return Err(error("Batch cannot use '\"' inside a quoted value")),
                '\n' => {
                    return Err(error(
                        "Batch cannot keep a line break in a variable, argument or path",
                    ));
                }
                '%' => output.push_str("%%"),
                '!' => output.push_str("^!"),
                '^' if bang => output.push_str("^^"),
                _ => output.push(ch),
            }
        }
    }

    Ok(output)
}

fn unquoted(parts: &[Part]) -> String {
    let bang = has_bang(parts);
    let mut output = String::new();

    for part in parts {
        let Part::Text(text) = part else {
            output.push_str(&variable(part));
            continue;
        };

        for ch in text.chars() {
            match ch {
                '%' => output.push_str("%%"),
                '!' => output.push_str("^^!"),
                '^' if bang => output.push_str("^^^^"),
                '^' => output.push_str("^^"),
                '&' | '|' | '<' | '>' | '(' | ')' | '"' => {
                    output.push('^');
                    output.push(ch);
                }
                _ => output.push(ch),
            }
        }
    }

    output
}

fn split_lines(parts: &[Part]) -> Vec<Vec<Part>> {
    let mut lines: Vec<Vec<Part>> = vec![Vec::new()];

    for part in parts {
        match part {
            Part::Text(text) => {
                for (index, piece) in text.split('\n').enumerate() {
                    if index > 0 {
                        lines.push(Vec::new());
                    }
                    if !piece.is_empty() {
                        lines
                            .last_mut()
                            .unwrap()
                            .push(Part::Text(piece.to_string()));
                    }
                }
            }
            other => lines.last_mut().unwrap().push(other.clone()),
        }
    }

    lines
}
