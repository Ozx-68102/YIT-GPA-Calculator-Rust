## Why

当前项目分层与依赖存在三方面问题：① handler 的 `score_from_file` 混杂 multipart 提取、Xlsx 解析、行跳过、列索引、成绩转换、Course
构建、GPA 处理与 session 写入，职责过重且违反 project-context 的分层规范；② handler::logout 直接 import scraping 的
`USER_AGENT` 与 `fake_user_agent::get_rua`，在 handler 内操作锁与 UA，依赖 scraping 内部实现；③ scraping 使用 `lazy_static`
作为额外依赖，而 UA 仅在 `AAOWebsite::new()` 一处使用，无需静态变量。本次重构旨在使架构符合 project-context
的分层规范，提升可维护性与可测试性。

## What Changes

- **Excel 解析职责拆分**：handler 仅负责「从 multipart 取 bytes → 调用解析函数 → 调用 process_scraped_course_results → 写入
  session」。解析逻辑「bytes → Vec<Course>」抽离到 business（或独立 parse 模块），依赖 `score_trans_grade`、`round_2decimal`、
  `Course`，返回 `Result<Vec<Course>, FileError>`。解析放置于 business（`parse_courses_from_excel_template`）或独立
  `parse_excel` 模块；若模板固定、短期无多格式需求，倾向于 business；若计划支持 CSV 等，倾向于独立 parse 模块。
- **UA 简化**：项目内 UA 仅在 `AAOWebsite::new()` 创建 client 时使用，每次登录都会新建实例。移除 `USER_AGENT` 静态变量、
  `refresh_user_agent()` 及 lazy_static；`AAOWebsite::new()` 直接调用 `get_rua()` 获取 UA，handler::logout 不再依赖
  scraping 内部实现。
- 不新增路由，不新增错误变体（沿用 `FileError`）；移除 lazy_static 后需更新 project-context 的依赖列表。

## Capabilities

### New Capabilities

- `excel-template-parsing`: 定义从 Excel 字节流解析为 `Vec<Course>` 的规范，包括模板格式（Sheet1、跳过前 3 行、列索引 0/1/2
  为课程名/学分/成绩）、使用 `score_trans_grade`/`round_2decimal` 转换、错误返回 `FileError`，供 handler 通过统一接口调用。

### Modified Capabilities

- 无。本次为内部重构，route-integration-tests 等现有 spec 的 REQUIREMENTS 不变；仅实现方式调整（handler 调用链简化、scraping
  UA 简化为直接 get_rua、移除 lazy_static）。

## Impact

- **影响层级**：handler（精简 score_from_file、logout 移除 scraping 内部依赖）、business（新增 parse 函数或承接解析逻辑）、scraping（移除
  USER_AGENT 静态与 refresh_user_agent，new 中直接 get_rua）、Cargo.toml（移除 lazy_static）、project-context（更新依赖列表）。
- **无新增路由或错误变体**：解析函数返回 `Result<Vec<Course>, FileError>`，与 models 中 `FileError` 一致。
- **注意事项**：解析函数需与现有 Excel 模板格式兼容（Sheet1、skip(3)、列 0/1/2）；handler 从 business 的 import 中移除
  `round_2decimal`、`score_trans_grade`（解析移出 handler 后，这些仅在解析层或 business 内使用）。
