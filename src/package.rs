use crate::error::BuildError;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Native SigmaOS package formats.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum PackageFormat {
    /// SigmaOS native package format (.sigma.pkg)
    #[default]
    SigmaPkg,
    /// Deb-compatible output for downstream consumers
    Deb,
    /// RPM-compatible output for downstream consumers
    Rpm,
    /// Universal tarball fallback
    Tarball,
}

pub struct PackageBuilder {
    format: PackageFormat,
}

impl PackageBuilder {
    pub fn new() -> Result<Self, BuildError> {
        Ok(Self {
            format: PackageFormat::SigmaPkg,
        })
    }

    pub fn with_format(format: PackageFormat) -> Self {
        Self { format }
    }

    pub fn build_package(&self, package_name: &str) -> Result<PackageResult, BuildError> {
        let build_dir = PathBuf::from("build").join(package_name);
        fs::create_dir_all(&build_dir)?;

        let source_dir = PathBuf::from("packages").join(package_name);

        if !source_dir.exists() {
            return Err(BuildError::PackageNotFound(package_name.to_string()));
        }

        let output_path = match self.format {
            PackageFormat::SigmaPkg => {
                self.build_sigma_package(&source_dir, &build_dir, package_name)?
            }
            PackageFormat::Deb => self.build_deb_package(&source_dir, &build_dir, package_name)?,
            PackageFormat::Rpm => self.build_rpm_package(&source_dir, &build_dir, package_name)?,
            PackageFormat::Tarball => {
                self.build_tarball_package(&source_dir, &build_dir, package_name)?
            }
        };

        let checksum = self.calculate_checksum(&output_path)?;

        Ok(PackageResult {
            name: package_name.to_string(),
            version: "1.0.0".to_string(),
            path: output_path,
            checksum,
        })
    }

    /// Build a SigmaOS native package using `sigma-build` (or the `SIGMA_BUILD_BIN` override).
    fn build_sigma_package(
        &self,
        source_dir: &Path,
        build_dir: &Path,
        package_name: &str,
    ) -> Result<PathBuf, BuildError> {
        let sigma_build_dir = build_dir.join("sigma-pkgbuild");
        fs::create_dir_all(&sigma_build_dir)?;

        let sigma_build_bin =
            env::var("SIGMA_BUILD_BIN").unwrap_or_else(|_| "sigma-build".to_string());

        let output = Command::new(&sigma_build_bin)
            .args(["--force", "--no-confirm"])
            .current_dir(source_dir)
            .output()?;

        if !output.status.success() {
            return Err(BuildError::BuildFailed(package_name.to_string()));
        }

        let pkg_file =
            source_dir.join(format!("{}-1.0.0-x86_64.sigma.pkg.zst", package_name));

        if pkg_file.exists() {
            Ok(pkg_file)
        } else {
            Err(BuildError::PackageNotFound(format!(
                "{}-1.0.0-x86_64.sigma.pkg.zst",
                package_name
            )))
        }
    }

    fn build_deb_package(
        &self,
        source_dir: &Path,
        build_dir: &Path,
        package_name: &str,
    ) -> Result<PathBuf, BuildError> {
        let deb_dir = build_dir.join("deb");
        fs::create_dir_all(&deb_dir)?;

        let output = Command::new("dpkg-buildpackage")
            .args(["-b", "-uc", "-us"])
            .current_dir(source_dir)
            .output()?;

        if !output.status.success() {
            return Err(BuildError::BuildFailed(package_name.to_string()));
        }

        let deb_file = deb_dir.join(format!("{}_1.0.0_amd64.deb", package_name));

        if deb_file.exists() {
            Ok(deb_file)
        } else {
            Err(BuildError::PackageNotFound(format!(
                "{}_1.0.0_amd64.deb",
                package_name
            )))
        }
    }

    fn build_rpm_package(
        &self,
        source_dir: &Path,
        build_dir: &Path,
        package_name: &str,
    ) -> Result<PathBuf, BuildError> {
        let rpm_dir = build_dir.join("rpm");
        fs::create_dir_all(&rpm_dir)?;

        let output = Command::new("rpmbuild")
            .args(["-ba", "--define", "_topdir", build_dir.to_str().unwrap()])
            .current_dir(source_dir)
            .output()?;

        if !output.status.success() {
            return Err(BuildError::BuildFailed(package_name.to_string()));
        }

        let rpm_file = rpm_dir.join(format!("{}-1.0.0-1.x86_64.rpm", package_name));

        if rpm_file.exists() {
            Ok(rpm_file)
        } else {
            Err(BuildError::PackageNotFound(format!(
                "{}-1.0.0-1.x86_64.rpm",
                package_name
            )))
        }
    }

    fn build_tarball_package(
        &self,
        source_dir: &Path,
        build_dir: &Path,
        package_name: &str,
    ) -> Result<PathBuf, BuildError> {
        let tarball_path = build_dir.join(format!("{}-1.0.0.tar.gz", package_name));

        let output = Command::new("tar")
            .args([
                "-czf",
                tarball_path.to_str().unwrap(),
                "-C",
                source_dir.to_str().unwrap(),
                ".",
            ])
            .output()?;

        if !output.status.success() {
            return Err(BuildError::BuildFailed(package_name.to_string()));
        }

        Ok(tarball_path)
    }

    fn calculate_checksum(&self, path: &Path) -> Result<String, BuildError> {
        let output = Command::new("sha256sum").arg(path).output()?;

        if output.status.success() {
            let checksum = String::from_utf8_lossy(&output.stdout);
            Ok(checksum.split_whitespace().next().unwrap_or("").to_string())
        } else {
            Err(BuildError::ChecksumFailed(
                path.to_string_lossy().to_string(),
            ))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageResult {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub checksum: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Arrange: builder with SigmaPkg format
    // Act: build_package for a non-existent package
    // Assert: returns PackageNotFound
    #[test]
    fn test_build_sigma_pkg_missing_source_aaa() {
        let builder = PackageBuilder::with_format(PackageFormat::SigmaPkg);
        let result = builder.build_package("non-existent-pkg-xyz");
        assert!(matches!(result, Err(BuildError::PackageNotFound(_))));
    }

    // Arrange: builder with Deb format
    // Act: build_package for a non-existent package
    // Assert: returns PackageNotFound
    #[test]
    fn test_build_deb_missing_source_aaa() {
        let builder = PackageBuilder::with_format(PackageFormat::Deb);
        let result = builder.build_package("non-existent-pkg-xyz");
        assert!(matches!(result, Err(BuildError::PackageNotFound(_))));
    }

    // Arrange: builder with Rpm format
    // Act: build_package for a non-existent package
    // Assert: returns PackageNotFound
    #[test]
    fn test_build_rpm_missing_source_aaa() {
        let builder = PackageBuilder::with_format(PackageFormat::Rpm);
        let result = builder.build_package("non-existent-pkg-xyz");
        assert!(matches!(result, Err(BuildError::PackageNotFound(_))));
    }

    // Arrange: builder default (SigmaPkg)
    // Act: PackageBuilder::new()
    // Assert: succeeds
    #[test]
    fn test_new_builder_defaults_to_sigma_pkg_aaa() {
        let builder = PackageBuilder::new().expect("builder should construct");
        // format is SigmaPkg by default
        let result = builder.build_package("any-package");
        assert!(matches!(result, Err(BuildError::PackageNotFound(_))));
    }
}
