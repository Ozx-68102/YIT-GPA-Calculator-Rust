use crate::{
    models::{Course, WebScrapingError},
    utils::{current_time, encode_inp, round_2decimal, score_trans_grade}
};
use anyhow::Result;
use fake_user_agent::get_rua;
use lazy_static::lazy_static;
use reqwest::header::HeaderValue;
use reqwest::{cookie::Cookie, header::HeaderMap, Client};
use rust_decimal::Decimal;
use scraper::{Html, Selector};
use std::collections::HashMap;

// 每次程序启动都随机加载一个 UA
lazy_static! {
    pub static ref USER_AGENT: &'static str = get_rua();
}

// 教务处网站结构体
pub struct AAOWebsite {
    client: Client, // HTTP 客户端, 相当于隔壁 Python 的 requests.Session()
    base_url: String,    // HOST
    headers: HeaderMap  // 动态管理请求头
}

// 实现结构体功能
impl AAOWebsite {
    // 创建爬虫实例
    pub fn new() -> Result<Self> {
        #[cfg(debug_assertions)]
        println!("[{}]正在初始化客户端实例", current_time());

        // 创建客户端实例, `?`表示失败就返回错误, 类似隔壁的 raise
        // 需要启动 cookie 储存
        let client = Client::builder()
            .user_agent(*USER_AGENT)    // 设置 UA
            .cookie_store(true) // 自动处理 Cookie
            .build()?;

        // cfg(debug_assertions) 表示下方紧贴着的内容只在 dev 模式下出现
        #[cfg(debug_assertions)]
        println!("[{}]客户端实例初始化完成：{:?}", current_time(), client);

        // 初始化请求头
        let mut init_headers = HeaderMap::new();
        init_headers.insert(
            "Referer",
            HeaderValue::from_static("http://yitjw.yinghuaonline.com/yjlgxy_jsxsd/kscj/cjcx_query?Ves632DSdyV=NEW_XSD_XJCJ")
        );
        init_headers.insert(
            "Content-Type",
            HeaderValue::from_static("application/x-www-form-urlencoded")
        );
        init_headers.insert(
            "Accept",
            HeaderValue::from_static("*/*")
        );

        #[cfg(debug_assertions)]
        println!("[{}]请求头设置完成：{:?}", current_time(), init_headers);

        // 用 Ok 包裹结构体则表示成功
        Ok(Self {
            client,
            base_url: "http://yitjw.yinghuaonline.com/yjlgxy_jsxsd".to_string(),
            headers: init_headers
        })
    }

    // [异步]初始化会话, 获取 cookie
    // self 前面要加 mut 因为需要更新请求头 headers
    pub async fn init(&mut self) -> Result<(), WebScrapingError> {
        #[cfg(debug_assertions)]
        println!("[{}]尝试访问：{}", current_time(), self.base_url);

        // await 表示等待请求完成, 出错会转换成自定义错误类型
        let response = self.client.get(&self.base_url)
            .headers(self.headers.clone())  // 设置请求头, 如果不使用 clone() 的话,
            .send().await.map_err(|e| WebScrapingError::HttpRequest(e.to_string()))?;

        let status_code = response.status();

        // 请求失败则报错并终止
        if !status_code.is_success() {
            return Err(WebScrapingError::HttpRequest(format!("初始化失败: {}", status_code)))
        }

        #[cfg(debug_assertions)]
        println!("[{}]访问 {} 成功！ HTTP {}。将获取 cookie", current_time(), self.base_url, response.status());

        // 获取 cookie, 找不到 cookie 也会报错并终止
        // response.cookies() 返回的是迭代器, 一旦迭代器被遍历, 它就被消耗掉了(consumed & moved)
        // 将其收集到 Vec 中即可多次访问
        let cookies: Vec<Cookie> = response.cookies().collect();
        if cookies.is_empty() { return Err(WebScrapingError::CookieInvalid) }

        #[cfg(debug_assertions)]
        println!("[{}]获取成功。cookies: {:?}", current_time(), cookies);

        // 更新 Referer, Cookie 会由 reqwest 自动管理
        self.headers.insert(
            "Referer",
            HeaderValue::from_str(&self.base_url).map_err(|e| WebScrapingError::ParseError(e.to_string()))?
        );

        #[cfg(debug_assertions)]
        println!("[{}]请求头已更新：{:?}", current_time(), self.headers);

        Ok(())
    }

    // [异步]登录系统
    // username 和 password 本来就是切片引用(&str), 所以它们已经是借用的形式, 所有权不会被消耗和移除
    // 它们的生命周期会随着其真正的拥有者(owner)被清理而移除, 在这之前它们一直存在
    pub async fn login(&mut self, username: &str, password: &str) -> Result<(), WebScrapingError> {
        #[cfg(debug_assertions)]
        println!("[{}]用户输入了登录信息[账：{}，密：{}]，将对其进行编码", current_time(), username, password);

        // b64 对账号密码进行编码
        let encoded = format!("{}%%%{}", encode_inp(username), encode_inp(password));

        #[cfg(debug_assertions)]
        println!("[{}]编码后结果：{}", current_time(), encoded);

        // 提交表单数据并登录
        let login_url = format!("{}/xk/LoginToXk", self.base_url);

        #[cfg(debug_assertions)]
        println!("[{}]现在开始提交表单数据并尝试登录，目标 URL 为 {}", current_time(), login_url);

        let form_data = [("encoded", &encoded)];
        let response = self.client.post(&login_url)
            .headers(self.headers.clone())
            .form(&form_data)
            .send().await.map_err(|e| WebScrapingError::HttpRequest(e.to_string()))?;

        let status_code = response.status();

        if !response.status().is_success() {
            return Err(WebScrapingError::HttpRequest(format!("登录异常：{}", status_code)))
        }

        // response.text() 会获取 response 的所有权并消耗(此时 response 生命周期终止）, 后续无法继续使用 response 变量
        // 因此要在所有权被消耗之前使用 url() 获取 URL
        // 该操作不会导致所有权转移(moved)
        let final_url_option = response.url().to_string();

        let response_text = response.text().await.map_err(|e| WebScrapingError::HttpRequest(e.to_string()))?;
        let login_failure_indicator = "/yjlgxy_jsxsd/xk/LoginToXk";
        if response_text.contains(login_failure_indicator) {
            return Err(WebScrapingError::LoginFailed)
        }

        #[cfg(debug_assertions)]
        println!("[{}]登录成功！ HTTP {}", current_time(), status_code);

        self.headers.insert(
            "Referer",
            HeaderValue::from_str(&final_url_option).map_err(|e| WebScrapingError::ParseError(e.to_string()))?
        );

        // 添加 x-requested-with 头
        self.headers.insert(
            "X-Requested-With",
            HeaderValue::from_static("XMLHttpRequest")
        );

        #[cfg(debug_assertions)]
        println!("[{}]请求头已更新：{:?}", current_time(), self.headers);

        Ok(())
    }

    // 获取并解析成绩, 这里不再需要更新 headers 的状态了, 所以不用 mut
    pub async fn get_grades(&self) -> Result<(Vec<Course>, Decimal), WebScrapingError> {
        // Step1. 获取成绩页面
        let grades_url = format!("{}/kscj/cjcx_list", self.base_url);

        #[cfg(debug_assertions)]
        println!("[{}]开始访问成绩页面：{}", current_time(), grades_url);

        let form_data = [("kksj", ""), ("kcxz", ""), ("kcmc", ""), ("xsfs", "all")];
        let response = self.client.post(&grades_url).form(&form_data).send().await.map_err(|e| WebScrapingError::HttpRequest(e.to_string()))?;

        let status_code = response.status();

        if !status_code.is_success() {
            return Err(WebScrapingError::HttpRequest(format!("无法访问{}：{}", grades_url, status_code)))
        }

        #[cfg(debug_assertions)]
        println!("[{}]访问成功！ HTTP {}。将获取并解析成绩单", current_time(), status_code);

        // 获取响应文本并解析
        let html_content = response.text().await.map_err(|e| WebScrapingError::HttpRequest(e.to_string()))?;
        let document = Html::parse_document(&html_content);

        // Step2. 定义排除的课程
        // vec! 代表动态数组, 类似隔壁的 list
        let excluded_courses = vec![
            "大学生安全教育", "创新创业教育", "劳动教育", "专业基础认知", "大学生心理健康教育", "形势与政策",
            "军事理论", "军事训练", "军事技能", "体育Ⅰ", "体育Ⅱ", "体育Ⅲ", "体育Ⅳ", "教育见习", "专业见习",
            "名师大讲堂", "入学教育", "毕业教育", "职业生涯规划与就业指导", "毕业实习", "教育实习", "社会实践",
            "职场体验", "领导力", "金工实习", "认知实习", "生产实习", "综合实训", "综合设计与展示", "专业认知讲座",
            "社会调研"
        ];

        // Step3. 解析 HTML 课程表格数据
        // 创建选择器, 类似隔壁 Beautiful Soup
        let tr_selector = Selector::parse("tr").map_err(|e| WebScrapingError::ParseError(e.to_string()))?;
        let td_selector = Selector::parse("td").map_err(|e| WebScrapingError::ParseError(e.to_string()))?;

        #[cfg(debug_assertions)]
        println!("[{}]解析完成，将收集成绩数据", current_time());

        // 创建[可变]哈希表, 只有 let 后面带 mut 关键字, 变量内容才可被改变, 或者说被重新赋值
        // 但作为静态强类型语言, 不论内容如何改变, 数据类型都不可变
        let mut courses_record: HashMap<String, Course> = HashMap::new();

        // 遍历所有数据行, 跳过表头行, 所以用 skip(1)
        for tr in document.select(&tr_selector).skip(1) {
            // 获取当前行的所有单元格, 过滤掉不完整的行
            let tds: Vec<_> = tr.select(&td_selector).collect();
            if tds.len() < 12 { continue }

            // 提取课程名称(在第4个单元格), 排除特定课程
            let name = tds[3].text().collect::<String>().trim().to_string();
            if excluded_courses.contains(&name.as_str()) { continue }

            // 提取总分(在第5个单元格)
            let score_text = tds[4].text().collect::<String>().trim().to_string();

            // 提取学分并且转换为 Decimal 类型
            let credit_text = tds[6].text().collect::<String>().trim().to_string();
            let credit = match credit_text.parse::<Decimal>() {
                Ok(c) => c,
                Err(_) => continue
            };

            // 转换绩点, 无效绩点则跳过
            let grade_point = match score_trans_grade(&score_text) {
                Some(g) => g,
                None => continue
            };

            // 计算加权绩点并保留后2位小数
            let credit_gpa = round_2decimal(grade_point * credit);

            // 哈希表去重: 课程存在多个, 则取较高绩点者; 否则直接插入表
            let course = Course {
                name: name.clone(),
                score: score_text,
                credit,
                grade: grade_point,
                credit_gpa
            };
            if let Some(existing) = courses_record.get_mut(&name) {
                if course.grade > existing.grade {
                    *existing = course.clone();
                }
            } else { courses_record.insert(name, course); }
        }

        #[cfg(debug_assertions)]
        println!("[{}]成绩数据收集完成，如下：\n{:?}", current_time(), courses_record);

        // 将值转为向量便于后续处理
        let course_list: Vec<_> = courses_record.into_values().collect();

        #[cfg(debug_assertions)]
        println!("[{}]已转换为向量，将开始计算总学分和加权绩点。", current_time());

        // 计算总学分和加权绩点
        let total_credits: Decimal = course_list.iter().map(|c| c.credit).sum();
        let total_cg: Decimal = course_list.iter().map(|c| c.credit_gpa).sum();

        #[cfg(debug_assertions)]
        println!("[{}]计算得出总学分 = {}，总加权绩点 = {}", current_time(), total_credits, total_cg);

        // 计算总绩点(GPA), 避免除以0引发错误
        let final_gpa = if total_credits > Decimal::ZERO { round_2decimal(total_cg / total_credits)}
        else { Decimal::ZERO };

        #[cfg(debug_assertions)]
        println!("[{}]GPA = {}", current_time(), final_gpa);

        // 返回课程列表和GPA
        Ok((course_list, final_gpa))
    }
}