use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CatalogEntry {
    pub id: &'static str,
    pub name: &'static str,
    pub params_billions: f64,
    pub size_mb: f64,
    pub license: &'static str,
    pub description: &'static str,
    pub hf_repo: &'static str,
    pub gguf_file: &'static str,
    pub tok_model_id: &'static str,
}

pub fn model_catalog() -> Vec<CatalogEntry> {
    vec![
        CatalogEntry {
            id: "HuggingFaceTB/SmolLM2-135M-Instruct",
            name: "SmolLM2 135M",
            params_billions: 0.135,
            size_mb: 90.0,
            license: "Apache-2.0",
            description: "Smallest instruct model, fast but limited quality. Good for simple tasks.",
            hf_repo: "unsloth/SmolLM2-135M-Instruct-GGUF",
            gguf_file: "SmolLM2-135M-Instruct-Q4_K_M.gguf",
            tok_model_id: "HuggingFaceTB/SmolLM2-135M-Instruct",
        },
        CatalogEntry {
            id: "HuggingFaceTB/SmolLM2-360M-Instruct",
            name: "SmolLM2 360M",
            params_billions: 0.36,
            size_mb: 220.0,
            license: "Apache-2.0",
            description: "Small instruct model, decent quality for short responses.",
            hf_repo: "unsloth/SmolLM2-360M-Instruct-GGUF",
            gguf_file: "SmolLM2-360M-Instruct-Q4_K_M.gguf",
            tok_model_id: "HuggingFaceTB/SmolLM2-360M-Instruct",
        },
        CatalogEntry {
            id: "HuggingFaceTB/SmolLM2-1.7B-Instruct",
            name: "SmolLM2 1.7B",
            params_billions: 1.7,
            size_mb: 1000.0,
            license: "Apache-2.0",
            description: "Best quality local model, suitable for general chat and coding help.",
            hf_repo: "unsloth/SmolLM2-1.7B-Instruct-GGUF",
            gguf_file: "SmolLM2-1.7B-Instruct-Q4_K_M.gguf",
            tok_model_id: "HuggingFaceTB/SmolLM2-1.7B-Instruct",
        },
    ]
}

pub fn find_catalog_entry(model_id: &str) -> Option<CatalogEntry> {
    model_catalog().into_iter().find(|e| e.id == model_id)
}
