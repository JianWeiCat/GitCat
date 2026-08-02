//! 报表导出 — 写入 Markdown / CSV / JSON 文件

use crate::stats::engine::StatsResult;
use crate::stats::report;

/// 导出统计结果
pub fn export(result: &StatsResult, output: &str, format: ExportFormat) -> Result<(), crate::error::GhError> {
    let content = match format {
        ExportFormat::Markdown => report::format_markdown(result),
        ExportFormat::Csv => format_csv(result),
        ExportFormat::Json => report::format_json(result),
    };

    std::fs::write(output, content)
        .map_err(|e| crate::error::GhError::Config(format!("导出失败: {}", e)))?;

    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Markdown,
    Csv,
    Json,
}

/// 生成 CSV 内容
fn format_csv(result: &StatsResult) -> String {
    let mut authors = result.authors.clone();
    authors.sort_by(|a, b| b.commits.cmp(&a.commits));

    let mut csv = String::from("排名,作者,邮箱,提交数,新增行,删除行,净增,代码行,注释行,空行\n");

    for (i, a) in authors.iter().enumerate() {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{}\n",
            i + 1,
            escape_csv(&a.name),
            a.email,
            a.commits,
            a.lines_added,
            a.lines_deleted,
            a.net_lines,
            a.code_lines,
            a.comment_lines,
            a.blank_lines,
        ));
    }

    csv
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
