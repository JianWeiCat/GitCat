//! 文件过滤 — 判断文件是否应被统计忽略

const IGNORE_PATTERNS: &[&str] = &[
    "*.o", "*.so", "*.dylib", "*.dll", "*.exe", "*.class", "*.pyc",
    "*.wasm", "*.obj", "*.pdb",
    "Cargo.lock", "package-lock.json", "yarn.lock", "pnpm-lock.yaml",
    "Gemfile.lock", "poetry.lock", "Pipfile.lock", "composer.lock",
    "*.generated.*", "*.pb.go", "*.pb.rs",
    "*.png", "*.jpg", "*.jpeg", "*.gif", "*.ico", "*.svg",
    "*.woff", "*.woff2", "*.ttf", "*.eot", "*.otf",
    "*.mp3", "*.mp4", "*.webm", "*.zip", "*.tar", "*.gz",
    "*.pdf", "*.doc", "*.docx", "*.xls", "*.xlsx",
    "target/", "node_modules/", "dist/", "build/", ".next/",
    "__pycache__/", "*.egg-info/", "vendor/",
    ".idea/", ".vscode/", "*.swp", "*.swo", "*~", ".DS_Store",
    ".git/", ".svn/",
];

pub fn should_ignore(path: &str) -> bool {
    if path.is_empty() || path == "-" || path == "/dev/null" {
        return true;
    }
    let normalized = path.replace('\\', "/");
    IGNORE_PATTERNS.iter().any(|p| {
        if p.ends_with('/') {
            normalized.contains(p) || normalized.starts_with(p)
        } else if p.starts_with('*') && p[1..].contains('.') && !p.contains('/') {
            normalized.ends_with(&p[1..])
        } else if p.starts_with('*') {
            normalized.ends_with(&p[1..])
        } else if p.contains('/') {
            normalized.contains(p)
        } else {
            normalized == *p || normalized.ends_with(&format!("/{}", p))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_build_output() {
        assert!(should_ignore("target/debug/main.o"));
        assert!(should_ignore("node_modules/react/index.js"));
        assert!(should_ignore("dist/bundle.js"));
    }

    #[test]
    fn ignores_binary_files() {
        assert!(should_ignore("icon.png"));
        assert!(should_ignore("font.woff2"));
        assert!(should_ignore("app.exe"));
    }

    #[test]
    fn ignores_lock_files() {
        assert!(should_ignore("Cargo.lock"));
        assert!(should_ignore("package-lock.json"));
    }

    #[test]
    fn keeps_source_code() {
        assert!(!should_ignore("src/main.rs"));
        assert!(!should_ignore("app.ts"));
        assert!(!should_ignore("main.go"));
    }

    #[test]
    fn keeps_config_files() {
        assert!(!should_ignore("README.md"));
        assert!(!should_ignore("Cargo.toml"));
        assert!(!should_ignore("package.json"));
        assert!(!should_ignore("Dockerfile"));
        assert!(!should_ignore("LICENSE"));
    }
}
