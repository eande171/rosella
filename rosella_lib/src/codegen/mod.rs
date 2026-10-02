mod backend;
mod bash;
mod batch;
mod values;

use std::collections::{HashMap, HashSet};

use crate::builtins::{self, Builtin};
use crate::check::check;
use crate::error::RosellaError;
use crate::syntax::{BinaryOp, Expr, OS, Param, Stmt, Type};
use backend::{Arg, Backend, Condition, Test, Transfer};

const INDENT: &str = "    ";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shell {
    Batch,
    Bash,
}

pub struct Compiler {
    statements: Vec<Stmt>,
    os: OS,
    shell: Shell,
}

impl Compiler {
    pub fn new(statements: Vec<Stmt>, os: OS, shell: Shell) -> Self {
        Compiler {
            statements,
            os,
            shell,
        }
    }

    pub fn compile(&self) -> Result<String, RosellaError> {
        let mut statements = self.statements.clone();
        check(&mut statements, self.os)?;

        let backend: Box<dyn Backend> = match self.shell {
            Shell::Bash => Box::new(bash::Bash),
            Shell::Batch => Box::new(batch::Batch::default()),
        };

        Generator {
            os: self.os,
            backend,
            depth: 0,
            function: None,
        }
        .program(&statements)
    }
}

struct Generator {
    os: OS,
    backend: Box<dyn Backend>,
    depth: usize,
    function: Option<(String, HashSet<String>)>,
}

impl Generator {
    fn program(mut self, statements: &[Stmt]) -> Result<String, RosellaError> {
        if !self.backend.supports_recursion() {
            reject_recursion(statements, self.os)?;
        }

        let body = self.block(statements)?;
        Ok(self.backend.program(body))
    }

    // Statements

    fn block(&mut self, statements: &[Stmt]) -> Result<String, RosellaError> {
        let mut output = String::new();
        for statement in statements {
            output.push_str(&self.statement(statement)?);
        }
        Ok(output)
    }

    fn body(&mut self, statements: &[Stmt]) -> Result<String, RosellaError> {
        let body = self.block(statements)?;

        // Placeholder For Empty Blocks
        if body.is_empty() {
            return Ok(indent(self.backend.empty_body()));
        }

        Ok(indent(&body))
    }

    // Track Call Depth
    fn nested_body(&mut self, statements: &[Stmt]) -> Result<String, RosellaError> {
        self.depth += 1;
        let body = self.body(statements);
        self.depth -= 1;
        body
    }

    fn statement(&mut self, statement: &Stmt) -> Result<String, RosellaError> {
        match statement {
            Stmt::Let {
                variable_type,
                name,
                value,
            } => {
                let name = self.resolve(name);
                match variable_type {
                    Type::Int => Ok(self.backend.let_int(&name, &self.arith(value)?)),
                    Type::Str => self.backend.let_str(&name, &self.value_parts(value)?),
                    Type::File => Err(error("The file type cannot be stored in a variable")),
                }
            }
            Stmt::If { .. } => self.if_chain(statement),
            Stmt::With { os, body } if *os == self.os => self.block(body),
            Stmt::With { .. } => Ok(String::new()),
            Stmt::While {
                condition_type,
                condition,
                body,
            } => {
                let condition = self.condition(*condition_type, condition)?;
                let body = self.nested_body(body)?;
                Ok(self.backend.while_loop(condition, body))
            }
            Stmt::Function {
                name,
                parameters,
                body,
            } => self.function(name, parameters, body),
            Stmt::Expression(Expr::Call { name, args }) => self.call(name, args),
            Stmt::Expression(_) => Err(error("Only function calls can be used as statements")),
            Stmt::RawInstruction(text) => Ok(format!("{}\n", text)),
        }
    }

    // Flatten Else If Chains
    fn if_chain(&mut self, statement: &Stmt) -> Result<String, RosellaError> {
        let mut branches = Vec::new();
        let mut current = statement;

        loop {
            let Stmt::If {
                condition_type,
                condition,
                then_branch,
                else_branch,
            } = current
            else {
                return Err(error("Expected an if statement"));
            };

            let condition = self.condition(*condition_type, condition)?;
            let body = self.body(then_branch)?;
            branches.push((condition, body));

            match else_branch.as_deref() {
                Some([next @ Stmt::If { .. }]) => current = next,
                Some(otherwise) => {
                    let otherwise = self.body(otherwise)?;
                    return Ok(self.backend.if_chain(branches, Some(otherwise)));
                }
                None => return Ok(self.backend.if_chain(branches, None)),
            }
        }
    }

    fn function(
        &mut self,
        name: &str,
        parameters: &[Param],
        body: &[Stmt],
    ) -> Result<String, RosellaError> {
        let names = parameters
            .iter()
            .map(|parameter| parameter.name.clone())
            .collect();
        let outer = self.function.replace((name.to_string(), names));

        let local_names: Vec<String> = parameters
            .iter()
            .map(|parameter| self.resolve(&parameter.name))
            .collect();
        let body = self.nested_body(body);
        self.function = outer;

        Ok(self.backend.function(name, &local_names, body?))
    }

    fn condition(
        &mut self,
        condition_type: Option<Type>,
        condition: &Expr,
    ) -> Result<Condition, RosellaError> {
        let test = match (condition_type, condition) {
            (
                Some(Type::Int),
                Expr::Binary {
                    left,
                    operator,
                    right,
                },
            ) if operator.is_comparison() => Test::Int {
                left: self.arith(left)?,
                operator: *operator,
                right: self.arith(right)?,
            },
            (
                Some(Type::Str),
                Expr::Binary {
                    left,
                    operator:
                        operator @ (BinaryOp::Equal
                        | BinaryOp::NotEqual
                        | BinaryOp::LessThan
                        | BinaryOp::GreaterThan),
                    right,
                },
            ) => Test::Str {
                left: self.value_parts(left)?,
                operator: *operator,
                right: self.value_parts(right)?,
            },
            (Some(Type::File), Expr::Call { name, args })
                if name == "exists" || name == "not_exists" =>
            {
                Test::File {
                    negate: name == "not_exists",
                    path: self.path_parts(args)?,
                }
            }
            (Some(Type::Int), _) => {
                return Err(error("An int condition needs a comparison, like x < 10"));
            }
            (Some(Type::Str), _) => {
                return Err(error(
                    "A str condition needs ==, !=, < or >, like name == \"Bob\"",
                ));
            }
            (Some(Type::File), _) => {
                return Err(error(
                    "A file condition needs exists() or not_exists(), like exists(\"notes.txt\")",
                ));
            }
            (None, _) => {
                return Err(error(
                    "The condition type was not resolved before compiling",
                ));
            }
        };

        self.backend.condition(test)
    }

    // Function Calls

    fn call(&mut self, name: &str, args: &[Expr]) -> Result<String, RosellaError> {
        let Some(signature) = builtins::find(name) else {
            let args = args
                .iter()
                .map(|arg| match arg {
                    Expr::Binary { .. } => self.arith(arg).map(Arg::Int),
                    _ => self.value_parts(arg).map(Arg::Value),
                })
                .collect::<Result<Vec<Arg>, RosellaError>>()?;
            return self.backend.call(name, &args);
        };

        let backend = &self.backend;
        match signature.builtin {
            Builtin::Print => backend.print(&self.concat_parts(args)?),
            Builtin::Cd => backend.cd(&self.path_parts(args)?),
            Builtin::MakeDir => backend.make_dir(&self.path_parts(args)?),
            Builtin::Remove => backend.remove(&self.path_parts(args)?, false, self.depth),
            Builtin::RemoveDir => backend.remove(&self.path_parts(args)?, true, self.depth),
            Builtin::Copy | Builtin::Move => {
                let transfer = match signature.builtin {
                    Builtin::Copy => Transfer::Copy,
                    _ => Transfer::Move,
                };
                backend.transfer(
                    transfer,
                    &self.path_parts(&args[..1])?,
                    &self.path_parts(&args[1..])?,
                )
            }
            Builtin::WriteFile | Builtin::AppendFile => backend.write(
                &self.path_parts(&args[..1])?,
                &self.value_parts(&args[1])?,
                signature.builtin == Builtin::AppendFile,
            ),
            Builtin::Read => {
                let [prompt, Expr::Identifier(variable)] = args else {
                    return Err(error("read() needs a prompt and a variable name"));
                };
                backend.read(&self.value_parts(prompt)?, &self.resolve(variable))
            }
            Builtin::Exit => Ok(backend.exit(&self.arith(&args[0])?, self.depth)),
            Builtin::Path
            | Builtin::Concat
            | Builtin::GetCwd
            | Builtin::Exists
            | Builtin::NotExists => Err(error(format!("{}() cannot be used as a statement", name))),
        }
    }
}

// Recursion

fn reject_recursion(statements: &[Stmt], os: OS) -> Result<(), RosellaError> {
    let mut calls: HashMap<String, Vec<String>> = HashMap::new();
    collect_calls(statements, None, os, &mut calls);

    for start in calls.keys() {
        let mut visited = HashSet::new();
        let mut pending: Vec<&String> = calls[start].iter().collect();

        while let Some(callee) = pending.pop() {
            if callee == start {
                return Err(error(format!(
                    "Function '{}' calls itself, which Batch cannot support because its variables have no call stack",
                    start
                )));
            }
            if visited.insert(callee) {
                pending.extend(calls.get(callee).into_iter().flatten());
            }
        }
    }

    Ok(())
}

fn collect_calls(
    statements: &[Stmt],
    function: Option<&str>,
    os: OS,
    calls: &mut HashMap<String, Vec<String>>,
) {
    for statement in statements {
        match statement {
            Stmt::Function { name, body, .. } => {
                calls.entry(name.clone()).or_default();
                collect_calls(body, Some(name), os, calls);
            }
            Stmt::Expression(Expr::Call { name, .. }) if builtins::find(name).is_none() => {
                if let Some(function) = function {
                    calls
                        .entry(function.to_string())
                        .or_default()
                        .push(name.clone());
                }
            }
            Stmt::If {
                then_branch,
                else_branch,
                ..
            } => {
                collect_calls(then_branch, function, os, calls);
                if let Some(else_branch) = else_branch {
                    collect_calls(else_branch, function, os, calls);
                }
            }
            Stmt::While { body, .. } => collect_calls(body, function, os, calls),
            Stmt::With { os: with_os, body } if *with_os == os => {
                collect_calls(body, function, os, calls)
            }
            _ => {}
        }
    }
}

// Helper Functions

fn error(message: impl Into<String>) -> RosellaError {
    RosellaError::compiler(message)
}

fn indent(text: &str) -> String {
    text.lines()
        .map(|line| {
            if line.is_empty() {
                "\n".to_string()
            } else {
                format!("{}{}\n", INDENT, line)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::{Lexer, Parser};

    fn try_compile(input: &str, os: OS, shell: Shell) -> Result<String, RosellaError> {
        let ast = Parser::new(Lexer::new(input).tokenise()?).parse()?;
        Compiler::new(ast, os, shell).compile()
    }

    fn bash(input: &str) -> String {
        try_compile(input, OS::Linux, Shell::Bash).unwrap()
    }

    fn batch_error(input: &str) -> String {
        try_compile(input, OS::Windows, Shell::Batch)
            .unwrap_err()
            .to_string()
    }

    #[test]
    fn batch_output_uses_crlf() {
        let output = try_compile("print(1);", OS::Windows, Shell::Batch).unwrap();
        assert!(output.contains("echo(1\r\n"));
        assert_eq!(output.matches('\n').count(), output.matches("\r\n").count());
    }

    #[test]
    fn paths_join_with_the_target_separator() {
        assert!(bash("cd(\"a\", \"b\\\\c\");").contains("cd \"a/b/c\"\n"));
        assert!(bash("cd(\"/\", \"etc\");").contains("cd \"/etc\"\n"));
        let output = try_compile(
            "copy(path(\"C:\", \"a/b.txt\"), \"c.txt\");",
            OS::Windows,
            Shell::Batch,
        )
        .unwrap();
        assert!(output.contains("copy /y \"C:\\a\\b.txt\" \"c.txt\" >nul\r\n"));
    }

    #[test]
    fn recursion_is_rejected_for_batch_only() {
        let source = "fn a(int n) { if n > 0 { b(n - 1); } }\nfn b(int n) { a(n); }\na(3);";
        assert!(try_compile(source, OS::Linux, Shell::Bash).is_ok());
        assert!(batch_error(source).contains("calls itself"));
        assert!(
            try_compile(
                "fn a() { b(); }\nfn b() { }\na();",
                OS::Windows,
                Shell::Batch
            )
            .is_ok()
        );
    }

    #[test]
    fn shell_limits_are_reported() {
        assert!(batch_error("let str s = \"say \\\"hi\\\"\";").contains("'\"'"));
        assert!(batch_error("let str s = \"a\\nb\";").contains("line break"));
        assert!(batch_error("let int f = 1.5;").contains("whole number"));
        assert!(batch_error("let str s = \"x\";\nif s <= \"y\" { }").contains("str condition"));
    }
}
