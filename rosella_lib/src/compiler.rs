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

    fn compile_statement(&self) -> Result<String, RosellaError> {
        match self.current_statement() {
            Stmt::Let {name, value} => Ok(self.compile_let_stmt(name, value)?),
            _ => unimplemented!()
        }
    }

    fn compile_let_stmt(&self, name: &String, value: &Expr) -> Result<String, RosellaError> {
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

    fn compile_expr(&self, expr: &Expr) -> Result<String, RosellaError> {
        println!("Compiling expression: {:?}", expr);

        match expr {
            Expr::Number(n) => Ok(n.to_string()),
            Expr::String(s) => Ok(format!("\"{}\"", s)),
            Expr::Identifier(id) => match self.shell {
                Shell::Batch => Ok(format!("%%{}%%", id)),
                Shell::Bash => Ok(format!("${}", id)),
            }, //Ok(id.clone()),
            Expr::Binary { left, operator, right } => {
                let left_str = self.compile_expr(left)?;
                let operator = self.format_operator(*operator)?;
                let right_str = self.compile_expr(right)?;
                
                Ok(format!("{} {} {}", left_str, operator, right_str))
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

    fn format_operator(&self, operator: BinaryOp) -> Result<&str, RosellaError> {
        let condition_type = match self.current_statement() {
            Stmt::Let { .. } => &String::from("int"),
            Stmt::If { condition_type, .. } => condition_type,
            Stmt::While { condition_type, .. } => condition_type,
            _ => return Err(RosellaError::CompilerError("No condition type found for operator formatting".to_string())),
        };

        println!("Condition Type: {:?}", condition_type);
        println!("Operator: {:?}", operator);
        
        match (self.shell, condition_type.as_str(), operator) {
            (_, _, BinaryOp::Add) => Ok("+"),
            (_, _, BinaryOp::Subtract) => Ok("-"),
            (_, _, BinaryOp::Multiply) => Ok("*"),
            (_, _, BinaryOp::Divide) => Ok("/"),

            (Shell::Bash, "string", BinaryOp::Equal) => Ok("=="),
            (Shell::Bash, "string", BinaryOp::NotEqual) => Ok("!="),
            (Shell::Bash, "string", BinaryOp::LessThan) => Ok("<"),
            (Shell::Bash, "string", BinaryOp::GreaterThan) => Ok(">"),

            (Shell::Bash, "int", BinaryOp::Equal) => Ok("-eq"),
            (Shell::Bash, "int", BinaryOp::NotEqual) => Ok("-ne"),
            (Shell::Bash, "int", BinaryOp::LessThan) => Ok("-lt"),
            (Shell::Bash, "int", BinaryOp::GreaterThan) => Ok("-gt"),
            (Shell::Bash, "int", BinaryOp::LessThanEq) => Ok("-le"),
            (Shell::Bash, "int", BinaryOp::GreaterThanEq) => Ok("-ge"),

            (Shell::Batch, "string", BinaryOp::Equal) => Ok("=="),
            (Shell::Batch, "string", BinaryOp::NotEqual) => Ok("!="),

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
}