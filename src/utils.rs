// 小工具库
use base64::Engine as _;
use rust_decimal::{Decimal};
use rust_decimal_macros::dec;

// b64编码
pub fn encode_inp(text: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(text)
}

// 成绩转换绩点
pub fn score_trans_grade(score: &str) -> Option<Decimal> {
    // 返回值有两个状态, Some 表示有值返回, 括号里面是值, None 表示无值
    // 等级制的判断更简短, 先做等级制判断
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
        s if s < dec!(100) => dec!(4.67),
        _ => return None
    };

    // 到最后的必定是 grade 有值, 因为没值的在上面被返回 None 了
    // 函数末尾省略 return
    Some(grade)
}

// 保留小数点后2位
pub fn round_2decimal(d: Decimal) -> Decimal {
    d.round_dp(2)
}
