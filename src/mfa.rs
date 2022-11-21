use std::io;
use std::io::Write;

use async_trait::async_trait;

#[async_trait]
pub trait ReadMfaToken {
    async fn read_mfa_token(&self, mfa_serial: &str) -> anyhow::Result<String>;
}

pub struct StdinMfaTokenReader;

#[async_trait]
impl ReadMfaToken for StdinMfaTokenReader {
    async fn read_mfa_token(&self, mfa_serial: &str) -> anyhow::Result<String> {
        print!("Enter MFA code for {}: ", mfa_serial);
        io::stdout().flush()?;

        let mut code = String::new();
        io::stdin().read_line(&mut code)?;
        Ok(code.trim().to_string())
    }
}

pub struct StaticMfaTokenReader {
    token: String,
}

impl<S: Into<String>> From<S> for StaticMfaTokenReader {
    fn from(s: S) -> Self {
        StaticMfaTokenReader { token: s.into() }
    }
}

#[async_trait]
impl ReadMfaToken for StaticMfaTokenReader {
    async fn read_mfa_token(&self, _mfa_serial: &str) -> anyhow::Result<String> {
        Ok(self.token.clone())
    }
}
