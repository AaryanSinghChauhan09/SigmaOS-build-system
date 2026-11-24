use crate::error::BuildError;
use crate::package::PackageBuilder;
use crate::verifier::BuildVerifier;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub packages: Vec<String>,
    pub build_options: BuildOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildOptions {
    pub parallel: bool,
    pub clean: bool,
    pub verbose: bool,
    pub reproducible: bool,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            parallel: true,
            clean: false,
            verbose: false,
            reproducible: true,
        }
    }
}

pub struct BuildSystem {
    manifest: BuildManifest,
    package_builder: PackageBuilder,
    verifier: BuildVerifier,
}

impl BuildSystem {
    pub fn new(manifest: BuildManifest) -> Result<Self, BuildError> {
        let package_builder = PackageBuilder::new()?;
        let verifier = BuildVerifier::new()?;
        
        Ok(Self {
            manifest,
            package_builder,
            verifier,
        })
    }
    
    pub fn load_manifest(path: &PathBuf) -> Result<BuildManifest, BuildError> {
        let content = fs::read_to_string(path)?;
        let manifest: BuildManifest = toml::from_str(&content)?;
        Ok(manifest)
    }
    
    pub fn build(&self) -> Result<BuildResult, BuildError> {
        log::info!("Starting build for {}", self.manifest.name);
        
        if self.manifest.build_options.clean {
            self.clean_build()?;
        }
        
        let mut built_packages = Vec::new();
        
        for package in &self.manifest.packages {
            log::info!("Building package: {}", package);
            let package_result = self.package_builder.build_package(package)?;
            built_packages.push(package_result);
        }
        
        let verification_result = self.verifier.verify_build(&self.manifest)?;
        
        Ok(BuildResult {
            packages: built_packages,
            verification: verification_result,
        })
    }
    
    pub fn verify(&self) -> Result<bool, BuildError> {
        let verification = self.verifier.verify_build(&self.manifest)?;
        Ok(verification.valid)
    }
    
    pub fn clean(&self) -> Result<(), BuildError> {
        self.clean_build()
    }
    
    pub fn get_manifest(&self) -> &BuildManifest {
        &self.manifest
    }
    
    fn clean_build(&self) -> Result<(), BuildError> {
        let build_dir = PathBuf::from("build");
        if build_dir.exists() {
            fs::remove_dir_all(&build_dir)?;
        }
        
        let target_dir = PathBuf::from("target");
        if target_dir.exists() {
            fs::remove_dir_all(&target_dir)?;
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub packages: Vec<crate::package::PackageResult>,
    pub verification: VerificationResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}
