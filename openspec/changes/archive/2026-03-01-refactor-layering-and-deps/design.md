## Context

当前 `handler::score_from_file` 混杂 multipart 提取、calamine Xlsx 解析、`skip(3)` 行跳过、列索引 0/1/2 取值、
`score_trans_grade`、`round_2decimal`、`Course` 构建、`process_scraped_course_results`、session 写入。handler 还直接 import
`scraping::USER_AGENT` 与 `fake_user_agent::get_rua`，在 logout 时操作 Mutex 与 UA，依赖 scraping 内部实现。scraping 使用
`lazy_static! { pub static ref USER_AGENT: Mutex<String> }`，引入 lazy_static 依赖。project-context 要求：handler 只处理
HTTP 与调用；business 负责计算与规则；scraping 负责教务网请求；models 定义数据结构与领域错误。

## Goals / Non-Goals

**Goals:**

- 将 Excel 解析「bytes → Vec<Course>」抽离出 handler，使其仅负责 multipart 提取、调用解析、调用
  process_scraped_course_results、写入 session。
- 解析函数返回 `Result<Vec<Course>, FileError>`，依赖 `score_trans_grade`、`round_2decimal`、`Course`。
- UA 简化：项目内 UA 仅在一处使用（`AAOWebsite::new()` 创建 client），每次登录新建实例时可直接调用 `get_rua()`，无需静态变量与
  refresh 逻辑；移除 `USER_AGENT`、`refresh_user_agent`、lazy_static。
- 更新 project-context 的 Technology Stack 依赖列表（移除 lazy_static）。

**Non-Goals:**

- 不新增路由、不新增错误变体；不修改 Excel 模板格式或解析规则（保持 Sheet1、skip(3)、列 0/1/2 等行为）。
- 不在此 change 中支持 CSV 等其他格式；若未来需要，可再引入独立 parse 模块。

## Decisions

1. **解析放置位置：business vs 独立 parse 模块**
    - **决策**：放在 business 中，新增
      `parse_courses_from_excel_template(bytes: &[u8]) -> Result<Vec<Course>, FileError>`。
    - **理由**：模板固定、短期内无多格式需求；解析依赖 business 的 `score_trans_grade`、`round_2decimal`，放 business
      可避免跨模块依赖；若将来支持 CSV 等，再拆出 `parse` 模块。
    - **备选**：独立 `src/parse_excel.rs` 模块，适合多格式扩展，但当前增加目录与 crate 结构复杂度。

2. **handler 依赖与 import**
    - **决策**：handler 从 business 的 import 中移除 `round_2decimal`、`score_trans_grade`；仅保留
      `process_scraped_course_results`、`ResultSource`、`ProcessedGPAResults` 及常量（如 `ATTR_EXCLUSIONS` 等用于模板渲染）。
    - **理由**：解析逻辑移入 business 后，handler 不再直接使用成绩转换或舍入。

3. **UA 实现**
    - **决策**：移除 `USER_AGENT` 静态变量与 `refresh_user_agent()`；`AAOWebsite::new()` 中直接调用
      `get_rua().to_string()` 作为 client 的 user_agent。
    - **理由**：UA 仅在创建 client 时使用，每次登录都会新建 `AAOWebsite`，直接调用 `get_rua()` 即可获得新 UA，无需共享静态；简化后
      handler::logout 不再依赖 scraping 内部，符合分层规范。

4. **修改的文件列表**
    - `src/handler.rs`：score_from_file 精简为 multipart → bytes → `parse_courses_from_excel_template` →
      process_scraped_course_results → session；logout 移除对 scraping 的 UA 相关依赖；移除
      calamine、round_2decimal、score_trans_grade 等相关 import。
    - `src/business.rs`：新增 `parse_courses_from_excel_template`，依赖 calamine、models::Course、FileError。
    - `src/scraping.rs`：移除 `USER_AGENT` 静态与 `refresh_user_agent()`；`AAOWebsite::new()` 中直接 `get_rua()`；移除
      lazy_static、Mutex、OnceLock 等 import。
    - `Cargo.toml`：移除 lazy_static。
    - `_bmad-output/project-context.md`：Technology Stack 中移除 lazy_static。

## Risks / Trade-offs

- **[Risk]** business 将依赖 calamine，若未来解析放回 handler 或独立模块，需调整依赖。  
  **Mitigation**：project-context 允许 business 与业务规则相关；Excel 解析属于「成绩数据准备」的规则，放在 business 合理；若拆分
  parse 模块，届时再移动 calamine 依赖。

## Migration Plan

无需数据迁移或回滚。按 tasks 顺序依次修改代码；每步完成后运行 `cargo build` 与 `cargo test`；最终移除 lazy_static 后确认
project-context 已更新。

## Open Questions

- 无。解析放置 business 的决策已明确；若实现过程中发现 calamine 与 business 其他逻辑耦合过重，可再评估拆分 parse 模块。
