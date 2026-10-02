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
    Read,
    Exit,
    Path,
    Concat,
    GetCwd,
    Exists,
    NotExists,
}

#[derive(Debug, Clone, Copy)]
pub enum Arity {
    Exactly(usize),
    AtLeast(usize),
}

pub struct Signature {
    pub name: &'static str,
    pub builtin: Builtin,
    pub arity: Arity,
    pub returns: Option<Type>,
    pub usage: &'static str,
}

impl Signature {
    pub fn accepts(&self, count: usize) -> bool {
        match self.arity {
            Arity::Exactly(expected) => count == expected,
            Arity::AtLeast(minimum) => count >= minimum,
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
        returns: None,
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
        returns: Some(returns),
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
    statement("read",        Builtin::Read,       Arity::Exactly(2), "read(\"Name: \", name)"),
    statement("exit",        Builtin::Exit,       Arity::Exactly(1), "exit(1)"),
    value("path",        Builtin::Path,      Arity::AtLeast(1), Type::Str,  "path(\"folder\", name)"),
    value("concat",      Builtin::Concat,    Arity::AtLeast(1), Type::Str,  "concat(\"Hello \", name)"),
    value("get_cwd",     Builtin::GetCwd,    Arity::Exactly(0), Type::Str,  "get_cwd()"),
    value("exists",      Builtin::Exists,    Arity::AtLeast(1), Type::File, "exists(\"notes.txt\")"),
    value("not_exists",  Builtin::NotExists, Arity::AtLeast(1), Type::File, "not_exists(\"notes.txt\")"),
];

pub fn find(name: &str) -> Option<&'static Signature> {
    SIGNATURES.iter().find(|signature| signature.name == name)
}
