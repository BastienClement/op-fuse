pub mod id;
pub mod types;

use std::process::Command;

use anyhow::Result;
use serde::de::DeserializeOwned;

use crate::config::Config;

/// A client for the 1Password CLI
#[derive(Debug)]
pub struct OnePassword {
    config: Config,
}

impl OnePassword {
    /// Creates a new 1Password client from the given configuration
    #[must_use]
    pub fn new(config: &Config) -> OnePassword {
        OnePassword {
            config: config.clone(),
        }
    }

    /// Runs the 1Password CLI with the given arguments and returns the result
    fn run<T>(&self, call_args: &[&str]) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let mut args: Vec<&str> = Vec::with_capacity(2 + call_args.len());
        args.extend(["--format", "json"]);
        args.extend(call_args);

        debug!(cmd = format!("op {:}", args.join(" ")));

        let output = Command::new(&self.config.op.cmd)
            .args(args)
            .output()
            .inspect_err(|e| error!(err = %e, "Failed to call OP"))?;

        Ok(serde_json::from_slice(&output.stdout)
            .inspect_err(|e| error!(err = %e, "Failed to decode OP response"))?)
    }

    /// Lists secrets in the given vault
    pub fn list_secrets(&self, vault: &id::Vault) -> Result<Vec<types::SecretMetadata>> {
        self.run(&[
            "item",
            "list",
            "--account",
            vault.account(),
            "--vault",
            vault.vault(),
        ])
    }

    /// Gets the secret with the given ID
    pub fn get_secret(&self, secret: &id::Secret) -> Result<types::Secret> {
        self.run(&[
            "item",
            "get",
            "--account",
            secret.account(),
            "--vault",
            secret.vault(),
            secret.secret(),
        ])
    }
}
