use super::backend::{
    Arg, Arith, Backend, Condition, FileCheck, Logic, LoopKind, LoopLabel, Part, ReturnCheck, Test,
    Transfer,
};
use super::indent;
use crate::error::RosellaError;
use crate::syntax::BinaryOp;

#[derive(Default)]
pub struct Bash {
    functions: String,
    uses_arguments: bool,
    uses_script_dir: bool,
}

impl Backend for Bash {
    fn program(&mut self, body: String) -> String {
        // Functions Have Their Own Arguments
        let arguments = if self.uses_arguments {
            "rosella_args=(\"$@\")\n"
        } else {
            ""
        };
        let script_dir = if self.uses_script_dir {
            "rosella_script_dir=\"$(cd \"$(dirname \"${BASH_SOURCE[0]}\")\" && pwd)\"\n"
        } else {
            ""
        };
        // Functions First So Calls Work From Anywhere
        format!(
            "#!/bin/bash\n{}{}{}{}",
            script_dir, arguments, self.functions, body
        )
    }

    fn empty_body(&self) -> &'static str {
        ":\n"
    }

    // Called Functions Would Otherwise See These Locals
    fn local_name(&self, function: &str, parameter: &str) -> String {
        format!("rosella_{}_{}", function, parameter)
    }

    fn assign_int(&self, name: &str, value: &Arith) -> String {
        match value.literal {
            Some(literal) => format!("{}={}\n", name, literal),
            None => format!("{}=$(( {} ))\n", name, value.text),
        }
    }

    fn assign_str(&self, name: &str, value: &[Part]) -> Result<String, RosellaError> {
        Ok(format!("{}={}\n", name, quote(value, false)))
    }

    fn condition(&mut self, logic: Logic) -> Result<Condition, RosellaError> {
        Ok(Condition {
            setup: String::new(),
            test: render(&logic, true),
        })
    }

    fn if_chain(&self, branches: Vec<(Condition, String)>, otherwise: Option<String>) -> String {
        let mut output = String::new();

        for (index, (condition, body)) in branches.into_iter().enumerate() {
            let keyword = if index == 0 { "if" } else { "elif" };
            output.push_str(&format!("{} {}; then\n{}", keyword, condition.test, body));
        }
        if let Some(otherwise) = otherwise {
            output.push_str(&format!("else\n{}", otherwise));
        }

        output.push_str("fi\n");
        output
    }

    fn begin_loop(&mut self, kind: LoopKind, exits: bool) -> LoopLabel {
        LoopLabel {
            name: String::new(),
            kind,
            exits,
        }
    }

    fn while_loop(
        &mut self,
        _label: &LoopLabel,
        condition: Condition,
        body: String,
        _check: ReturnCheck,
    ) -> String {
        format!("while {}; do\n{}done\n", condition.test, body)
    }

    fn range_loop(
        &mut self,
        _label: &LoopLabel,
        variable: &str,
        start: &Arith,
        end: &Arith,
        step: i64,
        body: String,
        _check: ReturnCheck,
    ) -> String {
        let (comparison, change) = if step > 0 {
            ("<", format!("+= {}", step))
        } else {
            (">", format!("-= {}", -step))
        };
        format!(
            "for (( {variable} = {start}; {variable} {comparison} {end}; {variable} {change} )); do\n{body}done\n",
            variable = variable,
            start = start.text,
            comparison = comparison,
            end = end.operand(),
            change = change,
            body = body
        )
    }

    // Matches Are Files Only
    fn files_loop(
        &mut self,
        _label: &LoopLabel,
        variable: &str,
        pattern: &[Part],
        body: String,
        _check: ReturnCheck,
    ) -> Result<String, RosellaError> {
        Ok(format!(
            "for {variable} in {pattern}; do\n    [[ -f \"${{{variable}}}\" ]] || continue\n{body}done\n",
            variable = variable,
            pattern = glob(pattern),
            body = body
        ))
    }

    fn break_loop(&self, _label: &LoopLabel) -> String {
        "break\n".to_string()
    }

    fn continue_loop(&self, _label: &LoopLabel) -> String {
        "continue\n".to_string()
    }

    fn function(&mut self, name: &str, parameters: &[String], body: String) -> String {
        let mut output = format!("{}() {{\n", name);
        for (index, parameter) in parameters.iter().enumerate() {
            output.push_str(&indent(&format!(
                "local {}=\"${{{}}}\"\n",
                parameter,
                index + 1
            )));
        }
        // Final Return Is Implied
        let body = body.strip_suffix(&indent("return\n")).unwrap_or(&body);
        if parameters.is_empty() && body.is_empty() {
            output.push_str(&indent(self.empty_body()));
        }
        output.push_str(body);
        output.push_str("}\n");

        self.functions.push_str(&output);
        String::new()
    }

    fn call(&self, name: &str, args: &[Arg], _exits: bool) -> Result<String, RosellaError> {
        let mut output = name.to_string();
        for arg in args {
            output.push(' ');
            output.push_str(&match arg {
                Arg::Int(value) => format!("\"$(( {} ))\"", value.text),
                Arg::Value(parts) => quote(parts, false),
            });
        }
        output.push('\n');
        Ok(output)
    }

    fn capture(&self, temporary: &str, local: bool) -> String {
        let keyword = if local { "local " } else { "" };
        format!("{}{}=\"${{rosella_return}}\"\n", keyword, temporary)
    }

    fn declare_local(&self, name: &str) -> String {
        format!("local {}\n", name)
    }

    fn return_from_function(&self, _nested_in_loop: bool) -> String {
        "return\n".to_string()
    }

    fn print(&self, text: &[Part]) -> Result<String, RosellaError> {
        Ok(format!("printf '%s\\n' {}\n", quote(text, false)))
    }

    fn cd(&self, path: &[Part]) -> Result<String, RosellaError> {
        Ok(format!("cd {}\n", quote(path, false)))
    }

    fn make_dir(&self, path: &[Part]) -> Result<String, RosellaError> {
        Ok(format!("mkdir -p -- {}\n", quote(path, false)))
    }

    fn remove(&self, path: &[Part], directory: bool) -> Result<String, RosellaError> {
        let command = if directory { "rm -rf" } else { "rm -f" };
        Ok(format!("{} -- {}\n", command, quote(path, true)))
    }

    fn transfer(
        &self,
        transfer: Transfer,
        source: &[Part],
        destination: &[Part],
    ) -> Result<String, RosellaError> {
        let command = match transfer {
            Transfer::Copy => "cp",
            Transfer::Move => "mv",
        };
        Ok(format!(
            "{} -- {} {}\n",
            command,
            quote(source, false),
            quote(destination, false)
        ))
    }

    fn write(&self, path: &[Part], content: &[Part], append: bool) -> Result<String, RosellaError> {
        Ok(format!(
            "printf '%s\\n' {} {} {}\n",
            quote(content, false),
            if append { ">>" } else { ">" },
            quote(path, false)
        ))
    }

    fn read(&self, prompt: &[Part], variable: &str, local: bool) -> Result<String, RosellaError> {
        let declare = if local {
            format!("local {}\n", variable)
        } else {
            String::new()
        };
        Ok(format!(
            "{}read -r -p {} {}\n",
            declare,
            quote(prompt, false),
            variable
        ))
    }

    fn exit(&self, code: &Arith, _depth: usize) -> String {
        match code.literal {
            Some(literal) => format!("exit {}\n", literal),
            None => format!("exit $(( {} ))\n", code.text),
        }
    }

    fn argument(&mut self, index: &Arith, target: &str, local: bool) -> String {
        self.uses_arguments = true;
        let keyword = if local { "local " } else { "" };

        match index.literal {
            Some(number) => format!(
                "{}{}=\"${{rosella_args[{}]}}\"\n",
                keyword,
                target,
                number - 1
            ),
            // Negative Indexes Count From The End
            None => format!(
                "{keyword}{target}=\"\"\nif (( {index} >= 1 )); then {target}=\"${{rosella_args[{index} - 1]}}\"; fi\n",
                keyword = keyword,
                target = target,
                index = index.operand()
            ),
        }
    }

    fn argument_count(&mut self, target: &str, local: bool) -> String {
        self.uses_arguments = true;
        let keyword = if local { "local " } else { "" };
        format!("{}{}=${{#rosella_args[@]}}\n", keyword, target)
    }

    // Command Skips Functions With The Same Name
    fn run(&self, command: &[Vec<Part>]) -> Result<String, RosellaError> {
        let words: Vec<String> = command.iter().map(|part| quote(part, false)).collect();
        Ok(format!("command {}\n", words.join(" ")))
    }

    fn capture_status(&self, target: &str, local: bool) -> String {
        let keyword = if local { "local " } else { "" };
        format!("{}{}=$?\n", keyword, target)
    }

    // First Line With Text Like Batch
    fn output(
        &self,
        command: &[Vec<Part>],
        target: &str,
        local: bool,
    ) -> Result<String, RosellaError> {
        let words: Vec<String> = command.iter().map(|part| quote(part, false)).collect();
        let keyword = if local { "local " } else { "" };
        Ok(format!(
            "{keyword}rosella_line=\"\"\n{keyword}{target}=\"\"\nwhile IFS= read -r rosella_line; do rosella_line=\"${{rosella_line%$'\\r'}}\"; if [[ -n \"${{rosella_line}}\" ]]; then {target}=\"${{rosella_line}}\"; break; fi; done < <(command {command})\n",
            keyword = keyword,
            target = target,
            command = words.join(" ")
        ))
    }

    fn set_env(&self, name: &str, value: &[Part]) -> Result<String, RosellaError> {
        Ok(format!("export {}={}\n", name, quote(value, false)))
    }

    fn length(&mut self, text: &[Part], target: &str, local: bool) -> Result<String, RosellaError> {
        let keyword = if local { "local " } else { "" };
        let (setup, source) = source(text, keyword);
        Ok(format!(
            "{setup}{keyword}{target}=${{#{source}}}\n",
            setup = setup,
            keyword = keyword,
            target = target,
            source = source
        ))
    }

    fn slice(
        &mut self,
        text: &[Part],
        start: &Arith,
        count: &Arith,
        target: &str,
        local: bool,
    ) -> Result<String, RosellaError> {
        let keyword = if local { "local " } else { "" };
        let (setup, source) = source(text, keyword);
        Ok(format!(
            "{setup}{keyword}{target}=\"${{{source}:{start}:{count}}}\"\n",
            setup = setup,
            keyword = keyword,
            source = source,
            target = target,
            start = start.operand(),
            count = count.operand()
        ))
    }

    // Quotes Keep The Search And Replacement Literal
    fn replace(
        &mut self,
        text: &[Part],
        from: &[Part],
        to: &[Part],
        target: &str,
        local: bool,
    ) -> Result<String, RosellaError> {
        let keyword = if local { "local " } else { "" };
        let (setup, source) = source(text, keyword);
        Ok(format!(
            "{setup}{keyword}{target}=\"${{{source}//{from}/{to}}}\"\n",
            setup = setup,
            keyword = keyword,
            source = source,
            from = quote(from, false),
            to = quote(to, false),
            target = target
        ))
    }

    fn change_case(
        &mut self,
        text: &[Part],
        upper: bool,
        target: &str,
        local: bool,
    ) -> Result<String, RosellaError> {
        let keyword = if local { "local " } else { "" };
        let (setup, source) = source(text, keyword);
        Ok(format!(
            "{setup}{keyword}{target}=\"${{{source}{operator}}}\"\n",
            setup = setup,
            keyword = keyword,
            source = source,
            target = target,
            operator = if upper { "^^" } else { ",," }
        ))
    }

    fn random(&mut self, min: &Arith, max: &Arith, target: &str, local: bool) -> String {
        let keyword = if local { "local " } else { "" };
        format!(
            "{keyword}{target}=$(( {min} + RANDOM % ({max} - {min} + 1) ))\n",
            keyword = keyword,
            target = target,
            min = min.operand(),
            max = max.operand()
        )
    }

    fn sleep(&self, seconds: &Arith) -> String {
        match seconds.literal {
            Some(literal) => format!("sleep {}\n", literal),
            None => format!("sleep $(( {} ))\n", seconds.text),
        }
    }

    fn use_script_dir(&mut self) {
        self.uses_script_dir = true;
    }
}

fn render(logic: &Logic, root: bool) -> String {
    match logic {
        Logic::Test { setup, test } => {
            let test = test_text(test);
            if setup.is_empty() {
                return test;
            }

            // Setup Runs Inside The Condition List
            let commands: String = setup.lines().map(|line| format!("{}; ", line)).collect();
            if root {
                format!("{}{}", commands, test)
            } else {
                format!("{{ {}{}; }}", commands, test)
            }
        }
        Logic::Not(inner) => format!("! {}", grouped(inner, |_| true)),
        Logic::And(left, right) => format!(
            "{} && {}",
            render(left, false),
            grouped(right, |other| matches!(other, Logic::Or(_, _)))
        ),
        Logic::Or(left, right) => format!(
            "{} || {}",
            render(left, false),
            grouped(right, |other| matches!(other, Logic::And(_, _)))
        ),
    }
}

// Bash Reads && And || Left To Right
fn grouped(logic: &Logic, needs_group: fn(&Logic) -> bool) -> String {
    let text = render(logic, false);
    match logic {
        Logic::Test { .. } => text,
        other if needs_group(other) => format!("{{ {}; }}", text),
        _ => text,
    }
}

fn test_text(test: &Test) -> String {
    match test {
        Test::Int {
            left,
            operator,
            right,
        } => format!(
            "(( {} {} {} ))",
            left.operand(),
            symbol(*operator),
            right.operand()
        ),
        Test::Str {
            left,
            operator,
            right,
        } => format!(
            "[[ {} {} {} ]]",
            quote(left, false),
            symbol(*operator),
            quote(right, false)
        ),
        Test::File { check, path } => {
            let operator = match check {
                FileCheck::Exists => "-e",
                FileCheck::Missing => "! -e",
                FileCheck::Directory => "-d",
                FileCheck::File => "-f",
            };
            format!("[[ {} {} ]]", operator, quote(path, false))
        }
        // The Quoted Part Matches Literally
        Test::Contains { text, part } => {
            format!("[[ {} == *{}* ]]", quote(text, false), quote(part, false))
        }
    }
}

// Wildcards Stay Outside The Quotes
fn glob(parts: &[Part]) -> String {
    let mut output = String::new();
    let mut quoted_text = String::new();

    for part in parts {
        match part {
            Part::Text(text) => {
                for ch in text.chars() {
                    if ch == '*' || ch == '?' {
                        if !quoted_text.is_empty() {
                            output.push_str(&quote(
                                &[Part::Text(std::mem::take(&mut quoted_text))],
                                false,
                            ));
                        }
                        output.push(ch);
                    } else {
                        quoted_text.push(ch);
                    }
                }
            }
            other => {
                if !quoted_text.is_empty() {
                    output.push_str(&quote(
                        &[Part::Text(std::mem::take(&mut quoted_text))],
                        false,
                    ));
                }
                output.push_str(&quote(std::slice::from_ref(other), false));
            }
        }
    }

    if !quoted_text.is_empty() {
        output.push_str(&quote(&[Part::Text(quoted_text)], false));
    }
    output
}

// Text Already In A Variable Is Used Directly
fn source(text: &[Part], keyword: &str) -> (String, String) {
    match text {
        [Part::Var(name)] => (String::new(), name.clone()),
        [Part::Cwd] => (String::new(), "PWD".to_string()),
        _ => (
            format!("{}rosella_text={}\n", keyword, quote(text, false)),
            "rosella_text".to_string(),
        ),
    }
}

fn symbol(operator: BinaryOp) -> &'static str {
    match operator {
        BinaryOp::Equal => "==",
        BinaryOp::NotEqual => "!=",
        BinaryOp::LessThan => "<",
        BinaryOp::LessThanEq => "<=",
        BinaryOp::GreaterThan => ">",
        BinaryOp::GreaterThanEq => ">=",
        BinaryOp::Add => "+",
        BinaryOp::Subtract => "-",
        BinaryOp::Multiply => "*",
        BinaryOp::Divide => "/",
        BinaryOp::Modulo => "%",
        BinaryOp::And => "&&",
        BinaryOp::Or => "||",
    }
}

fn quote(parts: &[Part], guard_empty: bool) -> String {
    let mut output = String::from('"');

    for part in parts {
        match part {
            Part::Text(text) => {
                for ch in text.chars() {
                    match ch {
                        // Avoid Literal Line Breaks
                        '\n' => output.push_str("\"$'\\n'\""),
                        '\\' | '"' | '$' | '`' => {
                            output.push('\\');
                            output.push(ch);
                        }
                        _ => output.push(ch),
                    }
                }
            }
            // Stop On Empty Variable
            Part::Var(name) if guard_empty => output.push_str(&format!("${{{}:?}}", name)),
            Part::Var(name) => output.push_str(&format!("${{{}}}", name)),
            Part::Cwd => output.push_str("${PWD}"),
        }
    }

    output.push('"');
    output
}
