# 为关键路由增加 Axum 集成测试

## Why

当前关键路由（如 `/result`）的「无 session 重定向」与「错误码映射」行为仅依赖手工验证，修改 handler 或
`WebError::into_response` 时容易漏测，导致线上出现错误状态码或重定向逻辑退化。通过引入基于 `axum::test`
的集成测试，可自动回归验证这些行为，降低回归风险。

## What Changes

- **新增** 集成测试模块（如 `tests/route_result.rs` 或 `tests/api.rs`），使用 `axum::test` 调用现有
  `router::create_router(tera)`（或封装好的 app 工厂），不启动真实服务器。
- **无 session 访问 /result**：GET `/result` 且无 session 时，断言响应为 302，`Location: /`（与当前 `first_result` 中
  `courses.is_empty()` 时 `Redirect::to("/")` 一致）。
- **错误码映射（可选但推荐）**：为关心的错误类型编写少量用例，验证 HTTP 状态码符合预期：
    - `WebError::WebScrapingError(LoginFailed)` → 401
    - `FileError`（经 `WebError::FileError`）→ 400
    - `TemplateError` / `InternalError` → 500
- **lib.rs**：新增 `src/lib.rs` 作为库入口（类似 Python 包里的 `__init__.py`），使本 crate 拥有「库」目标；集成测试在 `tests/`
  中作为独立 crate 依赖该库，才能调用 `create_router(tera)` 等。binary 与 lib 为不同编译 target，故 `main.rs` 通过
  `use yit_gpa_tool::{ router, TemplateAsset, ... }` 使用库（不能使用 `use crate::`）。
- **库名**：不修改 `[package] name`，在 `Cargo.toml` 中新增 `[lib] name = "yit_gpa_tool"`，使库的 crate 名为 `yit_gpa_tool`
  ，main 与集成测试中写 `use yit_gpa_tool::...` 引用。
- 测试所需 Tera 与 Session/Cookie 层需在测试内构造，与 `main` 中 app 构建方式一致。
- **影响层次**：仅新增测试代码与 lib 入口（router/handler 逻辑不变）；不改变对外 API。

## Capabilities

### New Capabilities

- `route-integration-tests`：关键路由的 Axum 集成测试能力。覆盖「无 session 访问 /result 重定向到 /」以及「WebError/FileError
  与 HTTP 状态码的映射」的可回归验证。

### Modified Capabilities

- 无。仅新增测试与可选的应用工厂抽取，不修改现有 spec 级需求。

## Impact

- **代码**：新增 `src/lib.rs` 作为库入口（`pub use` router、TemplateAsset/BinaryAsset 等），供 `main` 与集成测试共用；新增
  `tests/` 下集成测试文件；在 `Cargo.toml` 中新增 `[lib] name = "yit_gpa_tool"`（不修改 `[package] name`）。
- **依赖**：测试使用与 `main` 相同的 `tower-sessions`、`tower-cookies`、`axum` 等；集成测试通过依赖本 crate 的库目标调用
  `create_router`。
- **系统**：无对外行为变更；CI 中可增加 `cargo test` 以运行上述集成测试。
