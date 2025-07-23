use super::parser::BinaryOp;
use super::parser::Expr;
use super::parser::Stmt;
use super::error::RosellaError;

pub struct Compiler {
    statements: Vec<Stmt>,
    position: usize,
    os: OS,
    shell: Shell,
}

pub enum OS {
    Windows,
    Linux
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
            _ => unimplemented!()
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

    fn compile_expr(&self, expr: &Expr, parent_statement: &Stmt) -> Result<String, RosellaError> {
        println!("Compiling expression: {:?}", expr);

        match expr {
            Expr::Number(n) => Ok(n.to_string()),
            Expr::String(s) => Ok(format!("\"{}\"", s)),
            Expr::Identifier(id) => match self.shell {
                Shell::Batch => Ok(format!("%%{}%%", id)),
                Shell::Bash => Ok(format!("${}", id)),
            }, //Ok(id.clone()),
            Expr::Binary { left, operator, right } => {
                let left_str = self.compile_expr(left, parent_statement)?;
                let operator_str = self.format_operator(*operator, parent_statement)?;
                let right_str = self.compile_expr(right, parent_statement)?;

                let condition_type = self.get_condition_type(parent_statement)?;

                match (self.shell, condition_type.as_str()) {
                    (Shell::Bash, "int") => {
                        match operator {
                            BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide => {
                                return Ok(format!("(({} {} {}))", left_str, operator_str, right_str));
                            },
                            _ => return Ok(format!("{} {} {}", left_str, operator_str, right_str))
                        }
                    }
                    (Shell::Bash, "str") => {
                        return Ok(format!("\"{}{}\"", left_str, right_str));
                    }
                    _ => unimplemented!("Batch shell compilation for binary expressions not implemented yet")
                }
            },
            /*Expr::Call { name, args } => {
                let args_str: Vec<String> = args.iter()
                    .map(|arg| self.compile_expr(arg))
                    .collect::<Result<Vec<String>, RosellaError>>()?;
                Ok(format!("{}({})", name, args_str.join(", ")))
            },*/
            _ => unimplemented!("Expression type not implemented for compilation: {:?}", expr)
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

    fn get_condition_type(&self, statement: &Stmt) -> Result<String, RosellaError> {
        match statement {
            Stmt::Let { variable_type, .. } => Ok(variable_type.to_string()),
            Stmt::If { condition_type, .. } => Ok(condition_type.to_string()),
            Stmt::While { condition_type, .. } => Ok(condition_type.to_string()),
            _ => return Err(RosellaError::CompilerError("No condition type found for operator formatting".to_string())),
        }
    }
}