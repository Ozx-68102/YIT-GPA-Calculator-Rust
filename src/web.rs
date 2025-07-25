use axum::{
    extract::{Form, State}, // 提取器: Form 提取表单数据, State 共享状态
    response::Html, // 响应类型: Html 包装 HTML 字符串
    routing::{get, post},   // 路由方法: get 处理 GET 请求, post 处理 POST 请求
    Router  // 路由管理器, 类似隔壁的 Flask app.py
};
// 反序列化解析表单数据, 类似隔壁的 request.form
use serde::Deserialize;
// 模板引擎, 类似 Jinja2
use tera::Tera;

use crate::{
    models::WebError,
    utils::current_time,
    web_scraping::AAOWebsite
};

// 对应前端登录表单的两个字段
#[derive(Debug, Deserialize)]
pub struct LoginForm {
    account: String,
    password: String
}

// 定义路由
pub fn create_router(tera: Tera) -> Router {
    Router::new()
        .route("/", get(login_page))    // 根目录是登录页面
        .route("/score", post(handle_score))    // 登录后显示计算后学分
        .with_state(tera)   // 将 Tera 模板引擎作为共享状态以便所有路由处理器都能访问
}

// 登录页面
pub async fn login_page(State(tera): State<Tera>) -> Result<Html<String>, WebError> {
    #[cfg(debug_assertions)]
    println!("[{}]开始渲染登录界面", current_time());

    let context = tera::Context::new();
    let html = tera.render("login.html", &context).map_err(|e| WebError::TemplateError(e.to_string()))?;

    #[cfg(debug_assertions)]
    println!("[{}]登录界面渲染成功", current_time());

    Ok(Html(html))
}

// 外层函数用于捕获可能的错误
pub async fn handle_score(State(tera): State<Tera>, Form(form): Form<LoginForm>) -> Result<Html<String>, WebError> {
    match inner_handle_score(State(tera.clone()), Form(form)).await {
        Ok(html) => Ok(html),
        Err(error) => {
            // 获取错误信息
            let error_msg = error.to_string();

            // 存入 tera 上下文
            let mut context = tera::Context::new();
            context.insert("error_msg", &error_msg);

            // 渲染页面并且传入上下文
            let err_html = tera.render("error.html", &context).unwrap_or_else(|_| "无法加载错误页面".to_string());

            Ok(Html(err_html))
        }
    }
}

// 内层函数用于 GPA 查询处理
async fn inner_handle_score(State(tera): State<Tera>, Form(form): Form<LoginForm>) -> Result<Html<String>, WebError> {
    #[cfg(debug_assertions)]
    println!("[{}]准备初始化网页抓取爬虫与会话(Session)", current_time());
    // 初始化爬虫
    // 这里用 mut 的原因是 headers 的变动
    let mut scraper = AAOWebsite::new().map_err(|e| WebError::InternalError(e.to_string()))?;

    // 初始化会话, 获取 Cookie
    scraper.init().await?;

    #[cfg(debug_assertions)]
    println!("[{}]即将执行登录操作", current_time());

    // 用表单中的账号密码登录
    scraper.login(&form.account, &form.password).await?;

    #[cfg(debug_assertions)]
    println!("[{}]登录操作完成，将获取课程数据与GPA", current_time());

    // 获取课程成绩和 GPA
    let (courses, gpa) = scraper.get_grades().await?;

    #[cfg(debug_assertions)]
    println!("[{}]开始渲染 GPA 页面", current_time());

    // 渲染结果页面
    let mut context = tera::Context::new();
    context.insert("courses", &courses);
    context.insert("gpa", &gpa);
    let html = tera.render("result.html", &context).map_err(|e| WebError::TemplateError(e.to_string()))?;

    #[cfg(debug_assertions)]
    println!("[{}]渲染 GPA 界面成功", current_time());

    // 成功则返回网页内容
    Ok(Html(html))
}