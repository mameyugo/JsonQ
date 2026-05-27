//! Tests for StoreOpts

use jsonq::store::options::{CompressionMethod, StoreOpts};
use std::sync::Arc;

#[test]
fn test_default_options() {
    let opts = StoreOpts::default();
    assert_eq!(opts.pretty, false);
    assert_eq!(opts.fsync, false);
    assert!(matches!(opts.compression, CompressionMethod::None));
}

#[test]
fn test_production_options() {
    let opts = StoreOpts::production();
    assert_eq!(opts.pretty, false, "Production should use compact JSON");
    assert_eq!(opts.fsync, false, "Production should skip fsync for speed");
}

#[test]
fn test_development_options() {
    let opts = StoreOpts::development();
    assert_eq!(opts.pretty, true, "Development should use pretty JSON");
    assert_eq!(opts.fsync, true, "Development should fsync for safety");
}

#[test]
fn test_safe_options() {
    let opts = StoreOpts::safe();
    assert_eq!(opts.pretty, false, "Safe mode uses compact JSON");
    assert_eq!(opts.fsync, true, "Safe mode always fsyncs");
}

#[test]
fn test_custom_options() {
    let opts = StoreOpts {
        pretty: true,
        fsync: false,
        compression: CompressionMethod::None,
        revision_log: true,
    };
    assert_eq!(opts.pretty, true);
    assert_eq!(opts.fsync, false);
    assert_eq!(opts.revision_log, true);
}

#[test]
fn test_clone_options() {
    let opts1 = StoreOpts::development();
    let opts2 = opts1.clone();
    assert_eq!(opts1.pretty, opts2.pretty);
    assert_eq!(opts1.fsync, opts2.fsync);
}
