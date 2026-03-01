# Excel 模板解析规格

## ADDED Requirements

### Requirement: 从 Excel 字节流解析为课程列表

系统 SHALL 提供函数 `parse_courses_from_excel_template(bytes: &[u8]) -> Result<Vec<Course>, FileError>`，将符合模板格式的
Excel 字节流解析为 `Vec<Course>`。解析 MUST 遵循：工作簿为 Sheet1、跳过前 3 行、第 0 列为课程名称、第 1 列为学分、第 2
列为成绩；空行或无效行跳过；成绩经 `score_trans_grade` 转换、加权绩点经 `round_2decimal` 计算；GPA/学分一律使用
`rust_decimal::Decimal`。

#### Scenario: 有效模板解析成功

- **WHEN** 传入符合模板格式的 Excel 字节流（Sheet1、前 3 行为表头、后续行含课程名/学分/成绩）
- **THEN** 返回 `Ok(courses)`，其中 `courses` 为非空 `Vec<Course>`，每门课含正确的 name、credit、score、grade、credit_gpa

#### Scenario: 解析失败返回 OpenError

- **WHEN** 传入非 Excel 格式或损坏的字节流，或无法打开/解析为 Xlsx
- **THEN** 返回 `Err(FileError::OpenError(...))`

#### Scenario: 无有效数据返回 NoValidDataFound

- **WHEN** 解析后 `Vec<Course>` 为空（无有效行或所有行被跳过）
- **THEN** 返回 `Err(FileError::NoValidDataFound)`

#### Scenario: 无效行被跳过

- **WHEN** 某行课程名、学分或成绩为空，或学分无法 parse 为 Decimal，或成绩经 score_trans_grade 返回 None
- **THEN** 该行不加入结果，不影响其他有效行的解析
