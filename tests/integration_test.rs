use sigmaos_build_system::{BuildSystem, BuildManifest, BuildOptions};
use std::path::PathBuf;

#[test]
fn test_build_manifest_default() {
    let manifest = BuildManifest {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        description: "Test build".to_string(),
        packages: vec!["test-package".to_string()],
        build_options: BuildOptions::default(),
    };
    
    assert_eq!(manifest.name, "test");
    assert_eq!(manifest.version, "1.0.0");
    assert_eq!(manifest.packages.len(), 1);
}

#[test]
fn test_build_system_new() {
    let manifest = BuildManifest {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        description: "Test build".to_string(),
        packages: vec![],
        build_options: BuildOptions::default(),
    };
    
    let build_system = BuildSystem::new(manifest);
    assert!(build_system.is_ok());
}
