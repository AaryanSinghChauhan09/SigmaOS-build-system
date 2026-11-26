use crate::builder::{BuildManifest, VerificationResult};
use crate::error::BuildError;
use std::path::PathBuf;

pub struct BuildVerifier;

impl BuildVerifier {
    pub fn new() -> Result<Self, BuildError> {
        Ok(Self)
    }

    pub fn verify_build(&self, manifest: &BuildManifest) -> Result<VerificationResult, BuildError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        self.verify_manifest(manifest, &mut errors, &mut warnings)?;
        self.verify_packages(manifest, &mut errors, &mut warnings)?;
        self.verify_checksums(manifest, &mut errors, &mut warnings)?;

        Ok(VerificationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        })
    }

    fn verify_manifest(
        &self,
        manifest: &BuildManifest,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) -> Result<(), BuildError> {
        if manifest.name.is_empty() {
            errors.push("Package name cannot be empty".to_string());
        }

        if manifest.version.is_empty() {
            errors.push("Package version cannot be empty".to_string());
        }

        if manifest.packages.is_empty() {
            warnings.push("No packages specified in manifest".to_string());
        }

        Ok(())
    }

    fn verify_packages(
        &self,
        manifest: &BuildManifest,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) -> Result<(), BuildError> {
        for package in &manifest.packages {
            let package_dir = PathBuf::from("packages").join(package);

            if !package_dir.exists() {
                errors.push(format!("Package directory not found: {}", package));
            }

            let pkgbuild = package_dir.join("PKGBUILD");
            if !pkgbuild.exists() {
                warnings.push(format!("PKGBUILD not found for package: {}", package));
            }
        }

        Ok(())
    }

    fn verify_checksums(
        &self,
        manifest: &BuildManifest,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) -> Result<(), BuildError> {
        let build_dir = PathBuf::from("build");

        if !build_dir.exists() {
            warnings.push("Build directory does not exist".to_string());
            return Ok(());
        }

        for package in &manifest.packages {
            let package_build_dir = build_dir.join(package);

            if !package_build_dir.exists() {
                errors.push(format!("Build output not found for package: {}", package));
            }
        }

        Ok(())
    }
}
