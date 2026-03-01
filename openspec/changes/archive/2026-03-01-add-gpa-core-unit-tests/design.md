## Context

`src/business.rs` 当前包含 GPA 核心逻辑：`score_trans_grade`（等级制/百分制→绩点）、`round_2decimal`（保留两位小数）、
`calculate_gpa_from_list`（按模式过滤课程并计算加权 GPA）、`process_scraped_course_results`（对外暴露 Default/All
两套结果）。这些函数无单元测试，重构或修 bug 时易引入回归。project-context 要求 GPA/学分一律使用 Decimal 与 dec!，课程排除仅通过
business 内常量；测试需与现有常量及分段规则一致。

## Goals / Non-Goals

**Goals:**

- 在 `src/business.rs` 内用 `#[cfg(test)] mod tests` 锁死 `score_trans_grade`、`round_2decimal` 的边界与非法输入行为。
- 通过 `process_scraped_course_results`（及必要时的 `calculate_gpa_from_list`
  可见性调整）锁死：空列表、PERMANENT_IGNORED/ATTR_EXCLUSIONS/EXCLUDED_COURSES_KEYWORD 排除、Default vs All 差异、加权平均与
  round_2decimal 一致性。
- 不改变现有对外 API；测试仅编译进 `cargo test`，不影响生产二进制。

**Non-Goals:**

- 不新增集成测试或 E2E；不覆盖 handler/scraping 层；不修改 router、models、错误类型。
- 不改变等级制/百分制分段或排除常量的业务规则本身，仅用测试固化当前行为。

## Decisions

1. **测试位置与可见性**
    - **决策**：在 `src/business.rs` 底部增加 `#[cfg(test)] mod tests`，直接测 `score_trans_grade`、`round_2decimal`
      ；GPA/排除逻辑优先通过 `process_scraped_course_results(..., ResultSource::OfficialWebsite)` 构造 `Course` 列表验证
      Default 与 All 结果。
    - **备选**：将 `calculate_gpa_from_list` 改为 `pub(crate)` 并在同 crate 的 tests 中直接调用。
    - **理由**：优先不扩大可见性；若仅通过 `process_scraped_course_results` 难以构造「仅 All 含、Default 排除」的断言（因
      Default 由 source 决定），可再改为 `pub(crate)` 并直接测 `calculate_gpa_from_list`。实现时若发现需要直接测两套 mode
      的 (gpa, courses) 更方便，则采用 `pub(crate)`。

2. **测试数据与常量**
    - **决策**：测试中使用的课程名称、属性、分数、学分与 `Course` 字段，均与 `PERMANENT_IGNORED_COURSES`、`ATTR_EXCLUSIONS`、
      `EXCLUDED_COURSES_KEYWORD` 及现有百分制/等级制规则一致；不复制常量值到测试，直接使用 `super::` 引用 business 内常量。
    - **理由**：避免测试与实现不同步；project-context 要求排除规则只在 business 内扩展。

3. **Decimal 与断言**
    - **决策**：期望绩点/学分一律用 `dec!` 字面量；比较用 `assert_eq!(result, dec!(x.xx))` 或已构造的 `Decimal`。
    - **理由**：与 project-context 数值规范一致，避免浮点误差。

## Risks / Trade-offs

- **[Risk]** 若将来调整百分制分段或等级制映射，需同步改测试，否则 CI 报错。  
  **Mitigation**：在 spec 与 tasks 中注明「边界与 business 常量一致」；变更规则时同一 MR 内更新测试与实现。

- **[Trade-off]** 不直接测 `calculate_gpa_from_list` 时，部分边界（如「仅公选」在 Default 下 courses 为空）需通过
  `process_scraped_course_results` 的 `default`/`all` 输出间接断言，可读性略差。  
  **Mitigation**：若实现时发现断言冗长或难以覆盖，采纳「将 `calculate_gpa_from_list` 改为 `pub(crate)` 并直接测」的方案。

## Migration Plan

无需迁移或回滚：仅新增测试代码与可选可见性调整，无配置、数据或 API 变更。合并后运行 `cargo test` 通过即可。

## Open Questions

- 无。是否将 `calculate_gpa_from_list` 改为 `pub(crate)` 在实现阶段按需决定即可。
