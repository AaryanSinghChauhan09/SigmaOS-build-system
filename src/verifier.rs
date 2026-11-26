use crate::builder::{BuildManifest, VerificationResult};
use crate::error::BuildError;
use std::path::Path;

pub struct BuildVerifier;

impl BuildVerifier {
    pub fn new() -> Result<Self, BuildError> {
        Ok(Self)
    }

    pub fn verify_build(&self, manifest: &BuildManifest) -> Result<VerificationResult, BuildError> {
        self.verify_build_at(manifest, Path::new("."))
    }

    /// Verify against an explicit root directory (tests, chroots).
    pub fn verify_build_at(
        &self,
        manifest: &BuildManifest,
        root: &Path,
    ) -> Result<VerificationResult, BuildError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        self.verify_manifest(manifest, &mut errors, &mut warnings)?;
        self.verify_packages(manifest, root, &mut errors, &mut warnings)?;
        self.verify_checksums(manifest, root, &mut errors, &mut warnings)?;

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
        root: &Path,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) -> Result<(), BuildError> {
        for package in &manifest.packages {
            let package_dir = root.join("packages").join(package);

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
        root: &Path,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) -> Result<(), BuildError> {
        let build_dir = root.join("build");

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::BuildOptions;
    use std::fs;

    fn manifest(pkgs: &[&str]) -> BuildManifest {
        BuildManifest {
            name: "test".into(),
            version: "1.0".into(),
            description: String::new(),
            packages: pkgs.iter().map(|s| s.to_string()).collect(),
            build_options: BuildOptions::default(),
        }
    }

    #[test]
    fn valid_tree_passes() {
        let root = std::env::temp_dir().join(format!("bs-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("packages/foo")).unwrap();
        fs::write(root.join("packages/foo/PKGBUILD"), "pkgname=foo\n").unwrap();
        fs::create_dir_all(root.join("build/foo")).unwrap();
        let v = BuildVerifier::new()
            .unwrap()
            .verify_build_at(&manifest(&["foo"]), &root)
            .unwrap();
        assert!(v.valid, "{:?}", v.errors);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn missing_package_dir_fails() {
        let root = std::env::temp_dir().join(format!("bs-test-miss-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let v = BuildVerifier::new()
            .unwrap()
            .verify_build_at(&manifest(&["ghost"]), &root)
            .unwrap();
        assert!(!v.valid);
        assert!(v.errors.iter().any(|e| e.contains("ghost")));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn empty_name_is_error() {
        let root = std::env::temp_dir().join(format!("bs-test-name-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let mut m = manifest(&[]);
        m.name = String::new();
        let v = BuildVerifier::new()
            .unwrap()
            .verify_build_at(&m, &root)
            .unwrap();
        assert!(!v.valid);
        let _ = fs::remove_dir_all(&root);
    }
}
