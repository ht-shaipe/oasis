use std::fs;
use std::path::Path;

const INDEXABLE_EXTENSIONS: &[&str] = &[
    "txt", "md", "rs", "ts", "js", "vue", "jsx", "tsx",
    "json", "yaml", "yml", "toml", "css", "scss", "html",
    "py", "go", "java", "c", "cpp", "h", "hpp", "sh", "bash",
    "sql", "graphql", "proto", "lua", "zig", "dart", "rb",
    "php", "swift", "kt", "scala", "clj", "ex", "exs", "erl",
    "hs", "ml", "mli", "nim", "r", "R", "pl", "pm",
];

const MAX_FILE_SIZE: u64 = 512 * 1024;

pub fn is_indexable(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => INDEXABLE_EXTENSIONS.contains(&ext),
        None => false,
    }
}

pub fn read_file_content(path: &Path) -> Option<FileContent> {
    let metadata = fs::metadata(path).ok()?;

    if metadata.len() > MAX_FILE_SIZE {
        return None;
    }

    let content = fs::read_to_string(path).ok()?;

    let line_count = content.lines().count();

    Some(FileContent {
        content,
        size: metadata.len(),
        line_count,
    })
}

pub fn compute_hash(content: &str) -> String {
    blake3::hash(content.as_bytes()).to_hex().to_string()
}

pub struct FileContent {
    pub content: String,
    pub size: u64,
    pub line_count: usize,
}

pub fn should_skip_dir(dir_name: &str) -> bool {
    const SKIP_DIRS: &[&str] = &[
        ".git",
        "node_modules",
        "target",
        ".venv",
        "venv",
        "__pycache__",
        ".cache",
        ".npm",
        ".pnpm-store",
        "dist",
        "build",
        ".next",
        ".nuxt",
        ".svelte-kit",
        "coverage",
        ".tox",
        ".mypy_cache",
        ".pytest_cache",
        ".idea",
        ".vscode",
        ".DS_Store",
    ];
    SKIP_DIRS.contains(&dir_name)
}
