//! 多仓库结果汇总

use crate::multi_repo::runner::OpResult;

/// 格式化操作结果汇总
pub fn format_summary(results: &[OpResult], operation: &str) -> String {
    let mut output = String::new();
    output.push_str(&format!("=== 批量 {} 结果 ===\n\n", operation));

    let success_count = results.iter().filter(|r| r.success).count();
    let fail_count = results.len() - success_count;

    for r in results {
        let icon = if r.success { "✅" } else { "❌" };
        output.push_str(&format!(
            "{} {} : {}\n",
            icon,
            r.repo_path,
            r.message,
        ));
    }

    output.push_str(&format!(
        "\n总计: {} 个仓库, {} 成功, {} 失败\n",
        results.len(),
        success_count,
        fail_count,
    ));

    output
}
