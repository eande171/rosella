mod names;

use std::collections::HashMap;

use crate::builtins::{self, Builtin, Kind, Signature};
use crate::error::RosellaError;
use crate::syntax::{BinaryOp, Condition, Expr, Iteration, OS, Param, Statement, Stmt, Type};

// Resolve Types
pub fn check(statements: &mut [Statement], os: OS) -> Result<(), RosellaError> {
    let mut checker = Checker {
        os,
        variables: HashMap::new(),
        functions: HashMap::new(),
        parameters: HashMap::new(),
        returns: None,
        loops: 0,
    };

    checker.declare(statements, &HashMap::new())?;
    checker.check_block(statements)
}

type Parameters = HashMap<String, Type>;

struct Function {
    parameters: Vec<Type>,
    returns: Option<Type>,
}

struct Checker {
    os: OS,
    variables: HashMap<String, Type>,
    functions: HashMap<String, Function>,
    parameters: Parameters,
    // Outer None Means Top Level
    returns: Option<Option<Type>>,
    loops: usize,
}

impl Checker {
    // Declarations

    // Collect Global Declarations
    fn declare(
        &mut self,
        statements: &[Statement],
        parameters: &Parameters,
    ) -> Result<(), RosellaError> {
        for statement in statements {
            self.declare_statement(&statement.kind, parameters)
                .map_err(|error| error.located(statement.span))?;
        }

        Ok(())
    }

    fn declare_statement(
        &mut self,
        statement: &Stmt,
        parameters: &Parameters,
    ) -> Result<(), RosellaError> {
        {
            match statement {
                Stmt::Let {
                    variable_type,
                    name,
                    ..
                } => self.declare_variable(name, *variable_type, parameters)?,
                Stmt::For {
                    variable_type,
                    name,
                    body,
                    ..
                } => {
                    // Loops Can Share A Variable Of The Same Type
                    if self.variables.get(name.as_str()) != Some(variable_type) {
                        self.declare_variable(name, *variable_type, parameters)?;
                    }
                    self.declare(body, parameters)?;
                }
                Stmt::If {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    self.declare(then_branch, parameters)?;
                    if let Some(else_branch) = else_branch {
                        self.declare(else_branch, parameters)?;
                    }
                }
                Stmt::While { body, .. } => self.declare(body, parameters)?,
                Stmt::With { os, body } if *os == self.os => self.declare(body, parameters)?,
                Stmt::Function {
                    name,
                    return_type,
                    parameters: function_parameters,
                    body,
                } => {
                    self.declare_function(name, *return_type, function_parameters)?;
                    self.declare(body, &parameter_types(function_parameters))?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn declare_variable(
        &mut self,
        name: &str,
        variable_type: Type,
        parameters: &Parameters,
    ) -> Result<(), RosellaError> {
        if parameters.contains_key(name) {
            return Err(error(format!(
                "'{}' is already a parameter; assign to it with {} = ...",
                name, name
            )));
        }
        names::check_variable_name(name)?;

        // Batch Ignores Case In Names
        if let Some(other) = self
            .variables
            .keys()
            .find(|other| other.as_str() != name && other.eq_ignore_ascii_case(name))
        {
            return Err(error(format!(
                "'{}' and '{}' would be the same variable in Batch; choose names that differ by more than case",
                other, name
            )));
        }

        if self
            .variables
            .insert(name.to_string(), variable_type)
            .is_some()
        {
            return Err(error(format!(
                "'{}' is declared more than once; assign to it with {} = ...",
                name, name
            )));
        }
        Ok(())
    }

    fn declare_function(
        &mut self,
        name: &str,
        returns: Option<Type>,
        parameters: &[Param],
    ) -> Result<(), RosellaError> {
        if builtins::find(name).is_some() {
            return Err(error(format!(
                "Function '{}' has the same name as a built-in function",
                name
            )));
        }
        names::check_function_name(name)?;
        names::check_shell_command(name)?;

        // Batch Ignores Case In Names
        if let Some(other) = self
            .functions
            .keys()
            .find(|other| other.as_str() != name && other.eq_ignore_ascii_case(name))
        {
            return Err(error(format!(
                "Functions '{}' and '{}' would be the same in Batch; choose names that differ by more than case",
                other, name
            )));
        }

        for (index, parameter) in parameters.iter().enumerate() {
            names::check_variable_name(&parameter.name)?;
            if parameters[..index]
                .iter()
                .any(|other| other.name.eq_ignore_ascii_case(&parameter.name))
            {
                return Err(error(format!(
                    "Function '{}' has two parameters named '{}'",
                    name, parameter.name
                )));
            }
        }

        let function = Function {
            parameters: parameters
                .iter()
                .map(|parameter| parameter.param_type)
                .collect(),
            returns,
        };
        if self.functions.insert(name.to_string(), function).is_some() {
            return Err(error(format!(
                "Function '{}' is defined more than once",
                name
            )));
        }

        Ok(())
    }

    // Statements

    fn check_block(&mut self, statements: &mut [Statement]) -> Result<(), RosellaError> {
        for statement in statements {
            self.check_statement(&mut statement.kind)
                .map_err(|error| error.located(statement.span))?;
        }
        Ok(())
    }

    fn check_statement(&mut self, statement: &mut Stmt) -> Result<(), RosellaError> {
        match statement {
            Stmt::Let {
                variable_type,
                name,
                value,
            } => {
                let value_type = self.value_type(value)?;
                assignable(*variable_type, value_type, &format!("'{}' is int", name))
            }
            Stmt::Assign {
                variable_type,
                name,
                value,
            } => {
                let target = self.variable_type(name)?;
                assignable(
                    target,
                    self.value_type(value)?,
                    &format!("'{}' is int", name),
                )?;
                *variable_type = Some(target);
                Ok(())
            }
            Stmt::If {
                resolved,
                condition,
                then_branch,
                else_branch,
            } => {
                *resolved = Some(self.resolve_condition(condition)?);
                self.check_block(then_branch)?;
                if let Some(else_branch) = else_branch {
                    self.check_block(else_branch)?;
                }
                Ok(())
            }
            Stmt::While {
                resolved,
                condition,
                body,
            } => {
                *resolved = Some(self.resolve_condition(condition)?);
                self.loops += 1;
                let result = self.check_block(body);
                self.loops -= 1;
                result
            }
            Stmt::For {
                variable_type,
                iterable,
                resolved,
                body,
                ..
            } => {
                *resolved = Some(self.resolve_iteration(*variable_type, iterable)?);
                self.loops += 1;
                let result = self.check_block(body);
                self.loops -= 1;
                result
            }
            Stmt::Break | Stmt::Continue if self.loops == 0 => {
                Err(error("break and continue can only be used inside a loop"))
            }
            Stmt::Break | Stmt::Continue => Ok(()),
            Stmt::With { os, body } if *os == self.os => self.check_block(body),
            Stmt::With { .. } => Ok(()),
            Stmt::Function {
                name,
                return_type,
                parameters,
                body,
            } => {
                if return_type.is_some() && !always_returns(body, self.os) {
                    return Err(error(format!(
                        "Function '{}' must end with a return, or return in both branches of a final if and else",
                        name
                    )));
                }

                let outer_parameters =
                    std::mem::replace(&mut self.parameters, parameter_types(parameters));
                let outer_returns = self.returns.replace(*return_type);
                let outer_loops = std::mem::replace(&mut self.loops, 0);
                let result = self.check_block(body);
                self.parameters = outer_parameters;
                self.returns = outer_returns;
                self.loops = outer_loops;
                result
            }
            Stmt::Return(value) => self.check_return(value.as_ref()),
            Stmt::Expression(Expr::Call { name, args }) => self.check_call(name, args),
            Stmt::Expression(expr) => Err(error(format!(
                "{} does nothing on its own; only function calls can be used as statements",
                describe(expr)
            ))),
            Stmt::RawInstruction(_) => Ok(()),
        }
    }

    fn check_return(&self, value: Option<&Expr>) -> Result<(), RosellaError> {
        let Some(returns) = self.returns else {
            return Err(error(
                "return can only be used inside a function; use exit() to stop the script",
            ));
        };

        match (returns, value) {
            (None, None) => Ok(()),
            (None, Some(_)) => Err(error(
                "This function has no return type; declare one like fn int name() to return a value",
            )),
            (Some(returns), None) => Err(error(format!(
                "This function must return a {} value",
                returns
            ))),
            (Some(returns), Some(value)) => assignable(
                returns,
                self.value_type(value)?,
                "This function returns int",
            ),
        }
    }

    fn check_call(&self, name: &str, args: &[Expr]) -> Result<(), RosellaError> {
        if self.functions.contains_key(name) {
            return self.check_arguments(name, args);
        }

        let signature = self.builtin(name, args)?;
        if let Kind::Value(_) = signature.kind {
            return Err(error(format!(
                "{}() returns a value and cannot be used as a statement",
                name
            )));
        }

        Ok(())
    }

    fn check_arguments(&self, name: &str, args: &[Expr]) -> Result<(), RosellaError> {
        let parameters = &self.functions[name].parameters;
        if args.len() != parameters.len() {
            return Err(error(format!(
                "Function '{}' takes {} argument(s) but was given {}",
                name,
                parameters.len(),
                args.len()
            )));
        }

        for (index, (parameter, arg)) in parameters.iter().zip(args).enumerate() {
            assignable(
                *parameter,
                self.value_type(arg)?,
                &format!("Argument {} of '{}' is int", index + 1, name),
            )?;
        }

        Ok(())
    }

    fn builtin(&self, name: &str, args: &[Expr]) -> Result<&'static Signature, RosellaError> {
        let Some(signature) = builtins::find(name) else {
            // Older Typed Condition Form
            if matches!(name, "range" | "files") {
                return Err(error(format!("{}() can only be used in a for loop", name)));
            }
            if matches!(name, "int" | "str" | "file") {
                return Err(error(format!(
                    "{}(...) is no longer needed; write the comparison directly, like if x < 10",
                    name
                )));
            }
            return Err(error(format!("Unknown function '{}'", name)));
        };

        if !signature.accepts(args.len()) {
            return Err(error(format!(
                "{}() was given {} argument(s); use it like {}",
                name,
                args.len(),
                signature.usage
            )));
        }

        self.check_builtin_arguments(signature, args)?;
        Ok(signature)
    }

    fn check_builtin_arguments(
        &self,
        signature: &Signature,
        args: &[Expr],
    ) -> Result<(), RosellaError> {
        let mut types = Vec::new();
        for arg in args {
            types.push(self.value_type(arg)?);
        }

        match signature.builtin {
            Builtin::Env | Builtin::SetEnv => {
                let Some(Expr::String(variable)) = args.first() else {
                    return Err(error(format!(
                        "{}() needs the variable name written in quotes, like {}",
                        signature.name, signature.usage
                    )));
                };
                if !is_environment_name(variable) {
                    return Err(error(format!(
                        "'{}' is not a valid environment variable name",
                        variable
                    )));
                }

                // Batch Ignores Case In Names
                let clash = self
                    .variables
                    .keys()
                    .chain(self.parameters.keys())
                    .find(|other| other.eq_ignore_ascii_case(variable));
                if let Some(other) = clash {
                    return Err(error(format!(
                        "The environment variable '{}' would clash with the Rosella variable '{}'",
                        variable, other
                    )));
                }
                if signature.builtin == Builtin::SetEnv {
                    names::check_function_name(variable)?;
                }
            }
            Builtin::Arg => {
                if types[0] != Type::Int {
                    return Err(error(format!(
                        "arg() takes the argument number, like {}",
                        signature.usage
                    )));
                }
                if let Expr::Number(number) = args[0]
                    && number < 1
                {
                    return Err(error("Arguments are counted from 1, like arg(1)"));
                }
            }
            // Batch Expands Wildcards Even In Quotes
            Builtin::Cd
            | Builtin::MakeDir
            | Builtin::Remove
            | Builtin::RemoveDir
            | Builtin::Copy
            | Builtin::Move
            | Builtin::WriteFile
            | Builtin::AppendFile
            | Builtin::Exists
            | Builtin::NotExists
            | Builtin::IsDir
            | Builtin::IsFile => {
                let paths = match signature.builtin {
                    Builtin::WriteFile | Builtin::AppendFile => &args[..1],
                    _ => args,
                };
                if paths.iter().any(has_wildcard) {
                    return Err(error(format!(
                        "Wildcards like * only work in files(); {}() would treat them differently in Bash and Batch",
                        signature.name
                    )));
                }
            }
            Builtin::Exit | Builtin::Sleep => {
                if types[0] != Type::Int {
                    return Err(error(format!(
                        "{}() takes a whole number, like {}",
                        signature.name, signature.usage
                    )));
                }
            }
            Builtin::Slice | Builtin::Random => {
                let numbers = if signature.builtin == Builtin::Slice {
                    &types[1..]
                } else {
                    &types[..]
                };
                if numbers.iter().any(|number| *number != Type::Int) {
                    return Err(error(format!(
                        "{}() needs whole numbers there, like {}",
                        signature.name, signature.usage
                    )));
                }
            }
            _ => {}
        }

        Ok(())
    }

    // Types

    fn variable_type(&self, name: &str) -> Result<Type, RosellaError> {
        if let Some(parameter_type) = self.parameters.get(name) {
            return Ok(*parameter_type);
        }

        match self.variables.get(name) {
            Some(variable_type) => Ok(*variable_type),
            None => Err(error(format!(
                "Unknown variable '{}'; declare it with let before using it",
                name
            ))),
        }
    }

    fn expr_type(&self, expr: &Expr) -> Result<Type, RosellaError> {
        match expr {
            Expr::Number(_) => Ok(Type::Int),
            Expr::String(_) => Ok(Type::Str),
            Expr::Identifier(name) => self.variable_type(name),
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                if operator.is_comparison() || operator.is_logical() {
                    return Err(error(
                        "Comparisons, && and || can only be used in if and while conditions",
                    ));
                }
                for side in [left, right] {
                    if self.value_type(side)? == Type::Str {
                        return Err(error(
                            "Arithmetic needs int values; use concat() to join strings",
                        ));
                    }
                }
                Ok(Type::Int)
            }
            Expr::Not(_) => Err(error("! can only be used in if and while conditions")),
            Expr::Call { name, args } => {
                if let Some(function) = self.functions.get(name) {
                    self.check_arguments(name, args)?;
                    return function.returns.ok_or_else(|| {
                        error(format!(
                            "Function '{}' does not return a value; declare a return type like fn int {}()",
                            name, name
                        ))
                    });
                }

                let signature = self.builtin(name, args)?;
                signature
                    .returns()
                    .ok_or_else(|| error(format!("{}() cannot be used as a value", name)))
            }
        }
    }

    // File Checks Only In Conditions
    fn value_type(&self, expr: &Expr) -> Result<Type, RosellaError> {
        match self.expr_type(expr)? {
            Type::Check => Err(error(format!(
                "{} can only be used as an if or while condition",
                describe(expr)
            ))),
            other => Ok(other),
        }
    }

    fn resolve_iteration(
        &self,
        variable_type: Type,
        iterable: &Expr,
    ) -> Result<Iteration, RosellaError> {
        let shape_error = || error("A for loop goes over range(start, end) or files(\"*.txt\")");
        let Expr::Call { name, args } = iterable else {
            return Err(shape_error());
        };

        match name.as_str() {
            "range" => {
                if variable_type != Type::Int {
                    return Err(error(
                        "A range() loop needs an int variable, like for int i in range(0, 10)",
                    ));
                }
                let (start, end, step) = match args.as_slice() {
                    [start, end] => (start, end, 1),
                    [start, end, Expr::Number(step)] if *step != 0 => (start, end, *step),
                    [_, _, _] => {
                        return Err(error(
                            "The step of range() must be a whole number other than 0, like range(10, 0, -1)",
                        ));
                    }
                    _ => return Err(error("range() takes a start and an end, like range(0, 10)")),
                };
                for bound in [start, end] {
                    if self.value_type(bound)? != Type::Int {
                        return Err(error("range() needs int values for its start and end"));
                    }
                }
                Ok(Iteration::Range {
                    start: start.clone(),
                    end: end.clone(),
                    step,
                })
            }
            "files" => {
                if variable_type != Type::Str {
                    return Err(error(
                        "A files() loop needs a str variable, like for str file in files(\"*.txt\")",
                    ));
                }
                if args.is_empty() {
                    return Err(error("files() needs a pattern, like files(\"*.txt\")"));
                }
                for arg in args {
                    self.value_type(arg)?;
                }
                Ok(Iteration::Files {
                    pattern: args.clone(),
                })
            }
            _ => Err(shape_error()),
        }
    }

    fn resolve_condition(&self, condition: &Expr) -> Result<Condition, RosellaError> {
        match condition {
            Expr::Binary {
                left,
                operator: operator @ (BinaryOp::And | BinaryOp::Or),
                right,
            } => {
                let left = Box::new(self.resolve_condition(left)?);
                let right = Box::new(self.resolve_condition(right)?);
                Ok(match operator {
                    BinaryOp::And => Condition::And(left, right),
                    _ => Condition::Or(left, right),
                })
            }
            Expr::Not(inner) => Ok(Condition::Not(Box::new(self.resolve_condition(inner)?))),
            Expr::Binary {
                left,
                operator,
                right,
            } if operator.is_comparison() => {
                let (left_type, right_type) = (self.value_type(left)?, self.value_type(right)?);
                if left_type != right_type {
                    return Err(error(format!(
                        "This condition compares {} with {}; both sides need the same type",
                        left_type, right_type
                    )));
                }
                if left_type == Type::Str
                    && matches!(operator, BinaryOp::LessThanEq | BinaryOp::GreaterThanEq)
                {
                    return Err(error("Text can be compared with ==, !=, < and >"));
                }
                Ok(Condition::Compare {
                    value_type: left_type,
                    left: (**left).clone(),
                    operator: *operator,
                    right: (**right).clone(),
                })
            }
            Expr::Call { name, args } if self.expr_type(condition)? == Type::Check => {
                Ok(Condition::Check {
                    check: name.clone(),
                    args: args.clone(),
                })
            }
            _ => {
                self.expr_type(condition)?;
                Err(error(
                    "A condition needs a comparison like x < 10 or a file check like exists(\"notes.txt\")",
                ))
            }
        }
    }
}

// Helper Functions

fn error(message: impl Into<String>) -> RosellaError {
    RosellaError::compiler(message)
}

// Numbers Can Become Text
fn assignable(target: Type, value: Type, context: &str) -> Result<(), RosellaError> {
    if target == Type::Int && value == Type::Str {
        return Err(error(format!("{} and cannot take a str value", context)));
    }
    Ok(())
}

fn is_environment_name(name: &str) -> bool {
    let mut characters = name.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && characters.all(|rest| rest.is_ascii_alphanumeric() || rest == '_')
}

fn has_wildcard(expr: &Expr) -> bool {
    match expr {
        Expr::String(text) => text.contains(['*', '?']),
        Expr::Call { name, args } if name == "path" || name == "concat" => {
            args.iter().any(has_wildcard)
        }
        _ => false,
    }
}

fn always_returns(statements: &[Statement], os: OS) -> bool {
    statements.iter().any(|statement| match &statement.kind {
        Stmt::Return(_) => true,
        Stmt::If {
            then_branch,
            else_branch: Some(else_branch),
            ..
        } => always_returns(then_branch, os) && always_returns(else_branch, os),
        Stmt::With { os: with_os, body } if *with_os == os => always_returns(body, os),
        _ => false,
    })
}

fn describe(expr: &Expr) -> String {
    match expr {
        Expr::Number(n) => format!("The number {}", n),
        Expr::String(s) => format!("The string \"{}\"", s),
        Expr::Identifier(name) => format!("The variable '{}'", name),
        Expr::Binary { .. } => "An operator expression".to_string(),
        Expr::Call { name, .. } => format!("{}()", name),
        Expr::Not(_) => "A ! expression".to_string(),
    }
}

fn parameter_types(parameters: &[Param]) -> Parameters {
    parameters
        .iter()
        .map(|parameter| (parameter.name.clone(), parameter.param_type))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::{Lexer, Parser};

    fn checked(input: &str) -> Result<Vec<Statement>, RosellaError> {
        let mut ast = Parser::new(Lexer::new(input).tokenise()?).parse()?;
        check(&mut ast, OS::Linux)?;
        Ok(ast)
    }

    fn condition_type(input: &str) -> Type {
        let ast = checked(input).unwrap();
        match &ast.last().unwrap().kind {
            Stmt::If { resolved, .. } | Stmt::While { resolved, .. } => match resolved {
                Some(Condition::Compare { value_type, .. }) => *value_type,
                Some(Condition::Check { .. }) => Type::Check,
                other => panic!("not a single comparison: {:?}", other),
            },
            other => panic!("not a condition: {:?}", other),
        }
    }

    fn check_error(input: &str) -> String {
        checked(input).unwrap_err().to_string()
    }

    #[test]
    fn infers_condition_types() {
        assert_eq!(condition_type("let int x = 1;\nif x < 10 { }"), Type::Int);
        assert_eq!(
            condition_type("let str name = \"a\";\nwhile name != \"b\" { }"),
            Type::Str
        );
        assert_eq!(condition_type("if exists(\"notes.txt\") { }"), Type::Check);
        assert_eq!(
            condition_type("let int x = 1;\nif x + 1 >= 3 { }"),
            Type::Int
        );
        assert_eq!(
            condition_type("if read(\"Name: \") == \"Bob\" { }"),
            Type::Str
        );
        assert_eq!(
            condition_type("fn int f() { return 1; }\nif f() > 0 { }"),
            Type::Int
        );
    }

    #[test]
    fn declarations_are_order_independent() {
        assert_eq!(
            condition_type("fn f() { if x > 1 { } }\nlet int x = 0;\nif x > 1 { }"),
            Type::Int
        );
    }

    #[test]
    fn assignment_fills_in_its_type() {
        let ast = checked("let int x = 1;\nx = x + 1;").unwrap();
        assert!(matches!(
            &ast[1].kind,
            Stmt::Assign {
                variable_type: Some(Type::Int),
                ..
            }
        ));
        assert!(checked("fn f(int a) { a = a + 1; }").is_ok());
    }

    #[test]
    fn returns() {
        assert!(
            checked("fn int add(int a, int b) { return a + b; }\nlet int x = add(1, 2) * 3;")
                .is_ok()
        );
        assert!(
            checked(
                "fn str name(int n) { if n == 1 { return \"one\"; } else { return \"many\"; } }"
            )
            .is_ok()
        );
        assert!(checked("fn str label(int n) { return n; }").is_ok());
        assert!(checked("fn stop() { return; }\nstop();").is_ok());
        assert!(checked("fn int add(int a) { return a; }\nadd(1);").is_ok());
        assert!(checked("let str name = read(\"Name: \");\nread(\"Press enter\");").is_ok());
    }

    #[test]
    fn return_mistakes_are_reported() {
        assert!(check_error("return;").contains("only be used inside a function"));
        assert!(check_error("fn f() { return 1; }").contains("no return type"));
        assert!(check_error("fn int f() { return; }").contains("must return a int"));
        assert!(
            check_error("fn int f() { return \"a\"; }")
                .contains("returns int and cannot take a str")
        );
        assert!(check_error("fn int f() { }").contains("must end with a return"));
        assert!(
            check_error("fn int f(int a) { if a > 1 { return 1; } }")
                .contains("must end with a return")
        );
        assert!(check_error("fn f() { }\nlet int x = f();").contains("does not return a value"));
    }

    #[test]
    fn declaration_mistakes_are_reported() {
        assert!(check_error("let int x = 1;\nlet int x = 2;").contains("declared more than once"));
        assert!(
            check_error("let int x = 1;\nlet str x = \"a\";").contains("declared more than once")
        );
        assert!(check_error("fn f(int a) { let int a = 1; }").contains("already a parameter"));
        assert!(check_error("y = 1;").contains("Unknown variable 'y'"));
        assert!(check_error("fn f(int a, str a) { }").contains("two parameters"));
    }

    #[test]
    fn type_mistakes_are_reported() {
        assert!(check_error("print(y);").contains("Unknown variable 'y'"));
        assert!(check_error("let int x = 1;\nif x == \"a\" { }").contains("compares int with str"));
        assert!(
            check_error("let str s = \"a\";\nlet int n = s + 1;").contains("Arithmetic needs int")
        );
        assert!(
            check_error("let str s = \"a\";\nlet int n = s;")
                .contains("'n' is int and cannot take a str")
        );
        assert!(check_error("let int n = 0;\nn = \"a\";").contains("cannot take a str"));
        assert!(check_error("let int x = 1;\nif x { }").contains("needs a comparison"));
        assert!(
            check_error("let str s = exists(\"a\");")
                .contains("only be used as an if or while condition")
        );
        assert!(check_error("fn f(int a) { }\nf(\"x\");").contains("Argument 1 of 'f' is int"));
        assert!(checked("fn f(str a) { }\nf(5);").is_ok());
        assert!(check_error("let int n = read(\"N: \");").contains("cannot take a str"));
    }

    #[test]
    fn older_typed_conditions_explain_the_change() {
        assert!(
            check_error("let int x = 1;\nif int(x < 10) { }")
                .contains("int(...) is no longer needed")
        );
    }

    #[test]
    fn call_mistakes_are_reported() {
        assert!(check_error("get_cwd();").contains("cannot be used as a statement"));
        assert!(check_error("let str s = print(1);").contains("cannot be used as a value"));
        assert!(check_error("fn f(int a) { }\nf();").contains("takes 1 argument"));
        assert!(
            check_error("fn int f(int a) { return a; }\nlet int x = f();")
                .contains("takes 1 argument")
        );
        assert!(check_error("copy(\"a\");").contains("use it like copy("));
        assert!(check_error("read(\"a\", name);").contains("use it like let str name = read("));
        assert!(check_error("nope();").contains("Unknown function"));
        assert!(check_error("echo(1);").contains("Unknown function"));
        assert!(check_error("x;").contains("does nothing"));
    }

    #[test]
    fn naming_mistakes_are_reported() {
        assert!(check_error("fn print(int a) { }").contains("built-in"));
        assert!(check_error("fn f() { }\nfn f() { }").contains("more than once"));
        assert!(check_error("let str path = \"x\";").contains("environment variable"));
        assert!(check_error("fn f(str PATH) { }").contains("environment variable"));
        assert!(check_error("let int rosella_x = 1;").contains("reserved"));
    }

    #[test]
    fn script_inputs() {
        assert!(checked("let str first = arg(1);\nlet int count = arg_count();\nlet str home_dir = env(\"HOME\");").is_ok());
        assert!(checked("set_env(\"MODE\", \"dev\");\nlet int code = run(\"git\", \"status\");\nrun(\"git\", \"pull\");").is_ok());
        assert!(check_error("let str a = arg(0);").contains("counted from 1"));
        assert!(check_error("let str a = arg(\"one\");").contains("argument number"));
        assert!(
            check_error("let str name = \"HOME\";\nlet str home = env(name);")
                .contains("written in quotes")
        );
        assert!(
            check_error("let str h = env(\"MY-VAR\");")
                .contains("not a valid environment variable name")
        );
        assert!(
            check_error("let str home = \"x\";\nlet str h = env(\"HOME\");")
                .contains("clash with the Rosella variable 'home'")
        );
        assert!(check_error("set_env(\"rosella_x\", \"1\");").contains("reserved"));
        assert!(check_error("arg_count();").contains("cannot be used as a statement"));
        assert!(check_error("exit(\"a\");").contains("whole number"));
    }

    #[test]
    fn text_and_utility_functions() {
        assert!(
            checked(
                "let str s = \"abc\";
let int n = length(s);
let str t = slice(s, 0, n - 1);"
            )
            .is_ok()
        );
        assert!(
            checked(
                "let str s = replace(upper(\"a\"), \"A\", lower(\"B\"));
if contains(s, \"b\") { sleep(1); }"
            )
            .is_ok()
        );
        assert!(
            checked(
                "let int r = random(1, 6);
let str d = script_dir();
if is_dir(d) && !is_file(d) { }"
            )
            .is_ok()
        );
        assert!(
            check_error("let str t = slice(\"abc\", \"0\", 1);").contains("needs whole numbers")
        );
        assert!(check_error("let int r = random(\"1\", 6);").contains("needs whole numbers"));
        assert!(check_error("sleep(\"2\");").contains("whole number"));
        assert!(
            check_error("let str c = contains(\"a\", \"b\");")
                .contains("only be used as an if or while condition")
        );
        assert!(check_error("output(\"git\");").contains("cannot be used as a statement"));
        assert!(check_error("length(\"a\");").contains("cannot be used as a statement"));
    }

    #[test]
    fn wildcards_only_work_in_files() {
        assert!(
            check_error("remove(\"*.txt\");").contains("Wildcards like * only work in files()")
        );
        assert!(check_error("if exists(path(\"logs\", \"?.log\")) { }").contains("Wildcards"));
        assert!(check_error("copy(\"a.txt\", concat(\"b\", \"*\"));").contains("Wildcards"));
        assert!(checked("write_file(\"notes.txt\", \"stars * and ?\");").is_ok());
        assert!(checked("for str f in files(\"*.txt\") { remove(f); }").is_ok());
    }

    #[test]
    fn function_names_that_break_a_shell_are_rejected() {
        assert!(check_error("fn mkdir() { }").contains("would replace a command"));
        assert!(check_error("fn printf() { }").contains("would replace a command"));
        assert!(check_error("fn Greet() { }\nfn greet() { }").contains("same in Batch"));
        assert!(check_error("fn f(int a, int A) { }").contains("two parameters"));
    }

    #[test]
    fn names_differing_by_case_are_rejected() {
        assert!(check_error("let int x = 1;\nlet int X = 2;").contains("same variable in Batch"));
    }

    #[test]
    fn logical_conditions() {
        let ast = checked(
            "let int x = 1;\nlet str s = \"a\";\nif x > 0 && (s == \"a\" || !exists(\"f\")) { }",
        )
        .unwrap();
        let Stmt::If {
            resolved: Some(Condition::And(left, right)),
            ..
        } = &ast[2].kind
        else {
            panic!("expected an and condition")
        };
        assert!(matches!(
            **left,
            Condition::Compare {
                value_type: Type::Int,
                ..
            }
        ));
        assert!(matches!(**right, Condition::Or(_, _)));
        assert!(
            check_error("let int x = 1;\nlet int y = x && 1;")
                .contains("only be used in if and while")
        );
        assert!(check_error("let int x = 1;\nif x && x > 1 { }").contains("needs a comparison"));
        assert!(check_error("let str s = \"a\";\nif s <= \"b\" { }").contains("==, !=, < and >"));
        assert!(checked("let int x = 7 % 3;").is_ok());
    }

    #[test]
    fn for_loops() {
        assert!(
            checked("for int i in range(0, 10) { if i == 3 { continue; } }\nlet int n = i;")
                .is_ok()
        );
        assert!(checked("let int n = 3;\nfor int i in range(n, 0, -1) { break; }").is_ok());
        assert!(
            checked(
                "let str dir = \"logs\";\nfor str file in files(dir, \"*.log\") { print(file); }"
            )
            .is_ok()
        );
        assert!(check_error("for str i in range(0, 10) { }").contains("needs an int variable"));
        assert!(check_error("for int f in files(\"*\") { }").contains("needs a str variable"));
        assert!(
            check_error("let int s = 1;\nfor int i in range(0, 10, s) { }")
                .contains("step of range()")
        );
        assert!(check_error("for int i in range(0, 10, 0) { }").contains("step of range()"));
        assert!(check_error("for int i in range(0) { }").contains("start and an end"));
        assert!(check_error("for int i in 10 { }").contains("goes over range"));
        assert!(checked("for int i in range(0, 2) { }\nfor int i in range(0, 3) { }").is_ok());
        assert!(checked("let int i = 0;\nfor int i in range(0, 1) { }").is_ok());
        assert!(
            check_error("let str i = \"a\";\nfor int i in range(0, 1) { }")
                .contains("declared more than once")
        );
        assert!(check_error("let int r = range(0, 1);").contains("only be used in a for loop"));
    }

    #[test]
    fn break_and_continue_need_a_loop() {
        assert!(checked("let int i = 0;\nwhile i < 3 { if i == 1 { break; } continue; }").is_ok());
        assert!(check_error("break;").contains("inside a loop"));
        assert!(
            check_error("let int i = 0;\nwhile i < 3 { fn f() { continue; } }")
                .contains("inside a loop")
        );
    }

    #[test]
    fn errors_point_at_their_statement() {
        let message = check_error(
            "let int x = 1;
fn f() {
    print(y);
}",
        );
        assert!(
            message.starts_with("line 3, column 5: Unknown variable 'y'"),
            "{}",
            message
        );
        let message = check_error(
            "let int x = 1;
if x == 1 { } else if x == \"a\" { }",
        );
        assert!(
            message.starts_with("line 2, column 20: This condition compares"),
            "{}",
            message
        );
    }

    #[test]
    fn other_platform_code_is_ignored() {
        assert!(checked("with windows { let str x = \"a\"; }\nlet int x = 1;").is_ok());
    }
}
