// 业务逻辑层 - 处理获取到的数据
use crate::models::Course;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::Local;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

pub const PERMANENT_IGNORED_COURSES: &[&str] = &["入学教育"];

// 都一样, 公共选修课的性质对应 "公选" 属性, 通识教育选修同理
pub const ATTR_EXCLUSIONS: &[&str] = &["公选", "通识限选"];

pub const EXCLUDED_COURSES_KEYWORD: &[&str] = &[
    "体育", "职业生涯规划与就业指导", "大学生安全教育", "大学生心理健康教育",
    "形势与政策", "军事理论", "军事训练", "军事技能", "创新创业教育",
    "劳动教育", "专业基础认知", "毕业教育", "社会实践", "社会调研",
    "综合实训", "综合设计与展示", "职场体验", "实习", "见习",
    "名师大讲堂", "领导力", "系列讲座"
];

// 绩点计算模式
enum GPAMode {
    Default,    // 默认模式 - 排除部分课程的 GPA
    All,        // 完全模式 - 计算所有课程的 GPA
}

// 数据来源
pub enum ResultSource {
    OfficialWebsite,    // 登录获取
    InputFile,          // 导入文件计算
}

// 绩点计算信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPAResult {
    pub gpa: Decimal,
    pub courses: Vec<Course>,
}

// 不同模式的绩点计算信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedGPAResults {
    pub default: Option<GPAResult>, // 可能不存在
    pub all: GPAResult,             // 必定存在
}



/// base64 编码
pub fn b64_encode(text: &str) -> String {
    STANDARD.encode(text)
}

/// 成绩转换绩点
pub fn score_trans_grade(score: &str) -> Option<Decimal> {
    // 返回值有两个状态, Some 表示有值返回, 括号里面是值, None 表示无值
    // 等级制的判断更简短, 为了方便先做等级制判断
    match score {
        "不及格" | "不合格" => return Some(Decimal::ZERO),
        "及格" | "合格" => return Some(Decimal::ONE),
        "中" => return Some(dec!(2.33)),
        "良" => return Some(dec!(3.33)),
        "优" => return Some(dec!(4.33)),
        _ => {} // 默认值, 空括号表示不处理, 执行下面的代码
    }

    // parse::<Decimal> 表示转换成 Decimal 类型
    // Ok 表示成功, 箭头后面表示要赋的值
    // Err 表示失败, 返回空值 None
    let score_val = match score.parse::<Decimal>() {
        Ok(val) => val,
        Err(_) => return None
    };

    if score_val < Decimal::ZERO || score_val > dec!(100) {
        return None;
    }

    // match 从上到下匹配, s 表示一个变量(可以自己取别的名字), 后面if补充条件
    // 性能比 if-else 语句略好
    let grade = match score_val {
        s if s < dec!(60) => Decimal::ZERO,
        s if s < dec!(64) => dec!(1.33),
        s if s < dec!(67) => dec!(1.67),
        s if s < dec!(70) => dec!(2.00),
        s if s < dec!(74) => dec!(2.33),
        s if s < dec!(77) => dec!(2.67),
        s if s < dec!(80) => dec!(3.00),
        s if s < dec!(83) => dec!(3.33),
        s if s < dec!(87) => dec!(3.67),
        s if s < dec!(90) => dec!(4.00),
        s if s < dec!(95) => dec!(4.33),
        s if s <= dec!(100) => dec!(4.67),
        _ => return None
    };

    // 到最后的必定是 grade 有值, 因为没值的在上面被返回 None 了
    // 函数末尾省略 return
    Some(grade)
}

/// 保留小数点后2位
pub fn round_2decimal(d: Decimal) -> Decimal {
    d.round_dp(2)
}

/// 提供当前时间
fn current_time() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S%.6f").to_string()
}


/// 计算GPA
fn calculate_gpa_from_list(courses: &[Course], mode: GPAMode) -> (Decimal, Vec<Course>) {
    let courses: Vec<Course> = courses
        .iter()
        .filter(|c| !PERMANENT_IGNORED_COURSES.contains(&c.name.as_str()))
        .cloned()
        .collect();

    let courses_to_use: Vec<Course> = match mode {
        GPAMode::Default => {
            courses.iter()
                .filter(|c|
                    !EXCLUDED_COURSES_KEYWORD.iter().any(|k| c.name.contains(k))
                        && !ATTR_EXCLUSIONS.contains(&c.attr.as_str())
                ).cloned().collect()
        }
        GPAMode::All => { courses.to_vec() }
    };

    let total_credits: Decimal = courses_to_use.iter().map(|c| c.credit).sum();
    let total_cg: Decimal = courses_to_use.iter().map(|c| c.credit_gpa).sum();
    let gpa: Decimal = if total_credits > Decimal::ZERO {
        round_2decimal(total_cg / total_credits)
    } else {
        Decimal::ZERO
    };

    (gpa, courses_to_use)
}

/// 根据数据获取方式计算所有结果
pub fn process_scraped_course_results(courses: &[Course], source: ResultSource) -> ProcessedGPAResults {
    // 先计算 All 模式的结果
    let all_result = {
        let (gpa_all, courses_all) = calculate_gpa_from_list(&courses, GPAMode::All);

        GPAResult { gpa: gpa_all, courses: courses_all }
    };

    // 根据数据来源决定是否需要计算 Default 模式
    let default_result = match source {
        ResultSource::OfficialWebsite => {
            let (gpa_default, courses_default) = calculate_gpa_from_list(&courses, GPAMode::Default);

            Some(GPAResult { gpa: gpa_default, courses: courses_default })
        }
        ResultSource::InputFile => None
    };

    ProcessedGPAResults {
        default: default_result,
        all: all_result,
    }
}

/// 格式化信息
pub fn format_log_msg(msg: &str) -> String {
    format!("[{}]{}", current_time(), msg)
}

/// 打印正常信息
pub fn print_info(msg: &str) {
    println!("{}", format_log_msg(msg));
}

/// 打印异常信息
pub fn print_error(msg: &str) {
    eprintln!("{}", format_log_msg(msg));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_course(name: &str, attr: &str, score: &str, credit: Decimal, grade: Decimal) -> Course {
        Course {
            name: name.to_string(),
            attr: attr.to_string(),
            score: score.to_string(),
            credit,
            grade,
            credit_gpa: round_2decimal(credit * grade),
        }
    }

    // ---------- score_trans_grade：等级制 ----------
    #[test]
    fn score_trans_grade_level_fail() {
        assert_eq!(score_trans_grade("不及格"), Some(Decimal::ZERO));
        assert_eq!(score_trans_grade("不合格"), Some(Decimal::ZERO));
    }

    #[test]
    fn score_trans_grade_level_pass() {
        assert_eq!(score_trans_grade("及格"), Some(Decimal::ONE));
        assert_eq!(score_trans_grade("合格"), Some(Decimal::ONE));
    }

    #[test]
    fn score_trans_grade_level_mid_good_excellent() {
        assert_eq!(score_trans_grade("中"), Some(dec!(2.33)));
        assert_eq!(score_trans_grade("良"), Some(dec!(3.33)));
        assert_eq!(score_trans_grade("优"), Some(dec!(4.33)));
    }

    // ---------- score_trans_grade：百分制边界 ----------
    #[test]
    fn score_trans_grade_percent_boundaries() {
        assert_eq!(score_trans_grade("0"), Some(Decimal::ZERO));
        assert_eq!(score_trans_grade("59"), Some(Decimal::ZERO));
        assert_eq!(score_trans_grade("60"), Some(dec!(1.33)));
        assert_eq!(score_trans_grade("63"), Some(dec!(1.33)));
        assert_eq!(score_trans_grade("64"), Some(dec!(1.67)));
        assert_eq!(score_trans_grade("66"), Some(dec!(1.67)));
        assert_eq!(score_trans_grade("67"), Some(dec!(2.00)));
        assert_eq!(score_trans_grade("70"), Some(dec!(2.33)));
        assert_eq!(score_trans_grade("74"), Some(dec!(2.67)));
        assert_eq!(score_trans_grade("77"), Some(dec!(3.00)));
        assert_eq!(score_trans_grade("80"), Some(dec!(3.33)));
        assert_eq!(score_trans_grade("83"), Some(dec!(3.67)));
        assert_eq!(score_trans_grade("87"), Some(dec!(4.00)));
        assert_eq!(score_trans_grade("90"), Some(dec!(4.33)));
        assert_eq!(score_trans_grade("95"), Some(dec!(4.67)));
        assert_eq!(score_trans_grade("100"), Some(dec!(4.67)));
    }

    // ---------- score_trans_grade：非法输入 ----------
    #[test]
    fn score_trans_grade_invalid() {
        assert_eq!(score_trans_grade(""), None);
        assert_eq!(score_trans_grade("abc"), None);
        assert_eq!(score_trans_grade("60.5.5"), None);
        assert_eq!(score_trans_grade("101"), None);
        assert_eq!(score_trans_grade("-1"), None);
    }

    // ---------- round_2decimal ----------
    #[test]
    fn round_2decimal_keeps_two_dp() {
        assert_eq!(round_2decimal(dec!(2.333)), dec!(2.33));
        assert_eq!(round_2decimal(dec!(1.111)), dec!(1.11));
    }

    #[test]
    fn round_2decimal_rounds_half_up() {
        assert_eq!(round_2decimal(dec!(2.335)), dec!(2.34));
        assert_eq!(round_2decimal(dec!(2.334)), dec!(2.33));
    }

    #[test]
    fn round_2decimal_integer_unchanged() {
        assert_eq!(round_2decimal(dec!(3)), dec!(3.00));
        assert_eq!(round_2decimal(dec!(4.00)), dec!(4.00));
    }

    // ---------- process_scraped_course_results：空列表 ----------
    #[test]
    fn process_empty_list() {
        let courses: Vec<Course> = vec![];
        let out = process_scraped_course_results(&courses, ResultSource::OfficialWebsite);
        assert_eq!(out.default.as_ref().unwrap().gpa, Decimal::ZERO);
        assert!(out.default.as_ref().unwrap().courses.is_empty());
        assert_eq!(out.all.gpa, Decimal::ZERO);
        assert!(out.all.courses.is_empty());
    }

    // ---------- 仅「入学教育」被永久排除 ----------
    #[test]
    fn process_only_permanent_ignored() {
        let courses = vec![make_course("入学教育", "必修", "合格", dec!(1), dec!(1))];
        let out = process_scraped_course_results(&courses, ResultSource::OfficialWebsite);
        assert_eq!(out.default.as_ref().unwrap().gpa, Decimal::ZERO);
        assert!(out.default.as_ref().unwrap().courses.is_empty());
        assert_eq!(out.all.gpa, Decimal::ZERO);
        assert!(out.all.courses.is_empty());
    }

    // ---------- 仅公选/通识限选：Default 排除、All 计入 ----------
    #[test]
    fn process_only_attr_excluded_default_excludes() {
        let courses = vec![
            make_course("某公选课", "公选", "85", dec!(2), dec!(3.33)),
            make_course("某通识限选", "通识限选", "80", dec!(1), dec!(3.33)),
        ];
        let out = process_scraped_course_results(&courses, ResultSource::OfficialWebsite);
        assert_eq!(out.default.as_ref().unwrap().gpa, Decimal::ZERO);
        assert!(out.default.as_ref().unwrap().courses.is_empty());
        assert_eq!(out.all.courses.len(), 2);
        let expected_all = round_2decimal((dec!(2) * dec!(3.33) + dec!(1) * dec!(3.33)) / dec!(3));
        assert_eq!(out.all.gpa, expected_all);
    }

    // ---------- 仅含体育等关键词：Default 排除、All 计入 ----------
    #[test]
    fn process_only_keyword_excluded_default_excludes() {
        let courses = vec![make_course("体育", "必修", "90", dec!(1), dec!(4.33))];
        let out = process_scraped_course_results(&courses, ResultSource::OfficialWebsite);
        assert_eq!(out.default.as_ref().unwrap().gpa, Decimal::ZERO);
        assert!(out.default.as_ref().unwrap().courses.is_empty());
        assert_eq!(out.all.gpa, dec!(4.33));
        assert_eq!(out.all.courses.len(), 1);
    }

    // ---------- 正常多门课：GPA 与 round_2decimal(总 credit_gpa / 总 credit) 一致 ----------
    #[test]
    fn process_normal_courses_gpa_consistent() {
        let courses = vec![
            make_course("高数", "必修", "85", dec!(4), dec!(3.33)),
            make_course("英语", "必修", "78", dec!(2), dec!(2.67)),
        ];
        let out = process_scraped_course_results(&courses, ResultSource::OfficialWebsite);
        let total_cg = dec!(4) * dec!(3.33) + dec!(2) * dec!(2.67);
        let total_c = dec!(6);
        let expected = round_2decimal(total_cg / total_c);
        assert_eq!(out.default.as_ref().unwrap().gpa, expected);
        assert_eq!(out.all.gpa, expected);
        assert_eq!(out.default.as_ref().unwrap().courses.len(), 2);
        assert_eq!(out.all.courses.len(), 2);
    }

    // ---------- 混合课程：Default 与 All 结果差异 ----------
    #[test]
    fn process_mixed_default_vs_all() {
        let courses = vec![
            make_course("高数", "必修", "85", dec!(4), dec!(3.33)),
            make_course("体育", "必修", "90", dec!(1), dec!(4.33)),
        ];
        let out = process_scraped_course_results(&courses, ResultSource::OfficialWebsite);
        assert_eq!(out.default.as_ref().unwrap().courses.len(), 1);
        assert_eq!(out.all.courses.len(), 2);
        assert_ne!(out.default.as_ref().unwrap().gpa, out.all.gpa);
        assert_eq!(out.default.as_ref().unwrap().gpa, dec!(3.33));
        let all_cg = dec!(4) * dec!(3.33) + dec!(1) * dec!(4.33);
        assert_eq!(out.all.gpa, round_2decimal(all_cg / dec!(5)));
    }
}