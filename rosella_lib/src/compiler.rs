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
    Linux,
    MacOS,
}

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

    /*fn advance(&mut self) {
        if self.position < self.statements.len(){
           self.position += 1;
        }
    }*/

    pub fn compile(&mut self) -> Result<String, RosellaError> {
        let mut output = String::new();
        
        while self.position < self.statements.len() {
            output.push_str(self.compile_statement()?.as_str());
            self.position += 1;
        }

        Ok(output)
    }

    pub fn compile_statement(&self) -> Result<String, RosellaError> {
        match self.current_statement() {
            Stmt::Let {name, value} => Ok(self.compile_let_stmt(name, value)?),
            _ => unimplemented!()
        }
    }

    pub fn compile_let_stmt(&self, name: &String, value: &Expr) -> Result<String, RosellaError> {
        match self.shell {
            Shell::Batch => {
                let value_str = self.compile_expr(value)?;
                Ok(format!("set {}={}\n", name, value_str))
            },
            Shell::Bash => {
                let value_str = self.compile_expr(value)?;
                Ok(format!("{}={}\n", name, value_str))
            }
        }
    }

    pub fn compile_expr(&self, expr: &Expr) -> Result<String, RosellaError> {
        match expr {
            Expr::Number(n) => Ok(n.to_string()),
            Expr::String(s) => Ok(format!("\"{}\"", s)),
            Expr::Identifier(id) => Ok(id.clone()),
            /*Expr::Binary { left, operator, right }=> {
                let left_str = self.compile_expr(left)?;
                let right_str = self.compile_expr(right)?;
                Ok(format!("{} {} {}", left_str, operator, right_str))
            },
            Expr::Call { name, args } => {
                let args_str: Vec<String> = args.iter()
                    .map(|arg| self.compile_expr(arg))
                    .collect::<Result<Vec<String>, RosellaError>>()?;
                Ok(format!("{}({})", name, args_str.join(", ")))
            },*/
            _ => unimplemented!("Expression type not implemented for compilation: {:?}", expr)
        }
    }

    /*fn format_operator(&self, operator: BinaryOp, expr_type: Expr) -> &str {
        
        match Shell {
            Shell::Batch => match operator {
                BinaryOp::Add => "+",
                BinaryOp::Subtract => "-",
                BinaryOp::Multiply => "*",
                BinaryOp::Divide => "/",

            }

        }
    }*/
}