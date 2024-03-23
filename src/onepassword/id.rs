/// The ID of a 1Password account
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account(String);

impl Account {
    /// Creates a new account ID
    pub fn new(account: &str) -> Account {
        Account(account.to_string())
    }

    /// The account ID
    pub fn account(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Account {
    /// Creates a new account ID from the given string
    fn from(account: &str) -> Account {
        Account::new(account)
    }
}

/// The ID of a 1Password vault
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vault(Account, String);

impl Vault {
    /// Creates a new vault ID
    pub fn new(account: &Account, vault: &str) -> Vault {
        Vault(account.clone(), vault.to_string())
    }

    /// The account ID
    pub fn account(&self) -> &str {
        self.0.account()
    }

    /// The vault ID
    pub fn vault(&self) -> &str {
        &self.1
    }
}

/// The ID of a 1Password secret
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Secret(Vault, String);

impl Secret {
    /// Creates a new secret ID
    pub fn new(vault: &Vault, secret: &str) -> Secret {
        Secret(vault.clone(), secret.to_string())
    }

    /// The account ID
    pub fn account(&self) -> &str {
        self.0.account()
    }

    /// The vault ID
    pub fn vault(&self) -> &str {
        self.0.vault()
    }

    /// The secret ID
    pub fn secret(&self) -> &str {
        &self.1
    }
}
