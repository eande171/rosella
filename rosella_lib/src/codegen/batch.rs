use super::backend::{
    Arg, Arith, Backend, Condition, FileCheck, Logic, LoopKind, LoopLabel, Part, ReturnCheck, Test,
    Transfer,
};
use super::{error, indent};
use crate::error::RosellaError;
use crate::syntax::BinaryOp;

#[derive(Default)]
pub struct Batch {
    subroutines: String,
    unique_index: usize,
    uses_arguments: bool,
    uses_script_dir: bool,
    uses_length: bool,
    uses_replace: bool,
    uses_contains: bool,
    uses_keep: bool,
    uses_capture: bool,
    uses_slashes: bool,
}

impl Batch {
    fn next_index(&mut self) -> usize {
        let index = self.unique_index;
        self.unique_index += 1;
        index
    }

    fn int_operand(&mut self, value: &Arith, setup: &mut String) -> String {
        if let Some(literal) = value.literal {
            return literal.to_string();
        }
        if !value.grouped {
            return format!("!{}!", value.text);
        }
        self.int_number(value, setup)
    }

    // Computed Number
    fn int_number(&mut self, value: &Arith, setup: &mut String) -> String {
        if let Some(literal) = value.literal {
            return literal.to_string();
        }

        let temporary = format!("rosella_temp{}", self.next_index());
        setup.push_str(&format!("set /a \"{}={}\"\n", temporary, arithmetic(value)));
        format!("!{}!", temporary)
    }
}

impl Batch {
    fn leaf(&mut self, test: Test, mut setup: String) -> Result<Condition, RosellaError> {
        match test {
            Test::Int {
                left,
                operator,
                right,
            } => {
                let left = self.int_operand(&left, &mut setup);
                let right = self.int_operand(&right, &mut setup);
                let operator = match operator {
                    BinaryOp::Equal => "EQU",
                    BinaryOp::NotEqual => "NEQ",
                    BinaryOp::LessThan => "LSS",
                    BinaryOp::LessThanEq => "LEQ",
                    BinaryOp::GreaterThan => "GTR",
                    _ => "GEQ",
                };
                Ok(Condition {
                    setup,
                    test: format!("{} {} {}", left, operator, right),
                })
            }
            Test::Str {
                left,
                operator,
                right,
            } => {
                let (left, right) = (quoted(&left)?, quoted(&right)?);
                let test = match operator {
                    BinaryOp::Equal => format!("\"{}\"==\"{}\"", left, right),
                    BinaryOp::NotEqual => format!("not \"{}\"==\"{}\"", left, right),
                    BinaryOp::LessThan => format!("\"{}\" LSS \"{}\"", left, right),
                    _ => format!("\"{}\" GTR \"{}\"", left, right),
                };
                Ok(Condition { setup, test })
            }
            Test::File { check, path } => {
                let quoted_path = quoted(&path)?;
                // Empty Path
                let may_be_empty = check == FileCheck::Directory
                    && !path
                        .iter()
                        .any(|part| matches!(part, Part::Text(text) if !text.is_empty()));

                // Folder Check
                let found = match check {
                    FileCheck::Directory if may_be_empty => {
                        format!("not \"{0}\"==\"\" if exist \"{0}\\\"", quoted_path)
                    }
                    FileCheck::Directory => format!("exist \"{}\\\"", quoted_path),
                    FileCheck::File => {
                        format!("exist \"{0}\" if not exist \"{0}\\\"", quoted_path)
                    }
                    FileCheck::Exists | FileCheck::Missing => format!("exist \"{}\"", quoted_path),
                };
                let negate = if check == FileCheck::Missing {
                    "not "
                } else {
                    ""
                };

                // Flag Test
                let guard = wildcard_guard(&path);
                if check != FileCheck::File && !may_be_empty && guard.is_empty() {
                    return Ok(Condition {
                        setup,
                        test: format!("{}{}", negate, found),
                    });
                }

                let flag = format!("rosella_condition{}", self.next_index());
                setup.push_str(&format!(
                    "set \"{flag}=\"\n{guard}if {found} set \"{flag}=1\"\n",
                    flag = flag,
                    guard = guard,
                    found = found
                ));
                Ok(Condition {
                    setup,
                    test: format!("{}defined {}", negate, flag),
                })
            }
            Test::Contains { text, part } => {
                self.uses_contains = true;
                setup.push_str(&format!(
                    "set \"rosella_text={}\"\nset \"rosella_from={}\"\ncall :rosella_contains\n",
                    quoted(&text)?,
                    quoted(&part)?
                ));
                Ok(Condition {
                    setup,
                    test: "defined rosella_found".to_string(),
                })
            }
        }
    }

    fn command(&mut self, command: &[Vec<Part>], redirect: &str) -> Result<String, RosellaError> {
        let mut output = String::new();
        let mut words = Vec::new();

        for (index, parts) in command.iter().enumerate() {
            if index == 0 {
                output.push_str(&format!("set \"rosella_command={}\"\n", quoted(parts)?));
                words.push("\"%%rosella_command%%\"".to_string());
                continue;
            }

            // Trailing Backslashes
            let name = format!("rosella_argument{}", index);
            let fixed = parts.iter().all(|part| matches!(part, Part::Text(_)));
            if fixed {
                let text: String = parts
                    .iter()
                    .map(|part| match part {
                        Part::Text(text) => text.as_str(),
                        _ => "",
                    })
                    .collect();
                let trailing = text.len() - text.trim_end_matches('\\').len();
                let doubled = format!("{}{}", text, "\\".repeat(trailing));
                output.push_str(&format!(
                    "set \"{}={}\"\n",
                    name,
                    quoted(&[Part::Text(doubled)])?
                ));
            } else {
                self.uses_slashes = true;
                output.push_str(&format!(
                    "set \"{name}={value}\"\nif \"!{name}:~-1!\"==\"\\\" call :rosella_slashes {name}\n",
                    name = name,
                    value = quoted(parts)?
                ));
            }
            words.push(format!("\"%%{}%%\"", name));
        }

        // Literal Arguments
        output.push_str("setlocal disabledelayedexpansion\n");
        output.push_str(&format!("call {}{}\n", words.join(" "), redirect));
        output.push_str("endlocal\n");
        Ok(output)
    }

    fn single(&mut self, logic: Logic) -> Result<Result<Condition, Logic>, RosellaError> {
        match logic {
            Logic::Test { setup, test } => Ok(Ok(self.leaf(test, setup)?)),
            Logic::Not(inner) => Ok(self
                .single(*inner)?
                .map(negate)
                .map_err(|inner| Logic::Not(Box::new(inner)))),
            other => Ok(Err(other)),
        }
    }

    fn flag_lines(&mut self, logic: Logic, flag: &str) -> Result<String, RosellaError> {
        let logic = match self.single(logic)? {
            Ok(condition) => {
                return Ok(format!(
                    "{}if {} set \"{}=1\"\n",
                    condition.setup, condition.test, flag
                ));
            }
            Err(logic) => logic,
        };

        match logic {
            Logic::Test { .. } => Err(error("A single test should not need a flag")),
            Logic::Not(inner) => {
                let inner_flag = format!("rosella_condition{}", self.next_index());
                let lines = self.flag_lines(*inner, &inner_flag)?;
                Ok(format!(
                    "set \"{inner}=\"\n{lines}if not defined {inner} set \"{flag}=1\"\n",
                    inner = inner_flag,
                    lines = lines,
                    flag = flag
                ))
            }
            // Short Circuit
            Logic::And(left, right) => {
                let left_flag = format!("rosella_condition{}", self.next_index());
                let left_lines = self.flag_lines(*left, &left_flag)?;
                let right_lines = self.flag_lines(*right, flag)?;
                Ok(format!(
                    "set \"{left}=\"\n{left_lines}if defined {left} (\n{right_lines})\n",
                    left = left_flag,
                    left_lines = left_lines,
                    right_lines = indent(&right_lines)
                ))
            }
            Logic::Or(left, right) => {
                let left_lines = self.flag_lines(*left, flag)?;
                let right_lines = self.flag_lines(*right, flag)?;
                Ok(format!(
                    "{left_lines}if not defined {flag} (\n{right_lines})\n",
                    left_lines = left_lines,
                    flag = flag,
                    right_lines = indent(&right_lines)
                ))
            }
        }
    }
}

impl Backend for Batch {
    fn program(&mut self, body: String) -> String {
        let arguments = if self.uses_arguments {
            concat!(
                "set \"rosella_argc=0\"\n",
                ":rosella_arguments\n",
                "if \"%~1\"==\"\" if [%1]==[] goto :rosella_arguments_done\n",
                "set /a \"rosella_argc+=1\"\n",
                "set \"rosella_arg_%rosella_argc%=%~1\"\n",
                "shift\n",
                "goto :rosella_arguments\n",
                ":rosella_arguments_done\n",
            )
        } else {
            ""
        };

        let script_dir = if self.uses_script_dir {
            "set \"rosella_script_dir=%~dp0\"\nset \"rosella_script_dir=%rosella_script_dir:~0,-1%\"\n"
        } else {
            ""
        };

        let capture = if self.uses_capture {
            "set \"rosella_capture=%TEMP%\\rosella_%RANDOM%%RANDOM%.tmp\"\n"
        } else {
            ""
        };

        // Private Scope
        let early = format!("{}{}{}", script_dir, arguments, capture);
        let scope = if early.is_empty() { "" } else { "setlocal\n" };

        let mut output = format!(
            "@echo off\n{}{}setlocal enabledelayedexpansion\nset \"rosella_exit=\"\n{}",
            scope, early, body
        );

        let mut helpers = String::new();
        if self.uses_contains {
            helpers.push_str(CONTAINS_HELPER);
        }
        if self.uses_replace || self.uses_contains {
            helpers.push_str(REPLACE_HELPER);
        }
        if self.uses_length || self.uses_replace || self.uses_contains {
            helpers.push_str(LENGTH_HELPER);
        }
        if self.uses_keep {
            helpers.push_str(KEEP_HELPER);
        }
        if self.uses_slashes {
            helpers.push_str(SLASHES_HELPER);
        }

        // Default Exit Code
        output.push_str("exit /b 0\n");

        // Subroutines Last
        if !self.subroutines.is_empty() || !helpers.is_empty() {
            output.push('\n');
            output.push_str(&self.subroutines);
            output.push_str(&helpers);
        }

        output.replace('\n', "\r\n")
    }

    fn empty_body(&self) -> &'static str {
        "rem\n"
    }

    // No Call Stack
    fn supports_recursion(&self) -> bool {
        false
    }

    // No Local Variables
    fn local_name(&self, function: &str, parameter: &str) -> String {
        // Dot Separator
        format!("rosella_{}.{}", function, parameter)
    }

    fn assign_int(&self, name: &str, value: &Arith) -> String {
        format!("set /a \"{}={}\"\n", name, arithmetic(value))
    }

    fn assign_str(&self, name: &str, value: &[Part]) -> Result<String, RosellaError> {
        Ok(format!("set \"{}={}\"\n", name, quoted(value)?))
    }

    fn condition(&mut self, logic: Logic) -> Result<Condition, RosellaError> {
        let logic = match self.single(logic)? {
            Ok(condition) => return Ok(condition),
            Err(logic) => logic,
        };

        let flag = format!("rosella_condition{}", self.next_index());
        let lines = self.flag_lines(logic, &flag)?;
        Ok(Condition {
            setup: format!("set \"{}=\"\n{}", flag, lines),
            test: format!("defined {}", flag),
        })
    }

    fn begin_loop(&mut self, kind: LoopKind, exits: bool) -> LoopLabel {
        let prefix = match kind {
            LoopKind::While => "rosella_while",
            LoopKind::Range => "rosella_for",
            LoopKind::Files => "rosella_files",
        };
        LoopLabel {
            name: format!("{}{}", prefix, self.next_index()),
            kind,
            exits,
        }
    }

    // Stop Files Loop
    fn break_loop(&self, label: &LoopLabel) -> String {
        match label.kind {
            LoopKind::Files => "set \"rosella_break=1\"\ngoto :eof\n".to_string(),
            _ => "goto :eof\n".to_string(),
        }
    }

    fn continue_loop(&self, label: &LoopLabel) -> String {
        match label.kind {
            LoopKind::While => format!("goto :{}\n", label.name),
            LoopKind::Range => format!("goto :{}_next\n", label.name),
            LoopKind::Files => "goto :eof\n".to_string(),
        }
    }

    // Nest Else If
    fn if_chain(&self, branches: Vec<(Condition, String)>, otherwise: Option<String>) -> String {
        let mut tail = otherwise;
        let mut output = String::new();

        for (condition, body) in branches.into_iter().rev() {
            output = format!("{}if {} (\n{}", condition.setup, condition.test, body);
            if let Some(else_body) = tail {
                output.push_str(&format!(") else (\n{}", else_body));
            }
            output.push_str(")\n");
            tail = Some(indent(&output));
        }

        output
    }

    // Loops As Subroutines
    fn while_loop(
        &mut self,
        label: &LoopLabel,
        condition: Condition,
        body: String,
        check: ReturnCheck,
    ) -> String {
        self.subroutines.push_str(&format!(
            ":{label}\n{setup}if {test} goto :{label}_body\ngoto :eof\n:{label}_body\n{body}goto :{label}\n\n",
            label = label.name,
            setup = condition.setup,
            test = condition.test,
            body = body,
        ));

        loop_call(label, check)
    }

    fn range_loop(
        &mut self,
        label: &LoopLabel,
        variable: &str,
        start: &Arith,
        end: &Arith,
        step: i64,
        body: String,
        check: ReturnCheck,
    ) -> String {
        let end = match end.literal {
            Some(literal) => literal.to_string(),
            None => format!("!{}!", end.text),
        };
        let (comparison, change) = if step > 0 {
            ("LSS", format!("{} + {}", variable, step))
        } else {
            ("GTR", format!("{} - {}", variable, -step))
        };

        self.subroutines.push_str(&format!(
            ":{label}\nif !{variable}! {comparison} {end} goto :{label}_body\ngoto :eof\n:{label}_body\n{body}:{label}_next\nset /a \"{variable}={change}\"\ngoto :{label}\n\n",
            label = label.name,
            variable = variable,
            comparison = comparison,
            end = end,
            body = body,
            change = change,
        ));

        let mut output = format!("set /a \"{}={}\"\n", variable, arithmetic(start));
        output.push_str(&loop_call(label, check));
        output
    }

    // Body Subroutine
    fn files_loop(
        &mut self,
        label: &LoopLabel,
        variable: &str,
        pattern: &[Part],
        body: String,
        check: ReturnCheck,
    ) -> Result<String, RosellaError> {
        let exiting = if label.exits {
            "        if defined rosella_exit exit /b !rosella_exit!\n"
        } else {
            ""
        };
        let returning = if check == ReturnCheck::None {
            ""
        } else {
            "        if defined rosella_returning goto :eof\n"
        };

        // Dot Names
        let mut first = String::new();
        let shown = match name_start(pattern) {
            NameStart::Dot => "if 1 EQU 1 (".to_string(),
            NameStart::Other => {
                "set \"rosella_name=%%~nxf\"\n    if not \"!rosella_name:~0,1!\"==\".\" ("
                    .to_string()
            }
            NameStart::Variable(name) => {
                // Read Once
                first = format!("set \"{}_dot=!{}:~0,1!\"\n", label.name, name);
                format!(
                    "set \"rosella_name=%%~nxf\"\n    set \"rosella_show=1\"\n    if \"!rosella_name:~0,1!\"==\".\" if not \"!{}_dot!\"==\".\" set \"rosella_show=\"\n    if defined rosella_show (",
                    label.name
                )
            }
        };

        self.uses_keep = true;
        self.subroutines.push_str(&format!(
            ":{label}\nset \"rosella_break=\"\n{first}{guard}for %%f in (\"{pattern}\") do (\n    {shown}\n        call :rosella_keep {variable}\n        call :{label}_body\n{exiting}        if defined rosella_break (set \"rosella_break=\" & goto :eof)\n{returning}    )\n)\ngoto :eof\n\n:{label}_body\n{body}goto :eof\n\n",
            label = label.name,
            first = first,
            guard = wildcard_guard(pattern),
            pattern = quoted(pattern)?,
            shown = shown,
            variable = variable,
            exiting = exiting,
            returning = returning,
            body = body,
        ));

        Ok(loop_call(label, check))
    }

    // Argument Variables
    fn function(&mut self, name: &str, parameters: &[String], body: String) -> String {
        let mut output = format!(":{}\n", name);
        for (index, parameter) in parameters.iter().enumerate() {
            output.push_str(&indent(&format!(
                "set \"{}=!rosella_arg{}!\"\n",
                parameter,
                index + 1
            )));
        }
        output.push_str(&body);

        let jump = indent("goto :eof\n");
        if !body.ends_with(&jump) {
            output.push_str(&jump);
        }
        output.push('\n');

        self.subroutines.push_str(&output);
        String::new()
    }

    fn call(&self, name: &str, args: &[Arg], exits: bool) -> Result<String, RosellaError> {
        let mut output = String::new();
        for (index, arg) in args.iter().enumerate() {
            output.push_str(&match arg {
                Arg::Int(value) => format!(
                    "set /a \"rosella_arg{}={}\"\n",
                    index + 1,
                    arithmetic(value)
                ),
                Arg::Value(parts) => {
                    format!("set \"rosella_arg{}={}\"\n", index + 1, quoted(parts)?)
                }
            });
        }
        output.push_str(&call(name, exits));
        Ok(output)
    }

    fn capture(&self, temporary: &str, _local: bool) -> String {
        format!("set \"{}=!rosella_return!\"\n", temporary)
    }

    fn declare_local(&self, _name: &str) -> String {
        String::new()
    }

    // Leave Loops
    fn return_from_function(&self, nested_in_loop: bool) -> String {
        if nested_in_loop {
            "set \"rosella_returning=1\"\ngoto :eof\n".to_string()
        } else {
            "goto :eof\n".to_string()
        }
    }

    fn print(&self, text: &[Part]) -> Result<String, RosellaError> {
        Ok(split_lines(text)
            .iter()
            .map(|line| format!("echo({}\n", unquoted(line)))
            .collect())
    }

    fn cd(&self, path: &[Part]) -> Result<String, RosellaError> {
        Ok(format!(
            "{}cd /d \"{}\"\n",
            wildcard_guard(path),
            quoted(path)?
        ))
    }

    fn make_dir(&self, path: &[Part]) -> Result<String, RosellaError> {
        let path = quoted(path)?;
        Ok(format!("if not exist \"{}\" mkdir \"{}\"\n", path, path))
    }

    // Empty Guard
    fn remove(&self, path: &[Part], directory: bool) -> Result<String, RosellaError> {
        let mut output = String::new();
        for part in path {
            if let Part::Var(name) = part {
                output.push_str(&format!(
                    "if not defined {name} (>&2 echo(Stopped: '{name}' is empty in a path to remove& set \"rosella_exit=1\"& exit /b 1)\n",
                    name = name
                ));
            }
        }

        // Kind Check
        let command = if directory {
            "if exist \"{path}\\\" rmdir /s /q \"{path}\""
        } else {
            "if exist \"{path}\" if not exist \"{path}\\\" del /f /q \"{path}\""
        };
        output.push_str(&format!(
            "{}{}\n",
            wildcard_guard(path),
            command.replace("{path}", &quoted(path)?)
        ));
        Ok(output)
    }

    fn transfer(
        &self,
        transfer: Transfer,
        source: &[Part],
        destination: &[Part],
    ) -> Result<String, RosellaError> {
        let source_path = quoted(source)?;
        // Skip Folders
        let command = match transfer {
            Transfer::Copy => format!("if not exist \"{}\\\" copy", source_path),
            Transfer::Move => "move".to_string(),
        };
        let both: Vec<Part> = source.iter().chain(destination).cloned().collect();
        Ok(format!(
            "{}{} /y \"{}\" \"{}\" >nul\n",
            wildcard_guard(&both),
            command,
            source_path,
            quoted(destination)?
        ))
    }

    fn write(&self, path: &[Part], content: &[Part], append: bool) -> Result<String, RosellaError> {
        let path = quoted(path)?;
        let mut output = String::new();

        // Redirect First
        for (index, line) in split_lines(content).iter().enumerate() {
            let operator = if index == 0 && !append { ">" } else { ">>" };
            output.push_str(&format!(
                "{}\"{}\" echo({}\n",
                operator,
                path,
                unquoted(line)
            ));
        }

        Ok(output)
    }

    // Clear Before Prompt
    fn read(&self, prompt: &[Part], variable: &str, _local: bool) -> Result<String, RosellaError> {
        Ok(format!(
            "set \"{variable}=\"\nset /p \"{variable}={prompt}\"\n",
            variable = variable,
            prompt = quoted(prompt)?
        ))
    }

    fn exit(&self, code: &Arith, depth: usize) -> String {
        exit(&arithmetic(code), depth)
    }

    fn argument(&mut self, index: &Arith, target: &str, _local: bool) -> String {
        self.uses_arguments = true;

        match index.literal {
            Some(number) => format!("set \"{}=!rosella_arg_{}!\"\n", target, number),
            None => format!(
                "set /a \"rosella_index={}\"\nset \"{target}=\"\nfor %%i in (!rosella_index!) do set \"{target}=!rosella_arg_%%i!\"\n",
                arithmetic(index),
                target = target
            ),
        }
    }

    fn argument_count(&mut self, target: &str, _local: bool) -> String {
        self.uses_arguments = true;
        format!("set \"{}=!rosella_argc!\"\n", target)
    }

    fn run(&mut self, command: &[Vec<Part>]) -> Result<String, RosellaError> {
        self.command(command, "")
    }

    fn run_output(
        &mut self,
        command: &[Vec<Part>],
        target: &str,
        _local: bool,
    ) -> Result<String, RosellaError> {
        self.uses_keep = true;
        self.uses_capture = true;
        let mut output = self.command(command, " >\"%rosella_capture%\"")?;

        // First Text Line
        output.push_str(&format!(
            "set \"{target}=\"\nfor /f usebackq^ delims^=^ eol^= %%f in (\"!rosella_capture!\") do if not defined {target} call :rosella_keep {target}\ndel \"!rosella_capture!\"\n",
            target = target
        ));
        Ok(output)
    }

    fn capture_status(&self, target: &str, _local: bool) -> String {
        format!("set \"{}=!errorlevel!\"\n", target)
    }

    fn set_env(&self, name: &str, value: &[Part]) -> Result<String, RosellaError> {
        Ok(format!("set \"{}={}\"\n", name, quoted(value)?))
    }

    fn length(
        &mut self,
        text: &[Part],
        target: &str,
        _local: bool,
    ) -> Result<String, RosellaError> {
        self.uses_length = true;
        let (setup, source) = source(text, "rosella_measure")?;
        Ok(format!(
            "{}call :rosella_length {} {}\n",
            setup, target, source
        ))
    }

    fn slice(
        &mut self,
        text: &[Part],
        start: &Arith,
        count: &Arith,
        target: &str,
        _local: bool,
    ) -> Result<String, RosellaError> {
        let (mut output, source) = source(text, "rosella_text")?;
        let start_value = self.int_number(start, &mut output);
        let count_value = self.int_number(count, &mut output);
        output.push_str(&format!("set \"{}=\"\n", target));

        // Variable Positions
        let slice = match (start.literal, count.literal) {
            (Some(_), Some(_)) => format!(
                "set \"{target}=!{source}:~{start},{count}!\"",
                target = target,
                source = source,
                start = start_value,
                count = count_value
            ),
            _ => format!(
                "for /f \"tokens=1,2\" %%a in (\"{start} {count}\") do set \"{target}=!{source}:~%%a,%%b!\"",
                target = target,
                source = source,
                start = start_value,
                count = count_value
            ),
        };
        output.push_str(&format!("if defined {} {}\n", source, slice));
        Ok(output)
    }

    fn replace(
        &mut self,
        text: &[Part],
        from: &[Part],
        to: &[Part],
        target: &str,
        _local: bool,
    ) -> Result<String, RosellaError> {
        self.uses_replace = true;
        Ok(format!(
            "set \"rosella_text={}\"\nset \"rosella_from={}\"\nset \"rosella_to={}\"\ncall :rosella_replace {}\n",
            quoted(text)?,
            quoted(from)?,
            quoted(to)?,
            target
        ))
    }

    fn change_case(
        &mut self,
        text: &[Part],
        upper: bool,
        target: &str,
        _local: bool,
    ) -> Result<String, RosellaError> {
        let letters = if upper {
            "A B C D E F G H I J K L M N O P Q R S T U V W X Y Z"
        } else {
            "a b c d e f g h i j k l m n o p q r s t u v w x y z"
        };
        Ok(format!(
            "set \"{target}={text}\"\nif defined {target} for %%c in ({letters}) do set \"{target}=!{target}:%%c=%%c!\"\n",
            target = target,
            text = quoted(text)?,
            letters = letters
        ))
    }

    fn random(&mut self, min: &Arith, max: &Arith, target: &str, _local: bool) -> String {
        let min = min.operand().replace('%', "%%");
        let max = max.operand().replace('%', "%%");
        format!(
            "set /a \"{target}={min} + !random! %% ({max} - {min} + 1)\"\n",
            target = target,
            min = min,
            max = max
        )
    }

    // Ping Delay
    fn sleep(&self, seconds: &Arith) -> String {
        format!(
            "set /a \"rosella_sleep={} + 1\"\nping -n !rosella_sleep! 127.0.0.1 >nul\n",
            arithmetic(seconds)
        )
    }

    fn use_script_dir(&mut self) {
        self.uses_script_dir = true;
    }
}

// Helper Functions

const LENGTH_HELPER: &str = concat!(
    ":rosella_length\n",
    "set \"rosella_scan=!%~2!#\"\n",
    "set \"rosella_size=0\"\n",
    "for %%n in (4096 2048 1024 512 256 128 64 32 16 8 4 2 1) do (\n",
    "    if \"!rosella_scan:~%%n,1!\" NEQ \"\" (\n",
    "        set /a \"rosella_size+=%%n\"\n",
    "        set \"rosella_scan=!rosella_scan:~%%n!\"\n",
    "    )\n",
    ")\n",
    "set \"%~1=!rosella_size!\"\n",
    "goto :eof\n\n",
);

const REPLACE_HELPER: &str = concat!(
    ":rosella_replace\n",
    "set \"rosella_result=\"\n",
    "if not defined rosella_from set \"rosella_result=!rosella_text!\"\n",
    "if not defined rosella_from goto :rosella_replace_done\n",
    "call :rosella_length rosella_from_size rosella_from\n",
    ":rosella_replace_loop\n",
    "if not defined rosella_text goto :rosella_replace_done\n",
    "for %%n in (!rosella_from_size!) do set \"rosella_head=!rosella_text:~0,%%n!\"\n",
    "if \"!rosella_head!\"==\"!rosella_from!\" (\n",
    "    set \"rosella_result=!rosella_result!!rosella_to!\"\n",
    "    for %%n in (!rosella_from_size!) do set \"rosella_text=!rosella_text:~%%n!\"\n",
    ") else (\n",
    "    set \"rosella_result=!rosella_result!!rosella_text:~0,1!\"\n",
    "    set \"rosella_text=!rosella_text:~1!\"\n",
    ")\n",
    "goto :rosella_replace_loop\n",
    ":rosella_replace_done\n",
    "set \"%~1=!rosella_result!\"\n",
    "goto :eof\n\n",
);

const CONTAINS_HELPER: &str = concat!(
    ":rosella_contains\n",
    "set \"rosella_found=\"\n",
    "set \"rosella_original=!rosella_text!\"\n",
    "set \"rosella_to=\"\n",
    "call :rosella_replace rosella_result\n",
    "if not defined rosella_from set \"rosella_found=1\"\n",
    "if not \"!rosella_result!\"==\"!rosella_original!\" set \"rosella_found=1\"\n",
    "goto :eof\n\n",
);

const KEEP_HELPER: &str = concat!(
    ":rosella_keep\n",
    "setlocal disabledelayedexpansion\n",
    "for %%z in (1) do set \"rosella_raw=%%f\"\n",
    "set \"rosella_raw=%rosella_raw:\"=\"\"%\"\n",
    "if \"%rosella_raw:!=%\"==\"%rosella_raw%\" goto :rosella_keep_done\n",
    "set \"rosella_raw=%rosella_raw:^=^^%\"\n",
    "set \"rosella_raw=%rosella_raw:!=^!%\"\n",
    ":rosella_keep_done\n",
    "for /f delims^=^ eol^= %%v in (\"%rosella_raw%\") do endlocal & set \"%~1=%%v\"\n",
    "set \"%~1=!%~1:\"\"=\"!\"\n",
    "goto :eof\n\n",
);

enum NameStart {
    Dot,
    Other,
    Variable(String),
}

fn name_start(pattern: &[Part]) -> NameStart {
    let mut start = NameStart::Other;
    let mut at_start = true;

    for part in pattern {
        match part {
            Part::Text(text) => {
                for ch in text.chars() {
                    if ch == '\\' {
                        start = NameStart::Other;
                        at_start = true;
                    } else if at_start {
                        start = if ch == '.' {
                            NameStart::Dot
                        } else {
                            NameStart::Other
                        };
                        at_start = false;
                    }
                }
            }
            Part::Var(name) if at_start => {
                start = NameStart::Variable(name.clone());
                at_start = false;
            }
            _ => at_start = false,
        }
    }

    start
}

const SLASHES_HELPER: &str = concat!(
    ":rosella_slashes\n",
    "set \"rosella_tail=!%~1!\"\n",
    "set \"rosella_extra=\"\n",
    ":rosella_slashes_loop\n",
    "if \"!rosella_tail:~-1!\"==\"\\\" (\n",
    "    set \"rosella_extra=!rosella_extra!\\\"\n",
    "    set \"rosella_tail=!rosella_tail:~0,-1!\"\n",
    "    goto :rosella_slashes_loop\n",
    ")\n",
    "set \"%~1=!%~1!!rosella_extra!\"\n",
    "goto :eof\n\n",
);

fn wildcard_guard(path: &[Part]) -> String {
    let values: String = path
        .iter()
        .filter(|part| matches!(part, Part::Var(_)))
        .map(variable)
        .collect();
    if values.is_empty() {
        return String::new();
    }

    // Split Check
    format!(
        "set \"rosella_wild=\" & for /f \"tokens=2 delims=*?\" %%w in (\"x{values}x\") do set \"rosella_wild=1\"\nif not defined rosella_wild ",
        values = values
    )
}

// Direct Variable
fn source(text: &[Part], temporary: &str) -> Result<(String, String), RosellaError> {
    match text {
        [Part::Var(name)] => Ok((String::new(), name.clone())),
        _ => Ok((
            format!("set \"{}={}\"\n", temporary, quoted(text)?),
            temporary.to_string(),
        )),
    }
}

fn negate(condition: Condition) -> Condition {
    let test = match condition.test.strip_prefix("not ") {
        Some(test) => test.to_string(),
        None => format!("not {}", condition.test),
    };
    Condition {
        setup: condition.setup,
        test,
    }
}

// Double Percents
fn arithmetic(value: &Arith) -> String {
    value.text.replace('%', "%%")
}

fn loop_call(label: &LoopLabel, check: ReturnCheck) -> String {
    let mut output = call(&label.name, label.exits);
    output.push_str(match check {
        ReturnCheck::None => "",
        ReturnCheck::Propagate => "if defined rosella_returning goto :eof\n",
        ReturnCheck::Clear => {
            "if defined rosella_returning (set \"rosella_returning=\" & goto :eof)\n"
        }
    });
    output
}

// Exit Check
fn call(label: &str, exits: bool) -> String {
    if exits {
        format!(
            "call :{}\nif defined rosella_exit exit /b !rosella_exit!\n",
            label
        )
    } else {
        format!("call :{}\n", label)
    }
}

// Exit Flag
fn exit(code: &str, depth: usize) -> String {
    match (depth, code.parse::<i32>()) {
        (0, Ok(code)) => format!("exit /b {}\n", code),
        (0, _) => format!("set /a \"rosella_exit={}\"\nexit /b !rosella_exit!\n", code),
        _ => format!("set /a \"rosella_exit={}\"\ngoto :eof\n", code),
    }
}

// Caret Rescan
fn has_bang(parts: &[Part]) -> bool {
    parts.iter().any(|part| match part {
        Part::Text(text) => text.contains('!'),
        Part::Var(_) | Part::Cwd => true,
    })
}

fn variable(part: &Part) -> String {
    match part {
        Part::Var(name) => format!("!{}!", name),
        _ => "!CD!".to_string(),
    }
}

fn quoted(parts: &[Part]) -> Result<String, RosellaError> {
    let bang = has_bang(parts);
    let mut output = String::new();

    for part in parts {
        let Part::Text(text) = part else {
            output.push_str(&variable(part));
            continue;
        };

        for ch in text.chars() {
            match ch {
                '"' => return Err(error("Batch cannot use '\"' inside a quoted value")),
                '\n' => {
                    return Err(error(
                        "Batch cannot keep a line break in a variable, argument or path",
                    ));
                }
                '%' => output.push_str("%%"),
                '!' => output.push_str("^!"),
                '^' if bang => output.push_str("^^"),
                _ => output.push(ch),
            }
        }
    }

    Ok(output)
}

fn unquoted(parts: &[Part]) -> String {
    let bang = has_bang(parts);
    let mut output = String::new();

    for part in parts {
        let Part::Text(text) = part else {
            output.push_str(&variable(part));
            continue;
        };

        for ch in text.chars() {
            match ch {
                '%' => output.push_str("%%"),
                '!' => output.push_str("^^!"),
                '^' if bang => output.push_str("^^^^"),
                '^' => output.push_str("^^"),
                '&' | '|' | '<' | '>' | '(' | ')' | '"' => {
                    output.push('^');
                    output.push(ch);
                }
                _ => output.push(ch),
            }
        }
    }

    output
}

fn split_lines(parts: &[Part]) -> Vec<Vec<Part>> {
    let mut lines: Vec<Vec<Part>> = vec![Vec::new()];

    for part in parts {
        match part {
            Part::Text(text) => {
                for (index, piece) in text.split('\n').enumerate() {
                    if index > 0 {
                        lines.push(Vec::new());
                    }
                    if !piece.is_empty() {
                        lines
                            .last_mut()
                            .unwrap()
                            .push(Part::Text(piece.to_string()));
                    }
                }
            }
            other => lines.last_mut().unwrap().push(other.clone()),
        }
    }

    lines
}
