use std::env;
use std::ffi::CString;

use tracing::debug;

use assume_rolers_schema::credentials::ProfileCredentials;

use crate::handler::{into_variables, HandleCredentials, Variable};

pub struct ShellHandler;

impl HandleCredentials for ShellHandler {
    fn handle_credentials(self, credentials: ProfileCredentials) -> anyhow::Result<()> {
        set_credentials(credentials);
        start_shell_session()?;
        Ok(())
    }
}

fn set_credentials(credentials: ProfileCredentials) {
    let variables = into_variables(&credentials);
    for Variable { name, value } in variables {
        if let Some(value) = value {
            env::set_var(name, value);
        } else {
            env::remove_var(name);
        }
    }
}

fn start_shell_session() -> anyhow::Result<()> {
    let shell = env::var("SHELL")?;
    debug!("shell: {}, ", &shell);

    let shell = CString::new(shell.bytes().collect::<Vec<_>>())?;
    let args: Vec<CString> = Vec::new();
    nix::unistd::execv(&shell, &args)?;

    unreachable!("execv will replace the current process, so never reach this instruction.")
}
