use souls_llm_inventory::catalog::{
    resolve_canonical_models_dir, scan_local_models,
};
use souls_llm_inventory::profiler::HardwareDiagnostic;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_catalog_directory_creation_and_scanning() {
    let dir = tempdir().unwrap();
    let diag = HardwareDiagnostic::default();

    // Create dummy tier folders
    let tier1_dir = dir.path().join("tier1_coder");
    std::fs::create_dir_all(&tier1_dir).unwrap();

    let model_path = tier1_dir.join("qwen2.5-coder-7b-instruct-q4_k_m.gguf");
    let mut f = std::fs::File::create(&model_path).unwrap();
    f.write_all(b"GGUF_HEADER_MOCK_BYTES").unwrap();

    let models = scan_local_models(dir.path(), &diag).unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].file_name, "qwen2.5-coder-7b-instruct-q4_k_m.gguf");
    assert_eq!(models[0].quantization, "Q4_K_M");
}

#[test]
fn test_canonical_directory_resolution() {
    let dir = resolve_canonical_models_dir();
    assert!(dir.exists(), "Canonical models dir must exist or be created idempotently");
}
