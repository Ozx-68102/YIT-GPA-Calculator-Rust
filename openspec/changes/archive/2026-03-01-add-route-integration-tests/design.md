# 关键路由集成测试 — 技术设计

## Context

- 项目为 YIT-GPA-Calculator-Rust，Axum + Tera，Session 使用 `tower-sessions`（MemoryStore），Cookie 使用 `tower-cookies`。路由在
  `src/router.rs` 的 `create_router(tera)` 中注册，`main.rs` 中在 `create_router(tera)` 之上叠加 `Extension(shutdown_tx)`
  、中间件（注入 cookie key）、`SessionManagerLayer`、`CookieManagerLayer`。
- `/result` 由 `handler::first_result` 处理：从 session 读取 `courses`/`gpa`，若 `courses.is_empty()` 则写入 flash 并
  `Redirect::to("/")`，否则渲染 result 模板。
- 错误码映射在 `src/models.rs` 的 `WebError::into_response` 中实现：`TemplateError`/`InternalError`/`SessionError` → 500；
  `WebScrapingError(LoginFailed)` → 401，其它爬取错误 → 500；`FileError` → 400。
- 当前无 `lib.rs`，仅 binary。集成测试在 `tests/` 中编译为独立 crate，必须依赖本项目的「库」目标才能调用 `create_router`，故需新增
  `lib.rs` 作为库入口（作用类似 Python 的 `__init__.py`）。通过 `Cargo.toml` 的 `[lib] name = "yit_gpa_tool"` 为库指定简短
  crate 名（不修改 `[package] name`）；main 与集成测试通过 `use yit_gpa_tool::...` 使用库。

## Goals / Non-Goals

**Goals:**

- 用 `axum::test` 做集成测试，验证 GET `/result` 无 session 时返回 302 且 `Location: /`。
- 将「错误码与 WebError/FileError 的对应关系」固化为可回归测试（推荐：对 `WebError::into_response` 做单元测试，覆盖
  401/400/500 等）。
- 测试文件集中在一个文件中（如 `tests/route_result.rs` 或 `tests/api.rs`），便于维护。

**Non-Goals:**

- 不启动真实 HTTP 服务器；不测真实教务网请求；不引入外部 mock 框架（若后续需要再单独设计）。
- 不改变现有 handler 或 `WebError` 的对外行为，仅增加测试与可选的最小重构（如抽取 app 构建函数）。

## Decisions

1. **lib.rs 与包名**
    - **为何需要 lib.rs**：Rust 中 `tests/*.rs` 会编译成另一个 crate，只能依赖当前项目的「库」目标。仅有 `main.rs`
      时没有库目标，集成测试无法 `use 包名::router::create_router`。新增 `lib.rs` 后，库入口明确，`main` 与 tests
      共用同一套代码；概念上类似 Python 里用 `__init__.py` 作为包入口。
    - **库名**：不修改 `[package] name`，在 `Cargo.toml` 中新增 `[lib] name = "yit_gpa_tool"`，使库的 crate 名为
      `yit_gpa_tool`。binary 与 lib 为不同编译 target，故 `main.rs` 中必须用 `use yit_gpa_tool::{ router, ... }` 引用库（不能使用
      `use crate::`，否则会报 unresolved import）。

2. **测试 app 的构建方式**
    - 在 `tests/` 中自行构造：`Tera::default()`、`MemoryStore`、`SessionManagerLayer`、`CookieManagerLayer`、
      `Key::from(&[0u8;64])` 等，然后
      `create_router(tera).layer(...).layer(session_layer).layer(CookieManagerLayer::new())`，与 `main` 的 layer 顺序一致。无
      session 重定向用例不依赖模板，`Tera::default()` 即可。
    - 若日后 layer 重复过多，可再抽取 `build_app(...)` 到库中供 `main` 与 tests 共用。

3. **无 session 访问 /result**
    - 用例：对组装好的 app 使用
      `tower::ServiceExt::oneshot(Request::get("/result").body(Body::empty()).unwrap()).await.unwrap()`，不附带 cookie。
    - 断言：`response.status().is_redirection()`（实际为 303），`headers().get("location")` 为 `"/"` 或等价。
    - 实现对应关系：与当前 `first_result` 中 `courses.is_empty()` 时 `Redirect::to("/")` 一致，无需改 handler。

4. **错误码映射的测试方式**
    - **推荐**：在 `src/models.rs` 或 `tests/` 中为 `WebError`（及通过 `WebError::FileError` 的 `FileError`）的
      `IntoResponse` 写单元测试：构造各变体，调用 `.into_response()`，断言 `status()` 为 401 / 400 /
      500。这样无需在集成测试中「触发」登录失败或模板错误，即可锁定状态码与错误类型的对应关系。
    - **可选**：若希望集成层也覆盖 400，可增加一条「上传非法文件或错误 body 导致 FileError」的集成用例（若现有接口易于在测试中触发）；401/500
      的集成触发成本较高，建议以单元测试为主。

5. **测试文件与命名**
    - 集成测试：单文件 `tests/route_result.rs` 或 `tests/api.rs`，包含「无 session GET /result → 302, Location: /」及（可选）上述
      400 集成用例。
    - 错误码：与集成同文件或单独 `tests/error_codes.rs` 均可；若放在 `src/models.rs` 的 `#[cfg(test)] mod tests`
      中，则仅单元测试，不占 `tests/`。
    - 决策：集成测试放在 `tests/route_result.rs`（或 `api.rs`）；错误码映射采用 `models.rs` 内 `#[cfg(test)]`
      单元测试，保持一处定义、一处断言。

6. **Tera 与模板**
    - `first_result` 在无 session 时不会走到 `tera.render`，因此「无 session 重定向」用例不需要真实模板。若测试 app 使用
      `Tera::default()`，该用例仍可通过。若后续用例需要渲染，再在测试中 `add_raw_template` 最小内容或复用嵌入的模板（需以
      lib 或 binary 暴露资源）。

## Risks / Trade-offs

- **Risk**：测试中重复 `main` 的 layer 顺序与 store/key 构造，若 `main` 日后改动 layer，测试可能未同步。  
  **Mitigation**：将「无 session /result」与「错误码」用例写清楚；若重复代码增多，再抽取 `build_app` 或 lib。

- **Risk**：`tower-sessions` / `tower-cookies` 在测试中的行为与生产一致性的假设不成立。  
  **Mitigation**：仅做状态码与 Location 断言；若发现差异再补充文档或调整测试环境。

- **Trade-off**：错误码以单元测试锁定，集成测试不覆盖「真实请求触发 401」等，避免引入 mock 或真实教务请求。若后续需要端到端错误路径，可再加专门设计。

## Migration Plan

- 无数据或部署迁移。合并后 CI 增加 `cargo test` 即可。
- 若后续抽取 `build_app`，保持 `main.rs` 调用 `build_app(...)`，行为不变。

## Open Questions

- 无。若采用「错误码仅在 models 单元测试」的方案，实现即可按本设计执行。
