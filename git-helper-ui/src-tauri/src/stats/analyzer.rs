//! 行类型分析 — 判断行是代码 / 注释 / 空行

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineType {
    Code,
    Comment,
    Blank,
}

/// 根据文件扩展名判断行类型
pub fn classify_line(line: &str, ext: &str) -> LineType {
    let trimmed = line.trim();
    let ext_lower = ext.to_lowercase();

    if trimmed.is_empty() {
        return LineType::Blank;
    }

    match ext_lower.as_str() {
        // C-style: // 和 /* */
        "rs" | "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" | "hh"
        | "java" | "js" | "ts" | "jsx" | "tsx" | "go" | "swift"
        | "kt" | "kts" | "scala" | "dart" | "cs" | "m" | "mm"
        | "php" | "scss" | "less" | "css" | "sql" => {
            if trimmed.starts_with("//") || trimmed.starts_with("/*")
                || trimmed.starts_with('*') || trimmed == "*/" {
                LineType::Comment
            } else {
                LineType::Code
            }
        }
        // Hash-style: #
        "py" | "rb" | "sh" | "bash" | "zsh" | "fish" | "pl" | "pm"
        | "yaml" | "yml" | "toml" | "r" | "conf" | "cfg" | "ini"
        | "dockerfile" | "dockerignore" | "gitignore" => {
            if trimmed.starts_with('#') {
                LineType::Comment
            } else {
                LineType::Code
            }
        }
        // Hashbang / 未知: 视为代码
        _ => {
            if trimmed.starts_with("//") || trimmed.starts_with('#')
                || trimmed.starts_with("/*") || trimmed.starts_with("--") {
                LineType::Comment
            } else {
                LineType::Code
            }
        }
    }
}

/// 从文件路径提取扩展名（返回小写）
pub fn get_extension(path: &str) -> String {
    path.rsplit('.').next().unwrap_or("").to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blank_line() {
        assert_eq!(classify_line("", "rs"), LineType::Blank);
        assert_eq!(classify_line("   ", "py"), LineType::Blank);
        assert_eq!(classify_line("\t", "go"), LineType::Blank);
    }

    #[test]
    fn test_rust_comments() {
        assert_eq!(classify_line("fn main() {", "rs"), LineType::Code);
        assert_eq!(classify_line("// comment", "rs"), LineType::Comment);
        assert_eq!(classify_line("/// doc comment", "rs"), LineType::Comment);
        assert_eq!(classify_line("/* block */", "rs"), LineType::Comment);
    }

    #[test]
    fn test_python_comments() {
        assert_eq!(classify_line("def foo():", "py"), LineType::Code);
        assert_eq!(classify_line("# TODO", "py"), LineType::Comment);
        assert_eq!(classify_line("x = 1  # inline", "py"), LineType::Code);
    }

    #[test]
    fn test_js_comments() {
        assert_eq!(classify_line("const x = 1;", "js"), LineType::Code);
        assert_eq!(classify_line("// comment", "js"), LineType::Comment);
        assert_eq!(classify_line("* jsdoc", "js"), LineType::Comment);
    }
}
