// Integration test for SigmaOS-arch-build-system
// Verifies the crate builds and links as a dependency.
#![allow(unused_imports)]

use arch_build_system as _;

#[test]
fn crate_links_and_runs() {
    assert_eq!(2 + 2, 4);
}
