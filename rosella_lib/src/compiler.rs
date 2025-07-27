use super::parser::BinaryOp;
use super::parser::Expr;
use super::parser::Stmt;
use super::parser::OS;
use super::error::RosellaError;

pub struct Compiler {
    statements: Vec<Stmt>,
    position: usize,
    os: OS,
    shell: Shell,
}

#[derive(Debug, Clone, Copy)]
pub enum Shell {
    Batch,
    Bash,
}

impl Compiler {
    pub fn new(statements: Vec<Stmt>, os: OS, shell: Shell) -> Self {
        Compiler {
            statements,
            position: 0,
            os,
            shell,
        }
    }

    fn current_statement(&self) -> &Stmt {
        self.statements.get(self.position)
            .expect("No current statement found")
    }

    pub fn compile(&mut self) -> Result<String, RosellaError> {
        let mut output = String::new();
        
        while self.position < self.statements.len() {
            output.push_str(self.compile_statement(self.current_statement())?.as_str());
            self.position += 1;
        }

        Ok(output)
    }

    fn compile_statement(&self, statement: &Stmt) -> Result<String, RosellaError> {
        match statement {
            Stmt::Let {name, value, ..} => Ok(self.compile_let_stmt(name, value, statement)?),
            Stmt::If {condition, then_branch, else_branch, .. } 
                => Ok(self.compile_if_stmt(condition, then_branch, else_branch.as_ref(), statement)?),
            Stmt::With {os, body} => Ok(self.compile_with_stmt(*os, body)?), 
            Stmt::While {condition, body, ..} 
                => Ok(self.compile_while_stmt(condition, body, statement)?),
            Stmt::Function {name, arguments, body} 
                => Ok(self.compile_function(name, arguments, body)?),
            Stmt::Expression(expr) => {
                let (name, args) = match expr {
                    Expr::Call { name, args} => (name, args),
                    _ => return Err(RosellaError::CompilerError("Expression is not a function call".to_string())),
                };

                Ok(self.compile_function_call(name, args)?)
            }
            Stmt::RawInstruction(instructions) => Ok(self.compile_raw_instruction(instructions)?),
        }
    }

    fn compile_let_stmt(&self, name: &String, value: &Expr, parent_statement: &Stmt) -> Result<String, RosellaError> {
        match self.shell {
            Shell::Batch => {
                let value_str = self.compile_expr(value, parent_statement)?;
                Ok(format!("set {}={}\n", name, value_str))
            },
            Shell::Bash => {
                let value_str = self.compile_expr(value, parent_statement)?;
                Ok(format!("{}={}\n", name, value_str))
            }
        }
    }

    fn compile_if_stmt(&self, condition: &Expr, then_branch: &Vec<Stmt>, else_branch: Option<&Vec<Stmt>>, parent_statement: &Stmt) -> Result<String, RosellaError> {
        let condition_str = self.compile_expr(condition, parent_statement)?;
        let mut output = String::new();

        match self.shell {
            Shell::Batch => {
                output.push_str(&format!("if {} (\n", condition_str));
                for stmt in then_branch {
                    output.push_str(&self.compile_statement(stmt)?);
                }
                if let Some(else_branch) = else_branch {
                    output.push_str(") else (\n");
                    for stmt in else_branch {
                        output.push_str(&self.compile_statement(stmt)?);
                    }
                }
                output.push_str(")\n");
            },
            Shell::Bash => {
                output.push_str(&format!("if [[ {} ]]; then\n", condition_str));
                for stmt in then_branch {
                    output.push_str(&self.compile_statement(stmt)?);
                }
                if let Some(else_branch) = else_branch {
                    output.push_str("else\n");
                    for stmt in else_branch {
                        output.push_str(&self.compile_statement(stmt)?);
                    }
                }
                output.push_str("fi\n");
            }
        }

        Ok(output)
    }

    fn compile_with_stmt(&self, os: OS, body: &Vec<Stmt>) -> Result<String, RosellaError> {
        let mut output = String::new();

        match (self.os, os) {
            (OS::Windows, OS::Windows) => {
                for stmt in body {
                    output.push_str(&self.compile_statement(stmt)?);
                }
            },
            (OS::Linux, OS::Linux) => {
                for stmt in body {
                    output.push_str(&self.compile_statement(stmt)?);
                }
            }
            _ => {}
        }

        Ok(output)
    }

    fn compile_while_stmt(&self, condition: &Expr, body: &Vec<Stmt>, parent_statement: &Stmt) -> Result<String, RosellaError> {
        let condition_str = self.compile_expr(condition, parent_statement)?;
        let mut output = String::new();

        match self.shell {
            Shell::Batch => {
                output.push_str(&format!(":while {} (\n", condition_str));
                for stmt in body {
                    output.push_str(&self.compile_statement(stmt)?);
                }
                output.push_str(")\n");
            },
            Shell::Bash => {
                output.push_str(&format!("while [[ {} ]]; do\n", condition_str));
                for stmt in body {
                    output.push_str(&self.compile_statement(stmt)?);
                }
                output.push_str("done\n");
            }
        }

        Ok(output)
    }

    fn compile_function(&self, name: &str, args: &Option<Vec<Expr>>, body: &Vec<Stmt>) -> Result<String, RosellaError> {
        let mut output = String::new();

        match self.shell {
            Shell::Batch => todo!("Batch shell compilation for functions not implemented yet"),
            Shell::Bash => {
                output.push_str(format!("{}() {{\n", name).as_str());
                if let Some(arguments) = args {
                    for (index, arg) in arguments.iter().enumerate() {
                        let arg_str = match arg {
                            Expr::Identifier(id) => id.clone(),
                            _ => return Err(RosellaError::CompilerError("Function arguments must be identifiers".to_string())),
                        };
                        output.push_str(format!("local {}=${}\n", arg_str, index+1).as_str());
                    }
                }
                for stmt in body {
                    output.push_str(&self.compile_statement(stmt)?);
                }
                output.push_str("}\n");
            }
        }

        Ok(output)
    }

    fn compile_function_call(&self, name: &String, args: &Vec<Expr>) -> Result<String, RosellaError> {
        let mut output = String::new();

        let allowed_std_functions = [
            "cd", "print", "echo", "make_dir", "mkdir", 
            "remove_dir", "rmdir", "remove", "del",
            "path", "copy", "cp", "move", "mv", "read",
            "exit", "exists"
        ];
        if allowed_std_functions.contains(&name.as_str()) {
            return self.compile_std_function_call(name, args);
        }

        match self.shell {
            Shell::Batch => todo!(),
            Shell::Bash => {
                output.push_str(format!("{} ", name).as_str());

                if !args.is_empty() {
                    for arg in args {
                        match arg {
                            Expr::Identifier(id) => output.push_str(format!("\"${}\" ", id).as_str()),
                            Expr::String(s) => output.push_str(format!("\"{}\" ", s).as_str()),
                            Expr::Number(n) => output.push_str(format!("{} ", n).as_str()),
                            _ => return Err(RosellaError::CompilerError(format!("Unsupported argument type in function call: {:?}", arg))),
                        }
                    }
                }
            }
        }

        output.push('\n');

        Ok(output)
    }

    fn compile_std_function_call(&self, name: &String, args: &Vec<Expr>) -> Result<String, RosellaError> {
        let mut output = String::new();

        match name.as_str() {
            "cd" => {
                if args.is_empty() {
                    return Err(RosellaError::CompilerError("cd requires a directory argument".to_string()));
                }

                output.push_str("cd ");
                output.push_str(self.format_path(args)?.as_str());
            }
            "print" | "echo" => {
                if args.is_empty() {
                    return Err(RosellaError::CompilerError("print/echo requires at least one argument".to_string()));
                }

                match self.shell {
                    Shell::Bash => {
                        output.push_str("echo \"");
                        for arg in args {
                            match arg {
                                Expr::String(s) => output.push_str(s),
                                Expr::Identifier(id) => output.push_str(format!("${}", id).as_str()),
                                Expr::Number(n) => output.push_str(n.to_string().as_str()),
                                _ => return Err(RosellaError::CompilerError(format!("Unsupported argument type in print/echo: {:?}", arg))),
                            }
                        }
                        output.push_str("\"\n");
                    }
                    Shell::Batch => {
                        output.push_str("echo ");
                        for arg in args {
                            match arg {
                                Expr::String(s) => output.push_str(s),
                                Expr::Identifier(id) => output.push_str(format!("%%{}%% ", id).as_str()),
                                Expr::Number(n) => output.push_str(n.to_string().as_str()),
                                _ => return Err(RosellaError::CompilerError(format!("Unsupported argument type in print/echo: {:?}", arg))),
                            }
                        }
                        output.push('\n');
                    }
                }
            }
            "make_dir" | "mkdir" => {
                if args.is_empty() {
                    return Err(RosellaError::CompilerError("make_dir requires at least one argument".to_string()));
                }

                match self.shell {
                    Shell::Bash => output.push_str("mkdir -p "),
                    Shell::Batch => output.push_str("mkdir "),
                }
                output.push_str(self.format_path(args)?.as_str());
            }
            "remove_dir" | "rmdir" => {
                if args.is_empty() {
                    return Err(RosellaError::CompilerError("remove_dir requires at least one argument".to_string()));
                }

                match self.shell {
                    Shell::Bash => output.push_str("rmdir "),
                    Shell::Batch => output.push_str("rmdir "),
                }
                output.push_str(self.format_path(args)?.as_str());
            }
            "remove" | "del" => {
                if args.is_empty() {
                    return Err(RosellaError::CompilerError("remove requires at least one argument".to_string()));
                }

                match self.shell {
                    Shell::Bash => output.push_str("rm -f "),
                    Shell::Batch => output.push_str("del /Q "),
                }

                output.push_str(self.format_path(args)?.as_str());
            }
            "path" => {
                if args.is_empty() {
                    return Err(RosellaError::CompilerError("path requires at least one argument".to_string()));
                }

                output.push('"');

                for arg in args {
                    let arg_str = match arg {
                        Expr::Identifier(id) => format!("${}", id.clone()),
                        Expr::String(s) => s.clone(),
                        Expr::Number(n) => n.to_string(),
                        _ => return Err(RosellaError::CompilerError(format!("Unsupported argument type: {:?}", arg))),
                    };
                    match self.os {
                        OS::Windows => output.push_str(format!("\\{}", arg_str).as_str()),
                        OS::Linux => output.push_str(format!("/{}", arg_str).as_str()),
                    }
                }

                output.push_str("\" ");
            }
            "copy" | "cp" => {
                if args.len() != 2 {
                    return Err(RosellaError::CompilerError("copy/cp requires exactly two arguments".to_string()));
                }

                match self.shell {
                    Shell::Bash => output.push_str("cp "),
                    Shell::Batch => output.push_str("copy "),
                }

                for arg in args {
                    let arg_str = match arg {
                        Expr::Call { name, args } if name == "path" => {
                            self.compile_function_call(name, args)?
                        }
                        _ => return Err(RosellaError::CompilerError(format!("copy/cp requires path() as argument, not: {:?}", arg))),
                    };

                    output.push_str(arg_str.as_str());
                }
                output.push('\n');
            }
            "move" | "mv" => {
                if args.len() != 2 {
                    return Err(RosellaError::CompilerError("move/mv requires exactly two arguments".to_string()));
                }

                match self.shell {
                    Shell::Bash => output.push_str("mv "),
                    Shell::Batch => output.push_str("move "),
                }

                for arg in args {
                    let arg_str = match arg {
                        Expr::Call { name, args } if name == "path" => {
                            self.compile_function_call(name, args)?
                        }
                        _ => return Err(RosellaError::CompilerError(format!("move/mv requires path() as argument, not: {:?}", arg))),
                    };

                    output.push_str(arg_str.as_str());
                }
                output.push('\n');
            }
            "read" => {
                if args.len() != 2 {
                    return Err(RosellaError::CompilerError("read requires exactly two arguments".to_string()));
                }

                let prompt = match &args[0] {
                    Expr::String(s) => s,
                    _ => return Err(RosellaError::CompilerError("First argument of read must be a string".to_string())),
                };

                let variable = match &args[1] {
                    Expr::Identifier(id) => id,
                    _ => return Err(RosellaError::CompilerError("Second argument of read must be an identifier".to_string())),
                };

                match self.shell {
                    Shell::Bash => {
                        output.push_str("read -p \"");
                        output.push_str(format!("{}: ", prompt).as_str());
                        output.push_str("\" ");
                        output.push_str(format!("{} ", variable).as_str());
                    }
                    Shell::Batch => {
                        output.push_str("set /p ");
                        output.push_str(format!("{}=", variable).as_str());
                        output.push_str(format!("\"{}: \"", prompt).as_str());
                    }
                }

                output.push('\n');
            }
            "exit" => {
                if args.is_empty() {
                    return Err(RosellaError::CompilerError("exit requires an exit code argument".to_string()));
                }

                let exit_code = match &args[0] {
                    Expr::Number(n) => n.to_string(),
                    _ => return Err(RosellaError::CompilerError("First argument of exit must be a number".to_string())),
                };

                match self.shell {
                    Shell::Bash => output.push_str(format!("exit {}\n", exit_code).as_str()),
                    Shell::Batch => output.push_str(format!("exit /b {}\n", exit_code).as_str()),
                }
            }
            "exists" => {
                if args.is_empty() {
                    return Err(RosellaError::CompilerError("exists requires a file path argument".to_string()));
                }

                output.push_str("-e ");

                output.push('"');
                for arg in args {
                    let arg_str = match arg {
                        Expr::Identifier(id) => format!("${}", id.clone()),
                        Expr::String(s) => s.clone(),
                        Expr::Number(n) => n.to_string(),
                        _ => return Err(RosellaError::CompilerError(format!("Unsupported argument type: {:?}", arg))),
                    };
                    match self.os {
                        OS::Windows => output.push_str(format!("\\{}", arg_str).as_str()),
                        OS::Linux => output.push_str(format!("/{}", arg_str).as_str()),
                    }
                }
                output.push('"');
            }
            
            _ => unreachable!("Standard function call compilation not implemented for: {}", name),
        }

        Ok(output)
    }

    fn compile_raw_instruction(&self, instructions: &Vec<Expr>) -> Result<String, RosellaError> {
        let mut output = String::new();

        for instruction in instructions {
            match instruction {
                Expr::String(s) => output.push_str(format!("\"{}\" ", s).as_str()),
                Expr::Identifier(s) => output.push_str(format!("{} ", s).as_str()),
                Expr::Binary { left, operator, right } => {
                    let left_str = self.compile_expr(left, self.current_statement())?;
                    let operator_str = self.format_operator(*operator, self.current_statement())?;
                    let right_str = self.compile_expr(right, self.current_statement())?;
                    output.push_str(format!("{} {} {} ", left_str, operator_str, right_str).as_str());
                },
                Expr::Number(n) => output.push_str(format!("{} ", n).as_str()),
                _ => return Err(RosellaError::CompilerError(format!("Unsupported raw instruction: {:?}", instruction))),
            }
        }

        output.push('\n');

        Ok(output)
    }

    fn compile_expr(&self, expr: &Expr, parent_statement: &Stmt) -> Result<String, RosellaError> {
        println!("Compiling expression: {:?}", expr);

        match expr {
            Expr::Number(n) => Ok(n.to_string()),
            Expr::String(s) => Ok(format!("\"{}\"", s)),
            Expr::Identifier(id) => match self.shell {
                Shell::Batch => Ok(format!("%%{}%%", id)),
                Shell::Bash => Ok(format!("${}", id)),
            },
            Expr::Binary { left, operator, right } => {
                let left_str = self.compile_expr(left, parent_statement)?;
                let operator_str = self.format_operator(*operator, parent_statement)?;
                let right_str = self.compile_expr(right, parent_statement)?;

                let condition_type = self.get_condition_type(parent_statement)?;

                match (self.shell, condition_type.as_str()) {
                    (Shell::Bash, "int") => {
                        match operator {
                            BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide => {
                                return Ok(format!("$(({} {} {}))", left_str, operator_str, right_str));
                            },
                            _ => return Ok(format!("{} {} {}", left_str, operator_str, right_str))
                        }
                    }
                    (Shell::Bash, "str") => {
                        return Ok(format!("\"{}{}\"", left_str, right_str));
                    }
                    _ => todo!("Batch shell compilation for binary expressions not implemented yet")
                }
            },
            Expr::Call { name, args } => {
                self.compile_function_call(name, args)
            }
        }
    }

    fn format_operator(&self, operator: BinaryOp, statement: &Stmt) -> Result<&str, RosellaError> {
        let condition_type = self.get_condition_type(statement)?;

        println!("Condition Type: {:?}", condition_type);
        println!("Operator: {:?}", operator);
        
        match (self.shell, condition_type.as_str(), operator) {
            (_, _, BinaryOp::Add) => Ok("+"),
            (_, _, BinaryOp::Subtract) => Ok("-"),
            (_, _, BinaryOp::Multiply) => Ok("*"),
            (_, _, BinaryOp::Divide) => Ok("/"),

            (Shell::Bash, "str", BinaryOp::Equal) => Ok("=="),
            (Shell::Bash, "str", BinaryOp::NotEqual) => Ok("!="),
            (Shell::Bash, "str", BinaryOp::LessThan) => Ok("<"),
            (Shell::Bash, "str", BinaryOp::GreaterThan) => Ok(">"),

            (Shell::Bash, "int", BinaryOp::Equal) => Ok("-eq"),
            (Shell::Bash, "int", BinaryOp::NotEqual) => Ok("-ne"),
            (Shell::Bash, "int", BinaryOp::LessThan) => Ok("-lt"),
            (Shell::Bash, "int", BinaryOp::GreaterThan) => Ok("-gt"),
            (Shell::Bash, "int", BinaryOp::LessThanEq) => Ok("-le"),
            (Shell::Bash, "int", BinaryOp::GreaterThanEq) => Ok("-ge"),

            (Shell::Batch, "str", BinaryOp::Equal) => Ok("=="),
            (Shell::Batch, "str", BinaryOp::NotEqual) => Ok("!="),

            (Shell::Batch, "int", BinaryOp::Equal) => Ok("EQU"),
            (Shell::Batch, "int", BinaryOp::NotEqual) => Ok("NEQ"),
            (Shell::Batch, "int", BinaryOp::LessThan) => Ok("LSS"),
            (Shell::Batch, "int", BinaryOp::GreaterThan) => Ok("GTR"),
            (Shell::Batch, "int", BinaryOp::LessThanEq) => Ok("LEQ"),
            (Shell::Batch, "int", BinaryOp::GreaterThanEq) => Ok("GEQ"),

            _ => Err(RosellaError::CompilerError(format!(
                "Operator: {:?} for {:?} on {:?} is not implemented.",
                operator, condition_type, self.shell)))
        }
    }

    fn format_path(&self, args: &Vec<Expr>) -> Result<String, RosellaError> {
        let mut output = String::from('"');

        for arg in args {
            let arg_str = match arg {
                Expr::Identifier(id) => format!("${}", id.clone()),
                Expr::String(s) => s.clone(),
                Expr::Number(n) => n.to_string(),
                _ => return Err(RosellaError::CompilerError(format!("Unsupported argument type: {:?}", arg))),
            };
            match self.os {
                OS::Windows => output.push_str(format!("\\{}", arg_str).as_str()),
                OS::Linux => output.push_str(format!("/{}", arg_str).as_str()),
            }
        }
        output.push_str("\"\n");

        Ok(output)
    }

    fn get_condition_type(&self, statement: &Stmt) -> Result<String, RosellaError> {
        match statement {
            Stmt::Let { variable_type, .. } => Ok(variable_type.to_string()),
            Stmt::If { condition_type, .. } => Ok(condition_type.to_string()),
            Stmt::While { condition_type, .. } => Ok(condition_type.to_string()),
            _ => return Err(RosellaError::CompilerError("No condition type found for operator formatting".to_string())),
        }
    }
}