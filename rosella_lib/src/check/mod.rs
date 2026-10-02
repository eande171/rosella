mod names;

use std::collections::HashMap;

use crate::builtins::{self, Builtin};
use crate::error::RosellaError;
use crate::syntax::{Expr, OS, Param, Stmt, Type};

// Resolve Condition Types
pub fn check(statements: &mut [Stmt], os: OS) -> Result<(), RosellaError> {
    let mut checker = Checker {
        os,
        variables: HashMap::new(),
        functions: HashMap::new(),
        parameters: HashMap::new(),
    };

    checker.declare(statements, &HashMap::new())?;
    checker.check_block(statements)
}

type Parameters = HashMap<String, Option<Type>>;

struct Checker {
    os: OS,
    variables: HashMap<String, Type>,
    functions: HashMap<String, Vec<Option<Type>>>,
    parameters: Parameters,
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
                    if *variable_type == Type::File {
                        return Err(error(format!(
                            "'let file {}' is not supported; the file type is only for conditions",
                            name
                        )));
                    }
                    self.declare_variable(name, *variable_type, parameters)?;
                }
                Stmt::Expression(Expr::Call { name, args }) if name == "read" => {
                    if let [_, Expr::Identifier(variable)] = args.as_slice() {
                        self.declare_variable(variable, Type::Str, parameters)?;
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
                    parameters: function_parameters,
                    body,
                } => {
                    self.declare_function(name, function_parameters)?;
                    self.declare(body, &parameter_types(function_parameters))?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn declare_function(&mut self, name: &str, parameters: &[Param]) -> Result<(), RosellaError> {
        if builtins::find(name).is_some() {
            return Err(error(format!(
                "Function '{}' has the same name as a built-in function",
                name
            )));
        }
        names::check_function_name(name)?;

        for parameter in parameters {
            names::check_variable_name(&parameter.name)?;
            if parameter.param_type == Some(Type::File) {
                return Err(error(format!(
                    "Parameters of '{}' cannot use the file type; it is only for conditions",
                    name
                )));
            }
        }

        let types = parameters
            .iter()
            .map(|parameter| parameter.param_type)
            .collect();
        if self.functions.insert(name.to_string(), types).is_some() {
            return Err(error(format!(
                "Function '{}' is defined more than once",
                name
            )));
        }

        Ok(())
    }

    fn declare_variable(
        &mut self,
        name: &str,
        variable_type: Type,
        parameters: &Parameters,
    ) -> Result<(), RosellaError> {
        names::check_variable_name(name)?;

        // Parameters Stay Local
        if let Some(parameter_type) = parameters.get(name) {
            return match parameter_type {
                Some(parameter_type) if *parameter_type != variable_type => Err(error(format!(
                    "Parameter '{}' is {} but is assigned as {}",
                    name, parameter_type, variable_type
                ))),
                _ => Ok(()),
            };
        }

        match self.variables.insert(name.to_string(), variable_type) {
            Some(previous) if previous != variable_type => Err(error(format!(
                "'{}' is declared as both {} and {}; a variable keeps one type",
                name, previous, variable_type
            ))),
            _ => Ok(()),
        }
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
                value,
                ..
            } => {
                let value_type = self.value_type(value)?;
                if *variable_type == Type::Int && value_type == Some(Type::Str) {
                    return Err(error(
                        "A str value cannot be stored in an int; only numbers and int variables can",
                    ));
                }
                Ok(())
            }
            Stmt::If {
                condition_type,
                condition,
                then_branch,
                else_branch,
            } => {
                *condition_type = Some(self.condition_type(condition, *condition_type)?);
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
                *condition_type = Some(self.condition_type(condition, *condition_type)?);
                self.check_block(body)
            }
            Stmt::With { os, body } if *os == self.os => self.check_block(body),
            Stmt::With { .. } => Ok(()),
            Stmt::Function {
                parameters, body, ..
            } => {
                let outer = std::mem::replace(&mut self.parameters, parameter_types(parameters));
                let result = self.check_block(body);
                self.parameters = outer;
                result
            }
            Stmt::Expression(Expr::Call { name, args }) => self.check_call(name, args),
            Stmt::Expression(expr) => Err(error(format!(
                "{} does nothing on its own; only function calls can be used as statements",
                describe(expr)
            ))),
            Stmt::RawInstruction(_) => Ok(()),
        }
    }

    fn check_call(&self, name: &str, args: &[Expr]) -> Result<(), RosellaError> {
        if let Some(parameter_types) = self.functions.get(name) {
            if args.len() != parameter_types.len() {
                return Err(error(format!(
                    "Function '{}' takes {} argument(s) but was given {}",
                    name,
                    parameter_types.len(),
                    args.len()
                )));
            }

            for (index, (parameter_type, arg)) in parameter_types.iter().zip(args).enumerate() {
                // Reject Text As Int
                if *parameter_type == Some(Type::Int) && self.value_type(arg)? == Some(Type::Str) {
                    return Err(error(format!(
                        "Argument {} of '{}' must be int but is str",
                        index + 1,
                        name
                    )));
                }
            }
            return Ok(());
        }

        let signature = self.builtin(name, args)?;
        if signature.returns.is_some() {
            return Err(error(format!(
                "{}() returns a value and cannot be used as a statement",
                name
            )));
        }

        // Skip Read Target
        let used = match (signature.builtin, args) {
            (Builtin::Read, [prompt, Expr::Identifier(_)]) => std::slice::from_ref(prompt),
            (Builtin::Read, _) => {
                return Err(error(format!(
                    "read() takes a prompt and a variable name, like {}",
                    signature.usage
                )));
            }
            _ => args,
        };

        for arg in used {
            self.value_type(arg)?;
        }

        Ok(())
    }

    fn builtin(
        &self,
        name: &str,
        args: &[Expr],
    ) -> Result<&'static builtins::Signature, RosellaError> {
        let Some(signature) = builtins::find(name) else {
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

    fn variable_type(&self, name: &str) -> Result<Option<Type>, RosellaError> {
        if let Some(parameter_type) = self.parameters.get(name) {
            return Ok(*parameter_type);
        }

        match self.variables.get(name) {
            Some(variable_type) => Ok(Some(*variable_type)),
            None => Err(error(format!(
                "Unknown variable '{}'; declare it with let before using it",
                name
            ))),
        }
    }

    // None Means Unknown
    fn expr_type(&self, expr: &Expr) -> Result<Option<Type>, RosellaError> {
        match expr {
            Expr::Number(_) => Ok(Some(Type::Int)),
            Expr::String(_) => Ok(Some(Type::Str)),
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
                    if self.value_type(side)? == Some(Type::Str) {
                        return Err(error(
                            "Arithmetic needs int values; use concat() to join strings",
                        ));
                    }
                }
                Ok(Some(Type::Int))
            }
            Expr::Call { name, args } => {
                if self.functions.contains_key(name) {
                    return Err(error(format!(
                        "Function '{}' does not return a value",
                        name
                    )));
                }

                let signature = self.builtin(name, args)?;
                let Some(returns) = signature.returns else {
                    return Err(error(format!("{}() cannot be used as a value", name)));
                };

                for arg in args {
                    self.value_type(arg)?;
                }
                Ok(Some(returns))
            }
        }
    }

    // File Checks Only In Conditions
    fn value_type(&self, expr: &Expr) -> Result<Option<Type>, RosellaError> {
        match self.expr_type(expr)? {
            Some(Type::File) => Err(error(format!(
                "{} can only be used as an if or while condition",
                describe(expr)
            ))),
            other => Ok(other),
        }
    }

    fn condition_type(
        &self,
        condition: &Expr,
        explicit: Option<Type>,
    ) -> Result<Type, RosellaError> {
        let comparison =
            matches!(condition, Expr::Binary { operator, .. } if operator.is_comparison());

        let inferred = match condition {
            Expr::Binary { left, right, .. } if comparison => {
                match (self.value_type(left)?, self.value_type(right)?) {
                    (Some(left), Some(right)) if left != right && explicit.is_none() => {
                        return Err(error(format!(
                            "This condition compares {} with {}; both sides need the same type",
                            left, right
                        )));
                    }
                    (Some(known), _) | (None, Some(known)) => Some(known),
                    (None, None) => None,
                }
            }
            _ => self.expr_type(condition)?,
        };

        // Explicit Type Wins
        match (explicit, inferred) {
            (Some(explicit), _) => Ok(explicit),
            (None, Some(Type::File)) => Ok(Type::File),
            (None, Some(inferred)) if comparison => Ok(inferred),
            (None, None) if comparison => Err(error(
                "Cannot tell whether this condition compares numbers or text; give the parameter a type like fn f(int x), or write int(...) or str(...)",
            )),
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
            condition_type("read(\"Name: \", name);\nif name == \"Bob\" { }"),
            Type::Str
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
    fn parameters() {
        assert_eq!(
            condition_type(
                "fn f(int a) { }\nfn g(int a) { if a > 1 { } }\nlet int z = 0;\nif z > 1 { }"
            ),
            Type::Int
        );
        assert_eq!(
            condition_type("fn f(a) { if a == \"x\" { } }\nlet int z = 0;\nif z > 1 { }"),
            Type::Int
        );
        assert!(check_error("fn f(a, b) { if a == b { } }").contains("Cannot tell"));
        assert!(checked("fn f(a, b) { if int(a == b) { } }").is_ok());
        assert!(check_error("fn f(int a) { }\nf(\"x\");").contains("must be int"));
        assert!(checked("fn f(str a) { }\nf(5);").is_ok());
    }

    #[test]
    fn explicit_types_still_work() {
        assert_eq!(
            condition_type("let int x = 1;\nif str(x == \"1\") { }"),
            Type::Str
        );
    }

    #[test]
    fn type_mistakes_are_reported() {
        assert!(check_error("print(y);").contains("Unknown variable 'y'"));
        assert!(check_error("let int x = 1;\nlet str x = \"a\";").contains("both int and str"));
        assert!(check_error("let int x = 1;\nif x == \"a\" { }").contains("compares int with str"));
        assert!(
            check_error("let str s = \"a\";\nlet int n = s + 1;").contains("Arithmetic needs int")
        );
        assert!(
            check_error("let str s = \"a\";\nlet int n = s;")
                .contains("cannot be stored in an int")
        );
        assert!(check_error("let int x = 1;\nif x { }").contains("needs a comparison"));
        assert!(check_error("fn f(int a) { let str a = \"x\"; }").contains("Parameter 'a' is int"));
        assert!(
            check_error("let str s = exists(\"a\");")
                .contains("only be used as an if or while condition")
        );
    }

    #[test]
    fn call_mistakes_are_reported() {
        assert!(check_error("fn add(a) { }\nlet int r = add(1);").contains("does not return"));
        assert!(check_error("get_cwd();").contains("cannot be used as a statement"));
        assert!(check_error("let str s = print(1);").contains("cannot be used as a value"));
        assert!(check_error("fn f(a) { }\nf();").contains("takes 1 argument"));
        assert!(check_error("copy(\"a\");").contains("use it like copy("));
        assert!(check_error("read(\"a\", \"b\");").contains("prompt and a variable name"));
        assert!(check_error("nope();").contains("Unknown function"));
        assert!(check_error("echo(1);").contains("Unknown function"));
        assert!(check_error("x;").contains("does nothing"));
    }

    #[test]
    fn naming_mistakes_are_reported() {
        assert!(check_error("fn print(a) { }").contains("built-in"));
        assert!(check_error("fn f() { }\nfn f() { }").contains("more than once"));
        assert!(check_error("let str path = \"x\";").contains("environment variable"));
        assert!(check_error("fn f(PATH) { }").contains("environment variable"));
        assert!(check_error("let int rosella_x = 1;").contains("reserved"));
    }

    #[test]
    fn other_platform_code_is_ignored() {
        assert!(checked("with windows { let str x = \"a\"; }\nlet int x = 1;").is_ok());
    }
}
