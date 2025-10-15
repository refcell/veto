use thiserror::Error;

/// Errors that can occur while loading or resolving the configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Underlying IO failure.
    #[error("failed to read config file {path:?}: {source}")]
    Io {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    /// Failure to parse TOML.
    #[error("unable to parse config file as TOML: {source}")]
    TomlParse {
        #[from]
        source: toml::de::Error,
    },
    /// Invalid socket address for bind.
    #[error("invalid bind address '{value}': {source}")]
    BindAddress {
        value: String,
        source: std::net::AddrParseError,
    },
    /// Invalid upstream URL.
    #[error("invalid upstream url '{value}': {source}")]
    UpstreamUrl { value: String, source: http::Error },
}
