mod names;

use std::collections::HashMap;

use crate::builtins::{self, Builtin, Signature};
use crate::error::RosellaError;
use crate::syntax::{Expr, OS, Param, Stmt, Type};

// Resolve Types
pub fn check(statements: &mut [Stmt], os: OS) -> Result<(), RosellaError> {
    let mut checker = Checker {
        os,
        variables: HashMap::new(),
        functions: HashMap::new(),
        parameters: HashMap::new(),
        returns: None,
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
}

impl Checker {
    // Declarations

    // Collect Global Declarations
    fn declare(
        &mut self,
        statements: &[Stmt],
        parameters: &Parameters,
    ) -> Result<(), RosellaError> {
        for statement in statements {
            match statement {
                Stmt::Let {
                    variable_type,
                    name,
                    ..
                } => {
                    if parameters.contains_key(name) {
                        return Err(error(format!(
                            "'{}' is already a parameter; assign to it with {} = ...",
                            name, name
                        )));
                    }
                    names::check_variable_name(name)?;
                    if self
                        .variables
                        .insert(name.clone(), *variable_type)
                        .is_some()
                    {
                        return Err(error(format!(
                            "'{}' is declared more than once; assign to it with {} = ...",
                            name, name
                        )));
                    }
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

        for (index, parameter) in parameters.iter().enumerate() {
            names::check_variable_name(&parameter.name)?;
            if parameters[..index]
                .iter()
                .any(|other| other.name == parameter.name)
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

    fn check_block(&mut self, statements: &mut [Stmt]) -> Result<(), RosellaError> {
        for statement in statements {
            self.check_statement(statement)?;
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
                condition_type,
                condition,
                then_branch,
                else_branch,
            } => {
                *condition_type = Some(self.condition_type(condition)?);
                self.check_block(then_branch)?;
                if let Some(else_branch) = else_branch {
                    self.check_block(else_branch)?;
                }
                Ok(())
            }
            Stmt::While {
                condition_type,
                condition,
                body,
            } => {
                *condition_type = Some(self.condition_type(condition)?);
                self.check_block(body)
            }
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
                let result = self.check_block(body);
                self.parameters = outer_parameters;
                self.returns = outer_returns;
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

        // Read Can Discard Its Input
        if signature.returns.is_some() && signature.builtin != Builtin::Read {
            return Err(error(format!(
                "{}() returns a value and cannot be used as a statement",
                name
            )));
        }

        for arg in args {
            self.value_type(arg)?;
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

        Ok(signature)
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
                if operator.is_comparison() {
                    return Err(error(
                        "Comparisons can only be used in if and while conditions",
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
                let Some(returns) = signature.returns else {
                    return Err(error(format!("{}() cannot be used as a value", name)));
                };

                for arg in args {
                    self.value_type(arg)?;
                }
                Ok(returns)
            }
        }
    }

    // File Checks Only In Conditions
    fn value_type(&self, expr: &Expr) -> Result<Type, RosellaError> {
        match self.expr_type(expr)? {
            Type::File => Err(error(format!(
                "{} can only be used as an if or while condition",
                describe(expr)
            ))),
            other => Ok(other),
        }
    }

    fn condition_type(&self, condition: &Expr) -> Result<Type, RosellaError> {
        if let Expr::Binary {
            left,
            operator,
            right,
        } = condition
            && operator.is_comparison()
        {
            let (left, right) = (self.value_type(left)?, self.value_type(right)?);
            if left != right {
                return Err(error(format!(
                    "This condition compares {} with {}; both sides need the same type",
                    left, right
                )));
            }
            return Ok(left);
        }

        match self.expr_type(condition)? {
            Type::File => Ok(Type::File),
            _ => Err(error(
                "A condition needs a comparison like x < 10 or a file check like exists(\"notes.txt\")",
            )),
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

fn always_returns(statements: &[Stmt], os: OS) -> bool {
    statements.iter().any(|statement| match statement {
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

    fn checked(input: &str) -> Result<Vec<Stmt>, RosellaError> {
        let mut ast = Parser::new(Lexer::new(input).tokenise()?).parse()?;
        check(&mut ast, OS::Linux)?;
        Ok(ast)
    }

    fn condition_type(input: &str) -> Type {
        let ast = checked(input).unwrap();
        match ast.last().unwrap() {
            Stmt::If { condition_type, .. } | Stmt::While { condition_type, .. } => {
                condition_type.unwrap()
            }
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
        assert_eq!(condition_type("if exists(\"notes.txt\") { }"), Type::File);
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
            &ast[1],
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
    fn other_platform_code_is_ignored() {
        assert!(checked("with windows { let str x = \"a\"; }\nlet int x = 1;").is_ok());
    }
}
