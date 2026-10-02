use crate::error::RosellaError;

const INTERNAL_PREFIX: &str = "rosella_";

// Shell Environment Variables
const BATCH_RESERVED: &[&str] = &[
    "path",
    "pathext",
    "comspec",
    "cd",
    "errorlevel",
    "random",
    "date",
    "time",
    "temp",
    "tmp",
    "os",
    "prompt",
    "systemroot",
    "windir",
    "username",
    "userprofile",
    "appdata",
    "cmdcmdline",
    "cmdextversion",
];
const BASH_RESERVED: &[&str] = &[
    "PATH", "HOME", "PWD", "OLDPWD", "IFS", "SHELL", "USER", "UID", "EUID", "PPID", "RANDOM",
    "SECONDS", "LINENO", "HOSTNAME", "BASH",
];

// Bash Would Call These Instead Of The Real Commands
const SHELL_COMMANDS: &[&str] = &[
    "printf", "read", "mkdir", "rm", "cp", "mv", "cd", "export", "local", "return", "exit",
    "sleep", "dirname", "pwd", "command", "builtin", "set", "shift", "test", "echo", "eval",
    "exec", "source", "declare", "unset", "true", "false", "then", "fi", "do", "done", "elif",
    "case", "esac", "until", "select", "function", "time",
];

pub fn check_shell_command(name: &str) -> Result<(), RosellaError> {
    if SHELL_COMMANDS.contains(&name) {
        return Err(RosellaError::compiler(format!(
            "A function named '{}' would replace a command the generated script needs; choose another name",
            name
        )));
    }
    Ok(())
}

pub fn check_function_name(name: &str) -> Result<(), RosellaError> {
    if name.to_lowercase().starts_with(INTERNAL_PREFIX) {
        return Err(RosellaError::compiler(format!(
            "Names starting with '{}' are reserved for the compiler",
            INTERNAL_PREFIX
        )));
    }
    Ok(())
}

pub fn check_variable_name(name: &str) -> Result<(), RosellaError> {
    check_function_name(name)?;
    if BATCH_RESERVED.contains(&name.to_lowercase().as_str()) || BASH_RESERVED.contains(&name) {
        return Err(RosellaError::compiler(format!(
            "'{}' would overwrite an environment variable of the shell; choose another name",
            name
        )));
    }
    Ok(())
}
