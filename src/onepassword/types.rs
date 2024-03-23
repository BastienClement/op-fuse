use time::OffsetDateTime;

/// The version of a secret.
pub type SecretVersion = u16;

/// Metadata about a secret.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMetadata {
    /// The ID of the secret.
    pub id: String,

    /// The title of the secret.
    pub title: String,

    /// The version of the secret.
    pub version: SecretVersion,

    /// The category of the secret.
    pub category: String,

    /// The creation time of the secret.
    #[serde(with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,

    /// The update time of the secret.
    #[serde(with = "time::serde::iso8601")]
    pub updated_at: OffsetDateTime,
}

/// A secret.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret {
    /// Metadata of the secret.
    #[serde(flatten)]
    pub metadata: SecretMetadata,

    /// The fields in the secret.
    pub fields: Vec<SecretField>,
}

/// A field in a secret.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretField {
    /// The ID of the field.
    pub id: String,

    /// The op://-reference of the field.
    /// Might be empty - or even broken - in some cases.
    pub reference: String,

    /// The value of the field.
    /// Sometimes not present in the output.
    pub value: Option<String>,
}
