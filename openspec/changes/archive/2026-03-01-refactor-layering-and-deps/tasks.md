## 1. Excel 解析抽离到 business

- [x] 1.1 在 `src/business.rs` 中新增
  `parse_courses_from_excel_template(bytes: &[u8]) -> Result<Vec<Course>, FileError>`，实现 Sheet1、skip(3)、列 0/1/2
  解析逻辑，使用 `score_trans_grade`、`round_2decimal`、`Course`；返回 `FileError::OpenError` 或 `NoValidDataFound`
- [x] 1.2 在 `src/business.rs` 中添加 `calamine`、`crate::models::FileError`、`std::io::Cursor` 的 use（calamine 已在 crate
  依赖中）
- [x] 1.3 精简 `handler::score_from_file`：从 multipart 取 `gpa_file` 字段的 bytes → 调用
  `business::parse_courses_from_excel_template` → 调用 `process_scraped_course_results` → 写入 session
- [x] 1.4 从 handler 的 business import 中移除 `round_2decimal`、`score_trans_grade`；移除 `calamine`、`std::io::Cursor`
  等解析相关 import

## 2. UA 简化

- [x] 2.1 在 `src/scraping.rs` 中移除 `USER_AGENT` 静态变量、`refresh_user_agent()`；`AAOWebsite::new()` 直接调用
  `get_rua().to_string()` 作为 client 的 user_agent
- [x] 2.2 在 `handler::logout` 中移除对 `refresh_user_agent`、`USER_AGENT`、`fake_user_agent::get_rua` 的依赖；logout 仅销毁
  session

## 3. 移除 lazy_static 依赖

- [x] 3.1 从 `Cargo.toml` 移除 `lazy_static` 依赖
- [x] 3.2 在 `_bmad-output/project-context.md` 的 Technology Stack 中移除 lazy_static 条目

## 4. 验证

- [x] 4.1 运行 `cargo build` 与 `cargo test`，确保编译通过且现有测试不变
