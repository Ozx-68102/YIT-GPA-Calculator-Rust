# 实现任务：关键路由集成测试

## 1. 库入口与包名（供集成测试依赖）

- [x] 1.1 在 `Cargo.toml` 中保留 `[package] name` 不变，新增 `[lib] name = "yit_gpa_tool"` 作为库的 crate 名；新增
  `src/lib.rs` 作为库入口（`pub mod` + `pub use create_router`，以及 `TemplateAsset`/`BinaryAsset`）。`main.rs` 因 binary 与
  lib 为不同编译 target，改为通过 `use yit_gpa_tool::{ router, TemplateAsset, ... }` 使用库（不能使用 `use crate::`，否则会报错）
- [x] 1.2 确认 `Cargo.toml` 中 `axum`、`tower-sessions`、`tower-cookies` 等依赖在集成测试中可用；集成测试需增加
  dev-dependency `tower`（供 `ServiceExt::oneshot`）

## 2. 错误码映射的单元测试

- [x] 2.1 在 `src/models.rs` 中增加 `#[cfg(test)] mod tests`，对 `WebError::into_response()` 做单元测试：构造
  `WebError::WebScrapingError(WebScrapingError::LoginFailed)`，断言响应状态码为 401
- [x] 2.2 同上，构造 `WebError::FileError(FileError::OpenError(...))` 或 `NoValidDataFound`，断言状态码为 400
- [x] 2.3 同上，构造 `WebError::TemplateError(...)` 与 `WebError::InternalError(...)`，断言状态码为 500
- [x] 2.4 同上，构造 `WebError::WebScrapingError(WebScrapingError::ParseError(...))`（或其它非 LoginFailed 变体），断言状态码为
  500

## 3. 集成测试：无 session 访问 /result

- [x] 3.1 新增 `tests/route_result.rs`，在测试内构建与 `main` 一致的 app（`Tera::default()`、`MemoryStore`、
  `SessionManagerLayer`、`CookieManagerLayer`、固定 `Key`、注入 key 的 middleware），通过
  `yit_gpa_tool::create_router(tera).layer(...)` 组装；使用 `tower::ServiceExt::oneshot` 发请求（未使用 TestClient）
- [x] 3.2 编写用例：对上述 app 发起 GET `/result` 且不附带 session cookie，断言 `status.is_redirection()` 且 `Location` 头为
  `/`（实际为 303，用 is_redirection() 兼容）

## 4. 收尾与验证

- [x] 4.1 运行 `cargo test`，确认单元测试与集成测试均通过
- [x] 4.2 若项目有 CI 配置，在 CI 中增加或保留 `cargo test` 步骤
