use std::env;
use std::ffi::CString;
use tracing::debug;

use crate::handler::{into_variables, HandleCredentials, HandleCredentialsRequest, Variable};

pub struct ShellCredentialsHandler;

impl HandleCredentials for ShellCredentialsHandler {
    fn handle_credentials(&self, request: HandleCredentialsRequest) -> anyhow::Result<()> {
        set_credentials(request);
        start_shell_session()?;
        Ok(())
    }
}

fn set_credentials(request: HandleCredentialsRequest) {
    let variables = into_variables(request);
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
