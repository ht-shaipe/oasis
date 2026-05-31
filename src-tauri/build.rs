use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set"));
    let output_path = out_dir.join("generated_invoke_handler.rs");

    let sources = [
        (workspace_root.join("src/commands.rs"), "commands::"),
        (
            workspace_root.join("crates/credential/src/commands.rs"),
            "oasis_credential::commands::",
        ),
        (
            workspace_root.join("crates/toolbox/src/commands.rs"),
            "oasis_toolbox::commands::",
        ),
        (
            workspace_root.join("crates/browser/src/commands.rs"),
            "oasis_browser::commands::",
        ),
    ];

    for (path, _) in &sources {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    println!("cargo:rerun-if-changed=build.rs");

    let mut handlers = Vec::new();
    for (path, prefix) in &sources {
        handlers.extend(extract_handlers(path, prefix));
    }

    let mut generated = String::from("tauri::generate_handler![\n");
    for handler in handlers {
        generated.push_str("    ");
        generated.push_str(&handler);
        generated.push_str(",\n");
    }
    generated.push_str("]\n");

    fs::write(&output_path, generated).expect("failed to write generated handler file");

    tauri_build::build()
}

fn extract_handlers(path: &Path, prefix: &str) -> Vec<String> {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {}", path.display(), err));

    let mut handlers = Vec::new();
    let mut pending_command = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "#[tauri::command]" {
            pending_command = true;
            continue;
        }

        if pending_command {
            if let Some(name) = parse_pub_fn_name(trimmed) {
                handlers.push(format!("{}{}", prefix, name));
            }
            pending_command = false;
        }
    }

    handlers
}

fn parse_pub_fn_name(line: &str) -> Option<String> {
    let line = line.strip_prefix("pub fn ")?;
    let name = line.split_once('(')?.0.trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}
