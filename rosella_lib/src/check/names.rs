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
    "copycmd",
    "dircmd",
];
const BASH_RESERVED: &[&str] = &[
    "_",
    "BASH",
    "BASHOPTS",
    "BASHPID",
    "CDPATH",
    "CHILD_MAX",
    "COLUMNS",
    "COMPREPLY",
    "COPROC",
    "DIRSTACK",
    "EMACS",
    "ENV",
    "EPOCHREALTIME",
    "EPOCHSECONDS",
    "EUID",
    "EXECIGNORE",
    "FCEDIT",
    "FIGNORE",
    "FUNCNAME",
    "FUNCNEST",
    "GLOBIGNORE",
    "GROUPS",
    "HISTCMD",
    "HISTCONTROL",
    "HISTFILE",
    "HISTFILESIZE",
    "HISTIGNORE",
    "HISTSIZE",
    "HISTTIMEFORMAT",
    "HOME",
    "HOSTFILE",
    "HOSTNAME",
    "HOSTTYPE",
    "IFS",
    "IGNOREEOF",
    "INPUTRC",
    "INSIDE_EMACS",
    "LANG",
    "LINENO",
    "LINES",
    "MACHTYPE",
    "MAIL",
    "MAILCHECK",
    "MAILPATH",
    "MAPFILE",
    "OLDPWD",
    "OPTARG",
    "OPTERR",
    "OPTIND",
    "OSTYPE",
    "PATH",
    "PIPESTATUS",
    "POSIXLY_CORRECT",
    "PPID",
    "PROMPT_COMMAND",
    "PROMPT_DIRTRIM",
    "PS0",
    "PS1",
    "PS2",
    "PS3",
    "PS4",
    "PWD",
    "RANDOM",
    "REPLY",
    "SECONDS",
    "SHELL",
    "SHELLOPTS",
    "SHLVL",
    "SRANDOM",
    "TIMEFORMAT",
    "TMOUT",
    "TMPDIR",
    "UID",
    "USER",
    "histchars",
];
const BASH_RESERVED_PREFIXES: &[&str] = &["BASH_", "COMP_", "LC_", "READLINE_"];

const SHELL_COMMANDS: &[&str] = &[
    "printf", "read", "mkdir", "rm", "cp", "mv", "cd", "export", "local", "return", "exit",
    "sleep", "dirname", "pwd", "command", "builtin", "set", "shift", "test", "echo", "eval",
    "exec", "source", "declare", "unset", "true", "false", "then", "fi", "do", "done", "elif",
    "case", "esac", "until", "select", "function", "time", "coproc",
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
    if BATCH_RESERVED.contains(&name.to_lowercase().as_str())
        || BASH_RESERVED.contains(&name)
        || BASH_RESERVED_PREFIXES
            .iter()
            .any(|prefix| name.starts_with(prefix))
    {
        return Err(RosellaError::compiler(format!(
            "'{}' would overwrite an environment variable of the shell; choose another name",
            name
        )));
    }
    Ok(())
}
