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
    // A Variable That Always Holds A Number
    pub known: bool,
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
        check: FileCheck,
        path: Vec<Part>,
    },
    Contains {
        text: Vec<Part>,
        part: Vec<Part>,
    },
}

#[derive(Clone, Copy, PartialEq)]
pub enum FileCheck {
    Exists,
    Missing,
    Directory,
    File,
}

pub struct Condition {
    pub setup: String,
    pub test: String,
}

// Each Test Carries Its Own Setup
pub enum Logic {
    Test { setup: String, test: Test },
    Not(Box<Logic>),
    And(Box<Logic>, Box<Logic>),
    Or(Box<Logic>, Box<Logic>),
}

pub enum Arg {
    Int(Arith),
    Value(Vec<Part>),
}

#[derive(Clone, Copy, PartialEq)]
pub enum LoopKind {
    While,
    Range,
    Files,
}

#[derive(Clone)]
pub struct LoopLabel {
    pub name: String,
    pub kind: LoopKind,
    pub exits: bool,
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

    fn local_name(&self, function: &str, parameter: &str) -> String;

    fn assign_int(&self, name: &str, value: &Arith) -> String;
    fn assign_str(&self, name: &str, value: &[Part]) -> Result<String, RosellaError>;

    fn condition(&mut self, logic: Logic) -> Result<Condition, RosellaError>;
    fn if_chain(&self, branches: Vec<(Condition, String)>, otherwise: Option<String>) -> String;
    // Exits Means The Body Can Stop The Script
    fn begin_loop(&mut self, kind: LoopKind, exits: bool) -> LoopLabel;
    fn while_loop(
        &mut self,
        label: &LoopLabel,
        condition: Condition,
        body: String,
        check: ReturnCheck,
    ) -> String;
    #[allow(clippy::too_many_arguments)]
    fn range_loop(
        &mut self,
        label: &LoopLabel,
        variable: &str,
        start: &Arith,
        end: &Arith,
        step: i64,
        body: String,
        check: ReturnCheck,
    ) -> String;
    fn files_loop(
        &mut self,
        label: &LoopLabel,
        variable: &str,
        pattern: &[Part],
        body: String,
        check: ReturnCheck,
    ) -> Result<String, RosellaError>;
    fn break_loop(&self, label: &LoopLabel) -> String;
    fn continue_loop(&self, label: &LoopLabel) -> String;
    fn function(&mut self, name: &str, parameters: &[String], body: String) -> String;
    fn call(&self, name: &str, args: &[Arg], exits: bool) -> Result<String, RosellaError>;
    fn capture(&self, temporary: &str, local: bool) -> String;
    fn declare_local(&self, name: &str) -> String;
    fn return_from_function(&self, nested_in_loop: bool) -> String;

    fn print(&self, text: &[Part]) -> Result<String, RosellaError>;
    fn cd(&self, path: &[Part]) -> Result<String, RosellaError>;
    fn make_dir(&self, path: &[Part]) -> Result<String, RosellaError>;
    fn remove(&self, path: &[Part], directory: bool) -> Result<String, RosellaError>;
    fn transfer(
        &self,
        transfer: Transfer,
        source: &[Part],
        destination: &[Part],
    ) -> Result<String, RosellaError>;
    fn write(&self, path: &[Part], content: &[Part], append: bool) -> Result<String, RosellaError>;
    fn read(&self, prompt: &[Part], variable: &str, local: bool) -> Result<String, RosellaError>;
    fn exit(&self, code: &Arith, depth: usize) -> String;

    fn argument(&mut self, index: &Arith, target: &str, local: bool) -> String;
    fn argument_count(&mut self, target: &str, local: bool) -> String;
    fn run(&self, command: &[Vec<Part>]) -> Result<String, RosellaError>;
    fn capture_status(&self, target: &str, local: bool) -> String;
    fn output(
        &self,
        command: &[Vec<Part>],
        target: &str,
        local: bool,
    ) -> Result<String, RosellaError>;
    fn set_env(&self, name: &str, value: &[Part]) -> Result<String, RosellaError>;

    fn length(&mut self, text: &[Part], target: &str, local: bool) -> Result<String, RosellaError>;
    fn slice(
        &mut self,
        text: &[Part],
        start: &Arith,
        count: &Arith,
        target: &str,
        local: bool,
    ) -> Result<String, RosellaError>;
    fn replace(
        &mut self,
        text: &[Part],
        from: &[Part],
        to: &[Part],
        target: &str,
        local: bool,
    ) -> Result<String, RosellaError>;
    fn change_case(
        &mut self,
        text: &[Part],
        upper: bool,
        target: &str,
        local: bool,
    ) -> Result<String, RosellaError>;
    fn random(&mut self, min: &Arith, max: &Arith, target: &str, local: bool) -> String;
    fn sleep(&self, seconds: &Arith) -> String;
    fn use_script_dir(&mut self);
}
