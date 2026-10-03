mod backend;
mod bash;
mod batch;
mod values;

use std::collections::{HashMap, HashSet};

use crate::builtins::{self, Builtin};
use crate::check::check;
use crate::error::{RosellaError, Span};
use crate::syntax::{Condition, Expr, Iteration, OS, Param, Statement, Stmt, Type};
use backend::{
    Arg, Backend, FileCheck, Logic, LoopKind, LoopLabel, Part, ReturnCheck, Test, Transfer,
};

const INDENT: &str = "    ";
const RETURN_VARIABLE: &str = "rosella_return";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shell {
    Batch,
    Bash,
}

pub struct Compiler {
    statements: Vec<Statement>,
    os: OS,
    shell: Shell,
}

impl Compiler {
    pub fn new(statements: Vec<Statement>, os: OS, shell: Shell) -> Self {
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
            Shell::Bash => Box::new(bash::Bash::default()),
            Shell::Batch => Box::new(batch::Batch::default()),
        };

        Generator {
            os: self.os,
            backend,
            depth: 0,
            loop_labels: Vec::new(),
            results: 0,
            function: None,
            exiting: HashSet::new(),
            share_return: false,
        }
        .program(&statements)
    }
}

struct Scope {
    name: String,
    parameters: HashSet<String>,
    returns: Option<Type>,
}

struct Generator {
    os: OS,
    backend: Box<dyn Backend>,
    depth: usize,
    loop_labels: Vec<LoopLabel>,
    results: usize,
    function: Option<Scope>,
    exiting: HashSet<String>,
    share_return: bool,
}

impl Generator {
    fn program(mut self, statements: &[Statement]) -> Result<String, RosellaError> {
        if !self.backend.supports_recursion() {
            reject_recursion(statements, self.os)?;
        }
        self.exiting = exiting_functions(statements, self.os);

        let mut names = Vec::new();
        for statement in statements {
            statement_calls(&statement.kind, &mut names);
        }
        if names.iter().any(|name| name == "script_dir") {
            self.backend.use_script_dir();
        }

        let body = self.block(statements)?;
        Ok(self.backend.program(body))
    }

    // Statements

    fn block(&mut self, statements: &[Statement]) -> Result<String, RosellaError> {
        let mut output = String::new();
        for statement in statements {
            let compiled = self
                .statement(&statement.kind)
                .map_err(|error| error.located(statement.span))?;
            output.push_str(&compiled);
        }
        Ok(output)
    }

    fn body(&mut self, statements: &[Statement]) -> Result<String, RosellaError> {
        let body = self.block(statements)?;

        if body.is_empty() {
            return Ok(indent(self.backend.empty_body()));
        }

        Ok(indent(&body))
    }

    // Track Call Depth
    fn nested_body(&mut self, statements: &[Statement]) -> Result<String, RosellaError> {
        self.depth += 1;
        let body = self.body(statements);
        self.depth -= 1;
        body
    }

    fn statement(&mut self, statement: &Stmt) -> Result<String, RosellaError> {
        self.share_return = match statement {
            Stmt::Let { value, .. }
            | Stmt::Assign { value, .. }
            | Stmt::Return(Some(value))
            | Stmt::Expression(value) => match value {
                Expr::Call { args, .. } => args.iter().map(count_calls).sum::<usize>() == 1,
                other => count_calls(other) == 1,
            },
            _ => false,
        };

        match statement {
            Stmt::Let {
                variable_type,
                name,
                value,
            }
            | Stmt::Assign {
                variable_type: Some(variable_type),
                name,
                value,
            } => {
                let name = self.resolve(name);
                self.assignment(*variable_type, &name, value)
            }
            Stmt::Assign { .. } => Err(error(
                "The assignment type was not resolved before compiling",
            )),
            Stmt::If { .. } => self.if_chain(statement),
            Stmt::With { os, body } if *os == self.os => self.block(body),
            Stmt::With { .. } => Ok(String::new()),
            Stmt::While {
                condition: test,
                resolved,
                body,
            } => {
                let mut names = Vec::new();
                all_calls(test, &mut names);
                let exits = self.can_exit(&names) || self.body_exits(body);
                let label = self.backend.begin_loop(LoopKind::While, exits);
                let condition = self.condition(resolved)?;
                let check = self.return_check(body);
                let body = self.loop_body(&label, body)?;
                Ok(self.backend.while_loop(&label, condition, body, check))
            }
            Stmt::For {
                name,
                resolved,
                body,
                ..
            } => self.for_loop(name, resolved, body),
            Stmt::Break => match self.loop_labels.last() {
                Some(label) => Ok(self.backend.break_loop(label)),
                None => Err(error("break can only be used inside a loop")),
            },
            Stmt::Continue => match self.loop_labels.last() {
                Some(label) => Ok(self.backend.continue_loop(label)),
                None => Err(error("continue can only be used inside a loop")),
            },
            Stmt::Function {
                name,
                return_type,
                parameters,
                body,
            } => self.function(name, *return_type, parameters, body),
            Stmt::Return(value) => self.return_statement(value.as_ref()),
            Stmt::Expression(Expr::Call { name, args }) => self.call(name, args),
            Stmt::Expression(_) => Err(error("Only function calls can be used as statements")),
            Stmt::RawInstruction(text) => Ok(format!("{}\n", text)),
        }
    }

    fn return_check(&self, body: &[Statement]) -> ReturnCheck {
        match (&self.function, contains_return(body, self.os)) {
            (Some(_), true) if self.loop_labels.is_empty() => ReturnCheck::Clear,
            (Some(_), true) => ReturnCheck::Propagate,
            _ => ReturnCheck::None,
        }
    }

    fn loop_body(&mut self, label: &LoopLabel, body: &[Statement]) -> Result<String, RosellaError> {
        self.loop_labels.push(label.clone());
        let body = self.nested_body(body);
        self.loop_labels.pop();
        body
    }

    fn for_loop(
        &mut self,
        name: &str,
        resolved: &Option<Iteration>,
        body: &[Statement],
    ) -> Result<String, RosellaError> {
        let Some(iteration) = resolved else {
            return Err(error("The for loop was not resolved before compiling"));
        };
        let variable = self.resolve(name);
        let check = self.return_check(body);
        let exits = self.body_exits(body);
        let mut output = String::new();

        match iteration {
            Iteration::Range { start, end, step } => {
                let label = self.backend.begin_loop(LoopKind::Range, exits);
                self.share_return = count_calls(start) + count_calls(end) == 1;
                let start = self.hoist(start, &mut output)?;
                let start = self.arith(&start)?;

                // Fixed End
                let end = self.hoist(end, &mut output)?;
                let end = match end {
                    Expr::Number(_) => self.arith(&end)?,
                    // Fresh Result
                    Expr::Identifier(ref result) if result.starts_with("rosella_result") => {
                        self.arith(&end)?
                    }
                    _ => {
                        let result = self.next_result();
                        if self.function.is_some() {
                            output.push_str(&self.backend.declare_local(&result));
                        }
                        output.push_str(&self.backend.assign_int(&result, &self.arith(&end)?));
                        self.arith(&Expr::Identifier(result))?
                    }
                };

                let body = self.loop_body(&label, body);
                output.push_str(
                    &self
                        .backend
                        .range_loop(&label, &variable, &start, &end, *step, body?, check),
                );
            }
            Iteration::Files { pattern } => {
                let label = self.backend.begin_loop(LoopKind::Files, exits);
                self.share_return = pattern.iter().map(count_calls).sum::<usize>() == 1;
                let mut converted = Vec::new();
                for part in pattern {
                    let part = self.hoist(part, &mut output)?;
                    converted.push(self.stringify(&part, &mut output)?);
                }
                let pattern = self.path_parts(&converted)?;

                let body = self.loop_body(&label, body)?;
                output.push_str(
                    &self
                        .backend
                        .files_loop(&label, &variable, &pattern, body, check)?,
                );
            }
        }

        Ok(output)
    }

    fn assignment(
        &mut self,
        variable_type: Type,
        name: &str,
        value: &Expr,
    ) -> Result<String, RosellaError> {
        let mut output = String::new();

        let value = match value {
            // Direct Result
            Expr::Call { name: call, args } if is_produced(call) => {
                let lines = self.produce(call, args, name, false, &mut output)?;
                output.push_str(&lines);
                return Ok(output);
            }
            // Returned Value
            Expr::Call { name: call, args } if is_user_function(call) => {
                let call = self.user_call(call, args, &mut output)?;
                output.push_str(&call);
                Expr::Identifier(RETURN_VARIABLE.to_string())
            }
            _ => self.hoist(value, &mut output)?,
        };

        let assignment = match variable_type {
            Type::Int => self.backend.assign_int(name, &self.arith(&value)?),
            Type::Str => {
                let value = self.stringify(&value, &mut output)?;
                self.backend.assign_str(name, &self.value_parts(&value)?)?
            }
            Type::Check => return Err(error("The file type cannot be stored in a variable")),
        };
        output.push_str(&assignment);
        Ok(output)
    }

    fn return_statement(&mut self, value: Option<&Expr>) -> Result<String, RosellaError> {
        let mut output = String::new();

        if let Some(value) = value {
            let returns = self.function.as_ref().and_then(|scope| scope.returns);
            match (value, returns) {
                // Value Already Set
                (Expr::Call { name, args }, _) if is_user_function(name) => {
                    let call = self.user_call(name, args, &mut output)?;
                    output.push_str(&call);
                }
                (_, Some(returns)) => {
                    output.push_str(&self.assignment(returns, RETURN_VARIABLE, value)?)
                }
                (_, None) => return Err(error("This function has no return type")),
            }
        }

        output.push_str(
            &self
                .backend
                .return_from_function(!self.loop_labels.is_empty()),
        );
        Ok(output)
    }

    fn if_chain(&mut self, statement: &Stmt) -> Result<String, RosellaError> {
        let mut branches = Vec::new();
        let mut current = statement;
        let mut span = None;

        loop {
            let Stmt::If {
                resolved,
                then_branch,
                else_branch,
                ..
            } = current
            else {
                return Err(error("Expected an if statement"));
            };

            // Branch Error Line
            let condition = self.condition(resolved).map_err(|error| match span {
                Some(span) => error.located(span),
                None => error,
            })?;
            let body = self.body(then_branch)?;
            branches.push((condition, body));

            match else_branch.as_deref() {
                Some([next]) if matches!(next.kind, Stmt::If { .. }) => {
                    current = &next.kind;
                    span = Some(next.span);
                }
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
        returns: Option<Type>,
        parameters: &[Param],
        body: &[Statement],
    ) -> Result<String, RosellaError> {
        let scope = Scope {
            name: name.to_string(),
            parameters: parameters
                .iter()
                .map(|parameter| parameter.name.clone())
                .collect(),
            returns,
        };
        let outer_scope = self.function.replace(scope);
        let outer_loops = std::mem::take(&mut self.loop_labels);

        let local_names: Vec<String> = parameters
            .iter()
            .map(|parameter| self.resolve(&parameter.name))
            .collect();
        let body = self.nested_body(body);

        self.function = outer_scope;
        self.loop_labels = outer_loops;

        Ok(self.backend.function(name, &local_names, body?))
    }

    fn condition(
        &mut self,
        resolved: &Option<Condition>,
    ) -> Result<backend::Condition, RosellaError> {
        let Some(resolved) = resolved else {
            return Err(error("The condition was not resolved before compiling"));
        };
        let logic = self.logic(resolved)?;
        self.backend.condition(logic)
    }

    // Test Setup
    fn logic(&mut self, condition: &Condition) -> Result<Logic, RosellaError> {
        let mut setup = String::new();

        let test = match condition {
            Condition::Not(inner) => return Ok(Logic::Not(Box::new(self.logic(inner)?))),
            Condition::And(left, right) => {
                return Ok(Logic::And(
                    Box::new(self.logic(left)?),
                    Box::new(self.logic(right)?),
                ));
            }
            Condition::Or(left, right) => {
                return Ok(Logic::Or(
                    Box::new(self.logic(left)?),
                    Box::new(self.logic(right)?),
                ));
            }
            Condition::Compare {
                value_type: Type::Str,
                left,
                operator,
                right,
            } => {
                self.share_return = count_calls(left) + count_calls(right) == 1;
                let left = self.hoist(left, &mut setup)?;
                let left = self.stringify(&left, &mut setup)?;
                let right = self.hoist(right, &mut setup)?;
                let right = self.stringify(&right, &mut setup)?;
                Test::Str {
                    left: self.value_parts(&left)?,
                    operator: *operator,
                    right: self.value_parts(&right)?,
                }
            }
            Condition::Compare {
                left,
                operator,
                right,
                ..
            } => {
                self.share_return = count_calls(left) + count_calls(right) == 1;
                let left = self.hoist(left, &mut setup)?;
                let right = self.hoist(right, &mut setup)?;
                Test::Int {
                    left: self.arith(&left)?,
                    operator: *operator,
                    right: self.arith(&right)?,
                }
            }
            Condition::Check { check, args } => {
                self.share_return = args.iter().map(count_calls).sum::<usize>() == 1;
                let mut converted = Vec::new();
                for arg in args {
                    let arg = self.hoist(arg, &mut setup)?;
                    converted.push(self.stringify(&arg, &mut setup)?);
                }

                let file_check = match check.as_str() {
                    "contains" => {
                        return Ok(Logic::Test {
                            test: Test::Contains {
                                text: self.value_parts(&converted[0])?,
                                part: self.value_parts(&converted[1])?,
                            },
                            setup,
                        });
                    }
                    "not_exists" => FileCheck::Missing,
                    "is_dir" => FileCheck::Directory,
                    "is_file" => FileCheck::File,
                    _ => FileCheck::Exists,
                };
                Test::File {
                    check: file_check,
                    path: self.path_parts(&converted)?,
                }
            }
        };

        Ok(Logic::Test { setup, test })
    }

    // Function Calls

    fn call(&mut self, name: &str, args: &[Expr]) -> Result<String, RosellaError> {
        let mut output = String::new();

        let Some(signature) = builtins::find(name) else {
            let call = self.user_call(name, args, &mut output)?;
            output.push_str(&call);
            return Ok(output);
        };

        if signature.builtin == Builtin::Read {
            // Discard The Input
            let lines = self.produce(
                name,
                args,
                "rosella_input",
                self.function.is_some(),
                &mut output,
            )?;
            output.push_str(&lines);
            return Ok(output);
        }

        let mut hoisted = Vec::new();
        for arg in args {
            let arg = self.hoist(arg, &mut output)?;
            hoisted.push(match signature.builtin {
                Builtin::Exit | Builtin::Sleep => arg,
                _ => self.stringify(&arg, &mut output)?,
            });
        }
        let args = hoisted.as_slice();

        if signature.builtin == Builtin::Run {
            let command = self.command(args)?;
            output.push_str(&self.backend.run(&command)?);
            return Ok(output);
        }

        let backend = &self.backend;
        let statement = match signature.builtin {
            Builtin::Print => backend.print(&self.concat_parts(args)?),
            Builtin::Cd => backend.cd(&self.path_parts(args)?),
            Builtin::MakeDir => backend.make_dir(&self.path_parts(args)?),
            Builtin::Remove => backend.remove(&self.path_parts(args)?, false),
            Builtin::RemoveDir => backend.remove(&self.path_parts(args)?, true),
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
            Builtin::Exit => Ok(backend.exit(&self.arith(&args[0])?, self.depth)),
            Builtin::Sleep => Ok(backend.sleep(&self.arith(&args[0])?)),
            Builtin::SetEnv => match &args[0] {
                Expr::String(variable) => backend.set_env(variable, &self.value_parts(&args[1])?),
                _ => Err(error("set_env() needs the variable name written in quotes")),
            },
            Builtin::Read
            | Builtin::Run
            | Builtin::RunOutput
            | Builtin::Arg
            | Builtin::ArgCount
            | Builtin::Env
            | Builtin::Path
            | Builtin::Concat
            | Builtin::GetCwd
            | Builtin::Exists
            | Builtin::NotExists
            | Builtin::IsDir
            | Builtin::IsFile
            | Builtin::Contains
            | Builtin::Length
            | Builtin::Slice
            | Builtin::Replace
            | Builtin::Upper
            | Builtin::Lower
            | Builtin::Random
            | Builtin::ScriptDir => Err(error(format!("{}() cannot be used as a statement", name))),
        };

        output.push_str(&statement?);
        Ok(output)
    }

    fn can_exit(&self, names: &[String]) -> bool {
        names
            .iter()
            .any(|name| stops_script(name) || self.exiting.contains(name))
    }

    fn body_exits(&self, body: &[Statement]) -> bool {
        let mut names = Vec::new();
        run_calls(body, self.os, &mut names);
        self.can_exit(&names)
    }

    fn user_call(
        &mut self,
        name: &str,
        args: &[Expr],
        setup: &mut String,
    ) -> Result<String, RosellaError> {
        let mut values = Vec::new();
        for arg in args {
            let arg = self.hoist(arg, setup)?;
            values.push(match &arg {
                Expr::Binary { .. } => Arg::Int(self.arith(&arg)?),
                _ => {
                    let arg = self.stringify(&arg, setup)?;
                    Arg::Value(self.value_parts(&arg)?)
                }
            });
        }
        self.backend
            .call(name, &values, self.exiting.contains(name))
    }

    fn produce(
        &mut self,
        name: &str,
        args: &[Expr],
        target: &str,
        local: bool,
        setup: &mut String,
    ) -> Result<String, RosellaError> {
        let mut hoisted = Vec::new();
        for arg in args {
            hoisted.push(self.hoist(arg, setup)?);
        }

        let builtin = builtins::find(name).map(|signature| signature.builtin);
        match builtin {
            Some(Builtin::Read) => {
                let prompt = self.stringify(&hoisted[0], setup)?;
                self.backend
                    .read(&self.value_parts(&prompt)?, target, local)
            }
            Some(Builtin::Arg) => {
                Ok(self
                    .backend
                    .argument(&self.arith(&hoisted[0])?, target, local))
            }
            Some(Builtin::ArgCount) => Ok(self.backend.argument_count(target, local)),
            Some(Builtin::Run) => {
                let mut converted = Vec::new();
                for arg in &hoisted {
                    converted.push(self.stringify(arg, setup)?);
                }
                let command = self.command(&converted)?;
                let mut lines = self.backend.run(&command)?;
                lines.push_str(&self.backend.capture_status(target, local));
                Ok(lines)
            }
            Some(Builtin::Length) => {
                let text = self.stringify(&hoisted[0], setup)?;
                self.backend
                    .length(&self.value_parts(&text)?, target, local)
            }
            Some(Builtin::Slice) => {
                let text = self.stringify(&hoisted[0], setup)?;
                let text = self.value_parts(&text)?;
                self.backend.slice(
                    &text,
                    &self.arith(&hoisted[1])?,
                    &self.arith(&hoisted[2])?,
                    target,
                    local,
                )
            }
            Some(Builtin::Replace) => {
                let mut texts = Vec::new();
                for arg in &hoisted {
                    let arg = self.stringify(arg, setup)?;
                    texts.push(self.value_parts(&arg)?);
                }
                self.backend
                    .replace(&texts[0], &texts[1], &texts[2], target, local)
            }
            Some(builtin @ (Builtin::Upper | Builtin::Lower)) => {
                let text = self.stringify(&hoisted[0], setup)?;
                self.backend.change_case(
                    &self.value_parts(&text)?,
                    builtin == Builtin::Upper,
                    target,
                    local,
                )
            }
            Some(Builtin::Random) => {
                let min = self.arith(&hoisted[0])?;
                let max = self.arith(&hoisted[1])?;
                Ok(self.backend.random(&min, &max, target, local))
            }
            Some(Builtin::RunOutput) => {
                let mut converted = Vec::new();
                for arg in &hoisted {
                    converted.push(self.stringify(arg, setup)?);
                }
                let command = self.command(&converted)?;
                self.backend.run_output(&command, target, local)
            }
            _ => Err(error(format!(
                "{}() cannot be produced into a variable",
                name
            ))),
        }
    }

    fn command(&self, args: &[Expr]) -> Result<Vec<Vec<Part>>, RosellaError> {
        args.iter().map(|arg| self.value_parts(arg)).collect()
    }
}

// Recursion

fn reject_recursion(statements: &[Statement], os: OS) -> Result<(), RosellaError> {
    let mut calls: HashMap<String, Vec<String>> = HashMap::new();
    let mut definitions: HashMap<String, Span> = HashMap::new();
    collect_calls(statements, None, os, &mut calls, &mut definitions);

    for start in calls.keys() {
        let mut visited = HashSet::new();
        let mut pending: Vec<&String> = calls[start].iter().collect();

        while let Some(callee) = pending.pop() {
            if callee == start {
                return Err(error(format!(
                    "Function '{}' calls itself, which Batch cannot support because its variables have no call stack",
                    start
                ))
                .located(definitions[start]));
            }
            if visited.insert(callee) {
                pending.extend(calls.get(callee).into_iter().flatten());
            }
        }
    }

    Ok(())
}

fn collect_calls(
    statements: &[Statement],
    function: Option<&str>,
    os: OS,
    calls: &mut HashMap<String, Vec<String>>,
    definitions: &mut HashMap<String, Span>,
) {
    for statement in statements {
        let mut found = Vec::new();

        match &statement.kind {
            Stmt::Function { name, body, .. } => {
                calls.entry(name.clone()).or_default();
                definitions.insert(name.clone(), statement.span);
                collect_calls(body, Some(name), os, calls, definitions);
            }
            Stmt::Let { value, .. }
            | Stmt::Assign { value, .. }
            | Stmt::Return(Some(value))
            | Stmt::Expression(value) => expression_calls(value, &mut found),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                expression_calls(condition, &mut found);
                collect_calls(then_branch, function, os, calls, definitions);
                if let Some(else_branch) = else_branch {
                    collect_calls(else_branch, function, os, calls, definitions);
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                expression_calls(condition, &mut found);
                collect_calls(body, function, os, calls, definitions);
            }
            Stmt::For { iterable, body, .. } => {
                expression_calls(iterable, &mut found);
                collect_calls(body, function, os, calls, definitions);
            }
            Stmt::With { os: with_os, body } if *with_os == os => {
                collect_calls(body, function, os, calls, definitions)
            }
            _ => {}
        }

        if let Some(function) = function {
            calls.entry(function.to_string()).or_default().extend(found);
        }
    }
}

// Stopping The Script

fn exiting_functions(statements: &[Statement], os: OS) -> HashSet<String> {
    let mut defined = Vec::new();
    definitions(statements, os, &mut defined);
    let calls: Vec<(&str, Vec<String>)> = defined
        .into_iter()
        .map(|(name, body)| {
            let mut names = Vec::new();
            run_calls(body, os, &mut names);
            (name, names)
        })
        .collect();

    // Find Callers
    let mut exiting = HashSet::new();
    loop {
        let before = exiting.len();
        for (name, names) in &calls {
            if names
                .iter()
                .any(|callee| stops_script(callee) || exiting.contains(callee))
            {
                exiting.insert(name.to_string());
            }
        }
        if exiting.len() == before {
            return exiting;
        }
    }
}

fn definitions<'a>(
    statements: &'a [Statement],
    os: OS,
    found: &mut Vec<(&'a str, &'a [Statement])>,
) {
    for statement in statements {
        match &statement.kind {
            Stmt::Function { name, body, .. } => {
                found.push((name, body));
                definitions(body, os, found);
            }
            Stmt::If {
                then_branch,
                else_branch,
                ..
            } => {
                definitions(then_branch, os, found);
                if let Some(else_branch) = else_branch {
                    definitions(else_branch, os, found);
                }
            }
            Stmt::While { body, .. } | Stmt::For { body, .. } => definitions(body, os, found),
            Stmt::With { os: with_os, body } if *with_os == os => definitions(body, os, found),
            _ => {}
        }
    }
}

// Runtime Calls
fn run_calls(statements: &[Statement], os: OS, names: &mut Vec<String>) {
    for statement in statements {
        match &statement.kind {
            Stmt::Let { value, .. }
            | Stmt::Assign { value, .. }
            | Stmt::Return(Some(value))
            | Stmt::Expression(value) => all_calls(value, names),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                all_calls(condition, names);
                run_calls(then_branch, os, names);
                if let Some(else_branch) = else_branch {
                    run_calls(else_branch, os, names);
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                all_calls(condition, names);
                run_calls(body, os, names);
            }
            Stmt::For { iterable, body, .. } => {
                all_calls(iterable, names);
                run_calls(body, os, names);
            }
            Stmt::With { os: with_os, body } if *with_os == os => run_calls(body, os, names),
            _ => {}
        }
    }
}

// Stopping Built-ins
fn stops_script(name: &str) -> bool {
    builtins::find(name).is_some_and(|signature| {
        matches!(
            signature.builtin,
            Builtin::Exit | Builtin::Remove | Builtin::RemoveDir
        )
    })
}

fn count_calls(expr: &Expr) -> usize {
    match expr {
        Expr::Binary { left, right, .. } => count_calls(left) + count_calls(right),
        Expr::Not(inner) => count_calls(inner),
        Expr::Call { name, args } => {
            let own = usize::from(is_user_function(name));
            own + args.iter().map(count_calls).sum::<usize>()
        }
        _ => 0,
    }
}

// All Calls
fn statement_calls(statement: &Stmt, names: &mut Vec<String>) {
    let body_calls = |body: &[Statement], names: &mut Vec<String>| {
        for inner in body {
            statement_calls(&inner.kind, names);
        }
    };

    match statement {
        Stmt::Let { value, .. }
        | Stmt::Assign { value, .. }
        | Stmt::Return(Some(value))
        | Stmt::Expression(value) => all_calls(value, names),
        Stmt::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            all_calls(condition, names);
            body_calls(then_branch, names);
            if let Some(else_branch) = else_branch {
                body_calls(else_branch, names);
            }
        }
        Stmt::While {
            condition, body, ..
        } => {
            all_calls(condition, names);
            body_calls(body, names);
        }
        Stmt::For { iterable, body, .. } => {
            all_calls(iterable, names);
            body_calls(body, names);
        }
        Stmt::Function { body, .. } | Stmt::With { body, .. } => body_calls(body, names),
        _ => {}
    }
}

fn all_calls(expr: &Expr, names: &mut Vec<String>) {
    match expr {
        Expr::Binary { left, right, .. } => {
            all_calls(left, names);
            all_calls(right, names);
        }
        Expr::Not(inner) => all_calls(inner, names),
        Expr::Call { name, args } => {
            names.push(name.clone());
            for arg in args {
                all_calls(arg, names);
            }
        }
        _ => {}
    }
}

fn expression_calls(expr: &Expr, found: &mut Vec<String>) {
    match expr {
        Expr::Binary { left, right, .. } => {
            expression_calls(left, found);
            expression_calls(right, found);
        }
        Expr::Not(inner) => expression_calls(inner, found),
        Expr::Call { name, args } => {
            if is_user_function(name) {
                found.push(name.clone());
            }
            for arg in args {
                expression_calls(arg, found);
            }
        }
        _ => {}
    }
}

// Helper Functions

fn error(message: impl Into<String>) -> RosellaError {
    RosellaError::compiler(message)
}

fn is_produced(name: &str) -> bool {
    builtins::find(name).is_some_and(|signature| {
        matches!(
            signature.builtin,
            Builtin::Read
                | Builtin::Run
                | Builtin::RunOutput
                | Builtin::Arg
                | Builtin::ArgCount
                | Builtin::Length
                | Builtin::Slice
                | Builtin::Replace
                | Builtin::Upper
                | Builtin::Lower
                | Builtin::Random
        )
    })
}

fn is_user_function(name: &str) -> bool {
    builtins::find(name).is_none()
}

fn contains_return(statements: &[Statement], os: OS) -> bool {
    statements.iter().any(|statement| match &statement.kind {
        Stmt::Return(_) => true,
        Stmt::If {
            then_branch,
            else_branch,
            ..
        } => {
            contains_return(then_branch, os)
                || else_branch
                    .as_deref()
                    .is_some_and(|otherwise| contains_return(otherwise, os))
        }
        Stmt::While { body, .. } | Stmt::For { body, .. } => contains_return(body, os),
        Stmt::With { os: with_os, body } if *with_os == os => contains_return(body, os),
        _ => false,
    })
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

        let returning = "fn int fib(int n) { if n < 2 { return n; } return fib(n - 1) + fib(n - 2); }\nlet int f = fib(10);";
        assert!(try_compile(returning, OS::Linux, Shell::Bash).is_ok());
        assert!(batch_error(returning).contains("'fib' calls itself"));

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
    fn recursion_inside_not_is_rejected() {
        let source = "fn int down(int n) {\n    if !(down(n - 1) > 100) { return n; }\n    return n;\n}\nprint(down(3));";
        assert!(batch_error(source).contains("'down' calls itself"));
    }

    #[test]
    fn bash_functions_are_never_empty() {
        assert!(bash("fn nothing() { return; }\nnothing();").contains("nothing() {\n    :\n}\n"));
    }

    #[test]
    fn bash_parameters_are_hidden_from_called_functions() {
        let output = bash("fn f(int x) { print(x); }\nf(1);");
        assert!(output.contains("    local rosella_f_x=\"${1}\"\n"));
        assert!(output.contains("\"${rosella_f_x}\""));
    }

    #[test]
    fn batch_is_file_keeps_its_else() {
        let output = try_compile(
            "if is_file(\"a\") { print(1); } else { print(2); }",
            OS::Windows,
            Shell::Batch,
        )
        .unwrap();
        assert!(
            output.contains("if exist \"a\" if not exist \"a\\\" set \"rosella_condition0=1\"")
        );
        assert!(output.contains("if defined rosella_condition0 ("));
    }

    #[test]
    fn batch_skips_wildcards_held_in_variables() {
        let source = "let str p = \"x\";\nremove(p);\nremove(\"y\");";
        assert!(!bash(source).contains("for"));
        let output = try_compile(source, OS::Windows, Shell::Batch).unwrap();
        assert!(output.contains(
            "set \"rosella_wild=\" & for /f \"tokens=2 delims=*?\" %%w in (\"x!p!x\") do set \"rosella_wild=1\"\r\nif not defined rosella_wild if exist \"!p!\" if not exist \"!p!\\\" del /f /q \"!p!\"\r\n"
        ));
        assert!(output.contains("\r\nif exist \"y\" if not exist \"y\\\" del /f /q \"y\"\r\n"));
    }

    #[test]
    fn bash_results_are_local_inside_functions() {
        let output = bash(
            "fn int one() { return 1; }\nfn int two() { return one() + one(); }\nlet int x = one() + 1;",
        );
        assert!(output.contains("    local rosella_result0=\"${rosella_return}\"\n"));
        assert!(output.contains("\nx=$(( rosella_return + 1 ))\n"));
    }

    #[test]
    fn batch_passes_exits_along_only_when_needed() {
        let source = "fn quiet() { print(1); }\nfn loud() { exit(2); }\nfn outer() { loud(); }\nquiet();\nouter();";
        let output = try_compile(source, OS::Windows, Shell::Batch).unwrap();
        assert!(output.contains("call :quiet\r\ncall :outer\r\nif defined rosella_exit"));
    }

    #[test]
    fn batch_compares_variables_directly() {
        let output = try_compile(
            "let int x = 1;\nif x == 1 { }\nif !(x == 2) { }\nfn f(int n) { if n > x { } }\nif x + 1 == 2 { }",
            OS::Windows,
            Shell::Batch,
        )
        .unwrap();
        assert!(output.contains("if !x! EQU 1 ("));
        assert!(output.contains("if not !x! EQU 2 ("));
        assert!(output.contains("if !rosella_f.n! GTR !x! ("));
        assert!(output.contains("set /a \"rosella_temp0=x + 1\"\r\nif !rosella_temp0! EQU 2 ("));
    }

    #[test]
    fn batch_keeps_its_variables_out_of_the_window() {
        let output = try_compile("print(arg(1));", OS::Windows, Shell::Batch).unwrap();
        assert!(output.starts_with("@echo off\r\nsetlocal\r\nset \"rosella_argc=0\"\r\n"));
        let output = try_compile("print(1);", OS::Windows, Shell::Batch).unwrap();
        assert!(output.starts_with("@echo off\r\nsetlocal enabledelayedexpansion\r\n"));
        assert!(output.ends_with("exit /b 0\r\n"));
        assert!(bash("print(1);").ends_with("exit 0\n"));
    }

    #[test]
    fn remove_only_touches_its_own_kind() {
        let output = try_compile(
            "remove(\"a\");\nremove_dir(\"b\");",
            OS::Windows,
            Shell::Batch,
        )
        .unwrap();
        assert!(output.contains("if exist \"a\" if not exist \"a\\\" del /f /q \"a\"\r\n"));
        assert!(output.contains("if exist \"b\\\" rmdir /s /q \"b\"\r\n"));
        let output = bash("remove(\"a\");\nremove_dir(\"b\");");
        assert!(output.contains("if [[ ! -d \"a\" ]]; then rm -f -- \"a\"; fi\n"));
        assert!(output.contains("if [[ -d \"b\" ]]; then rm -rf -- \"b\"; fi\n"));
    }

    #[test]
    fn errors_point_at_their_statement() {
        let message = batch_error(
            "print(1);
fn int a(int n) { return a(n); }",
        );
        assert!(
            message.starts_with("line 2, column 1: Function 'a' calls itself"),
            "{}",
            message
        );
        let message = batch_error(
            "print(1);

    let str s = \"a\nb\";",
        );
        assert!(
            message.starts_with("line 3, column 5: Batch cannot keep a line break"),
            "{}",
            message
        );
    }

    #[test]
    fn shell_limits_are_reported() {
        assert!(batch_error("let str s = \"say \\\"hi\\\"\";").contains("'\"'"));
        assert!(batch_error("let str s = \"a\\nb\";").contains("line break"));
        assert!(batch_error("let int f = 1.5;").contains("whole number"));
        assert!(batch_error("let str s = \"x\";\nif s <= \"y\" { }").contains("==, !=, < and >"));
    }
}
