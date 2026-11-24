use thiserror::Error;

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),
    
    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),
    
    #[error("WalkDir error: {0}")]
    WalkDir(#[from] walkdir::Error),
    
    #[error("Build failed for package: {0}")]
    BuildFailed(String),
    
    #[error("Package not found: {0}")]
    PackageNotFound(String),
    
    #[error("Checksum calculation failed: {0}")]
    ChecksumFailed(String),
    
    #[error("Verification failed: {0}")]
    VerificationFailed(String),
}
