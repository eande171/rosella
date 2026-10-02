use crate::syntax::Type;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Builtin {
    Print,
    Cd,
    MakeDir,
    Remove,
    RemoveDir,
    Copy,
    Move,
    WriteFile,
    AppendFile,
    Exit,
    SetEnv,
    Sleep,
    Read,
    Run,
    Output,
    Arg,
    ArgCount,
    Env,
    Path,
    Concat,
    GetCwd,
    Exists,
    NotExists,
    IsDir,
    IsFile,
    Contains,
    Length,
    Slice,
    Replace,
    Upper,
    Lower,
    Random,
    ScriptDir,
}

#[derive(Debug, Clone, Copy)]
pub enum Arity {
    Exactly(usize),
    AtLeast(usize),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    Statement,
    Value(Type),
    // Returns A Value That Can Be Ignored
    Action(Type),
}

pub struct Signature {
    pub name: &'static str,
    pub builtin: Builtin,
    pub arity: Arity,
    pub kind: Kind,
    pub usage: &'static str,
}

impl Signature {
    pub fn accepts(&self, count: usize) -> bool {
        match self.arity {
            Arity::Exactly(expected) => count == expected,
            Arity::AtLeast(minimum) => count >= minimum,
        }
    }

    pub fn returns(&self) -> Option<Type> {
        match self.kind {
            Kind::Statement => None,
            Kind::Value(returns) | Kind::Action(returns) => Some(returns),
        }
    }
}

const fn statement(
    name: &'static str,
    builtin: Builtin,
    arity: Arity,
    usage: &'static str,
) -> Signature {
    Signature {
        name,
        builtin,
        arity,
        kind: Kind::Statement,
        usage,
    }
}

const fn action(
    name: &'static str,
    builtin: Builtin,
    arity: Arity,
    returns: Type,
    usage: &'static str,
) -> Signature {
    Signature {
        name,
        builtin,
        arity,
        kind: Kind::Action(returns),
        usage,
    }
}

const fn value(
    name: &'static str,
    builtin: Builtin,
    arity: Arity,
    returns: Type,
    usage: &'static str,
) -> Signature {
    Signature {
        name,
        builtin,
        arity,
        kind: Kind::Value(returns),
        usage,
    }
}

#[rustfmt::skip]
const SIGNATURES: &[Signature] = &[
    statement("print",       Builtin::Print,      Arity::AtLeast(0), "print(\"Hello \", name)"),
    statement("cd",          Builtin::Cd,         Arity::AtLeast(1), "cd(\"folder\")"),
    statement("make_dir",    Builtin::MakeDir,    Arity::AtLeast(1), "make_dir(\"folder\")"),
    statement("remove",      Builtin::Remove,     Arity::AtLeast(1), "remove(\"file.txt\")"),
    statement("remove_dir",  Builtin::RemoveDir,  Arity::AtLeast(1), "remove_dir(\"folder\")"),
    statement("copy",        Builtin::Copy,       Arity::Exactly(2), "copy(\"from.txt\", \"to.txt\")"),
    statement("move",        Builtin::Move,       Arity::Exactly(2), "move(\"from.txt\", \"to.txt\")"),
    statement("write_file",  Builtin::WriteFile,  Arity::Exactly(2), "write_file(\"notes.txt\", text)"),
    statement("append_file", Builtin::AppendFile, Arity::Exactly(2), "append_file(\"notes.txt\", text)"),
    statement("exit",        Builtin::Exit,       Arity::Exactly(1), "exit(1)"),
    statement("sleep",       Builtin::Sleep,      Arity::Exactly(1), "sleep(2)"),
    statement("set_env",     Builtin::SetEnv,     Arity::Exactly(2), "set_env(\"MODE\", \"release\")"),
    action("read",       Builtin::Read,      Arity::Exactly(1), Type::Str,  "let str name = read(\"Name: \");"),
    action("run",        Builtin::Run,       Arity::AtLeast(1), Type::Int,  "run(\"git\", \"pull\")"),
    value("output",      Builtin::Output,    Arity::AtLeast(1), Type::Str,  "output(\"git\", \"branch\", \"--show-current\")"),
    value("arg",         Builtin::Arg,       Arity::Exactly(1), Type::Str,  "arg(1)"),
    value("arg_count",   Builtin::ArgCount,  Arity::Exactly(0), Type::Int,  "arg_count()"),
    value("env",         Builtin::Env,       Arity::Exactly(1), Type::Str,  "env(\"HOME\")"),
    value("path",        Builtin::Path,      Arity::AtLeast(1), Type::Str,  "path(\"folder\", name)"),
    value("concat",      Builtin::Concat,    Arity::AtLeast(1), Type::Str,  "concat(\"Hello \", name)"),
    value("get_cwd",     Builtin::GetCwd,    Arity::Exactly(0), Type::Str,  "get_cwd()"),
    value("exists",      Builtin::Exists,    Arity::AtLeast(1), Type::Check, "exists(\"notes.txt\")"),
    value("not_exists",  Builtin::NotExists, Arity::AtLeast(1), Type::Check, "not_exists(\"notes.txt\")"),
    value("is_dir",      Builtin::IsDir,     Arity::AtLeast(1), Type::Check, "is_dir(\"folder\")"),
    value("is_file",     Builtin::IsFile,    Arity::AtLeast(1), Type::Check, "is_file(\"notes.txt\")"),
    value("contains",    Builtin::Contains,  Arity::Exactly(2), Type::Check, "contains(name, \"Bob\")"),
    value("length",      Builtin::Length,    Arity::Exactly(1), Type::Int,   "length(name)"),
    value("slice",       Builtin::Slice,     Arity::Exactly(3), Type::Str,   "slice(name, 0, 3)"),
    value("replace",     Builtin::Replace,   Arity::Exactly(3), Type::Str,   "replace(name, \"old\", \"new\")"),
    value("upper",       Builtin::Upper,     Arity::Exactly(1), Type::Str,   "upper(name)"),
    value("lower",       Builtin::Lower,     Arity::Exactly(1), Type::Str,   "lower(name)"),
    value("random",      Builtin::Random,    Arity::Exactly(2), Type::Int,   "random(1, 6)"),
    value("script_dir",  Builtin::ScriptDir, Arity::Exactly(0), Type::Str,   "script_dir()"),
];

pub fn find(name: &str) -> Option<&'static Signature> {
    SIGNATURES.iter().find(|signature| signature.name == name)
}
