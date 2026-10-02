use crate::error::RosellaError;
use crate::syntax::BinaryOp;

// Quotable String Pieces
#[derive(Debug, Clone, PartialEq)]
pub enum Part {
    Text(String),
    Var(String),
    Cwd,
}

// Shared Arithmetic
pub struct Arith {
    pub text: String,
    pub literal: Option<i64>,
    pub grouped: bool,
}

impl Arith {
    // Keep Nested Grouping
    pub fn operand(&self) -> String {
        if self.grouped {
            format!("({})", self.text)
        } else {
            self.text.clone()
        }
    }
}

pub enum Test {
    Int {
        left: Arith,
        operator: BinaryOp,
        right: Arith,
    },
    Str {
        left: Vec<Part>,
        operator: BinaryOp,
        right: Vec<Part>,
    },
    File {
        negate: bool,
        path: Vec<Part>,
    },
}

pub struct Condition {
    pub setup: String,
    pub test: String,
}

pub enum Arg {
    Int(Arith),
    Value(Vec<Part>),
}

// Return From Inside A Loop
#[derive(Clone, Copy, PartialEq)]
pub enum ReturnCheck {
    None,
    Propagate,
    Clear,
}

#[derive(Clone, Copy)]
pub enum Transfer {
    Copy,
    Move,
}

// Depth Counts Enclosing Calls
pub trait Backend {
    fn program(&mut self, body: String) -> String;
    fn empty_body(&self) -> &'static str;

    fn supports_recursion(&self) -> bool {
        true
    }

    fn local_name(&self, _function: &str, parameter: &str) -> String {
        parameter.to_string()
    }

    fn assign_int(&self, name: &str, value: &Arith) -> String;
    fn assign_str(&self, name: &str, value: &[Part]) -> Result<String, RosellaError>;

    fn condition(&mut self, test: Test, setup: String) -> Result<Condition, RosellaError>;
    fn if_chain(&self, branches: Vec<(Condition, String)>, otherwise: Option<String>) -> String;
    fn while_loop(&mut self, condition: Condition, body: String, check: ReturnCheck) -> String;
    fn function(&mut self, name: &str, parameters: &[String], body: String) -> String;
    fn call(&self, name: &str, args: &[Arg]) -> Result<String, RosellaError>;
    fn capture(&self, temporary: &str, local: bool) -> String;
    fn return_from_function(&self, nested_in_loop: bool) -> String;

    fn print(&self, text: &[Part]) -> Result<String, RosellaError>;
    fn cd(&self, path: &[Part]) -> Result<String, RosellaError>;
    fn make_dir(&self, path: &[Part]) -> Result<String, RosellaError>;
    fn remove(&self, path: &[Part], directory: bool, depth: usize) -> Result<String, RosellaError>;
    fn transfer(
        &self,
        transfer: Transfer,
        source: &[Part],
        destination: &[Part],
    ) -> Result<String, RosellaError>;
    fn write(&self, path: &[Part], content: &[Part], append: bool) -> Result<String, RosellaError>;
    fn read(&self, prompt: &[Part], variable: &str, local: bool) -> Result<String, RosellaError>;
    fn exit(&self, code: &Arith, depth: usize) -> String;
}
