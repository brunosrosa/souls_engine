//! Zero-copy metadata inspection for discovered models using souls_inference_runtime.

use std::path::Path;
use souls_inference_runtime::inspect_gguf_metadata_o1;
use crate::domain::model::ModelDescriptor;

/// Enhances descriptor with zero-copy GGUF header inspection.
pub fn enrich_with_gguf_metadata(desc: &mut ModelDescriptor) {
    let ext = desc.file_path.extension().and_then(|s| s.to_str()).unwrap_or("");
    if ext.to_lowercase() != "gguf" {
        return;
    }

    if let Ok(meta) = inspect_gguf_metadata_o1(&desc.file_path) {
        if meta.context_length > 0 {
            desc.ctx_max = meta.context_length as u32;
        }
        if !meta.architecture.is_empty() && desc.parameters == "Unknown" {
            desc.parameters = format!("{}-arch", meta.architecture);
        }
    }
}

/// Helper function to inspect any arbitrary file on disk.
pub fn inspect_model_file<P: AsRef<Path>>(path: P) -> Option<ModelDescriptor> {
    let p = path.as_ref();
    if !p.exists() || !p.is_file() {
        return None;
    }
    let mut desc = ModelDescriptor::from_file(p);
    enrich_with_gguf_metadata(&mut desc);
    Some(desc)
}
