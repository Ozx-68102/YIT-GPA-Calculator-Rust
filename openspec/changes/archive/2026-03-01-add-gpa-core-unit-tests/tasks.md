## 1. 测试模块与 score_trans_grade

- [x] 1.1 在 `src/business.rs` 底部添加 `#[cfg(test)] mod tests`，并引用 `super` 与 `crate::models::Course`、`dec!`、
  `Decimal` 等所需类型
- [x] 1.2 为 `score_trans_grade` 添加等级制用例：不及格/不合格→0，及格/合格→1，中→2.33，良→3.33，优→4.33
- [x] 1.3 为 `score_trans_grade` 添加百分制边界用例：0、60、64、67、70、74、77、80、83、87、90、95、100 及若干区间代表值（如
  59、63、66）与预期绩点一致
- [x] 1.4 为 `score_trans_grade` 添加非法输入用例：""、"abc"、"60.5.5"、"101"、"-1" 等返回 `None`

## 2. round_2decimal

- [x] 2.1 为 `round_2decimal` 添加用例：保留 2 位小数、四舍五入（如 2.335→2.34）、整数值不变（如 3、4.00）

## 3. GPA 与课程排除（process_scraped_course_results）

- [x] 3.1 若通过 `process_scraped_course_results` 难以覆盖「仅 Default 排除、All 计入」的断言，将 `calculate_gpa_from_list`
  改为 `pub(crate)` 并在 tests 中直接测两 mode（可选，按需）
- [x] 3.2 添加空课程列表用例：default 与 all 的 GPA 为 0、courses 为空
- [x] 3.3 添加仅「入学教育」用例：两套结果中该课均不参与，GPA 为 0、courses 为空
- [x] 3.4 添加仅公选/通识限选用例：Default 排除、All 计入，两种 mode 结果有差异
- [x] 3.5 添加仅含「体育」等 EXCLUDED_COURSES_KEYWORD 课程用例：Default 排除、All 计入
- [x] 3.6 添加正常多门课用例：GPA 与「总 credit_gpa / 总 credit」经 round_2decimal 一致
- [x] 3.7 添加混合课程用例：同时包含计入 Default 与仅计入 All 的课，断言 default.gpa/default.courses 与
  all.gpa/all.courses 差异符合排除规则

## 4. 验证

- [x] 4.1 运行 `cargo test`，确保全部新增单元测试通过
