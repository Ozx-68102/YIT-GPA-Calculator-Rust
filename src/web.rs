use axum::{
    extract::{Form, State},
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post}, Json, Router
};
use mime_guess;
use rust_decimal::Decimal;

// 反序列化解析表单数据, 类似隔壁的 request.form
use serde::Deserialize;

use crate::{
    models::{Course, WebError},
    utils::{current_time, round_2decimal},
    web_scraping::AAOWebsite,
    Asset
};

use serde_json::json;

// 模板引擎, 类似 Jinja2
use tera::Tera;

use tower_sessions::Session;

// 对应前端登录表单的两个字段
#[derive(Debug, Deserialize)]
pub struct LoginForm {
    account: String,
    password: String
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches("/");

    if path.is_empty() {
        return (StatusCode::NOT_FOUND, "Not Found").into_response();
    }

    match Asset::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();

            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(content.data.into())
                .unwrap()
        }
        None => (StatusCode::NOT_FOUND, "Not Found").into_response()
    }
}

// 定义路由
pub fn create_router(tera: Tera) -> Router {
    Router::new()
        .route("/", get(login_page))    // 根目录是登录页面
        .route("/score", post(handle_score))    // 这是回传登录数据的 API 接口
        .route("/result", get(show_result)) // 显示计算后学分
        .fallback(static_handler)   // 自动加载并注册 static 的资源
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
    println!("[{}]登录界面被访问", current_time());

    Ok(Html(html))
}

// 负责登录与爬取数据, 然后将数据存入 Session, 并返回 JSON
pub async fn handle_score(session: Session, Form(form): Form<LoginForm>) -> Result<Json<serde_json::Value>, WebError> {
    #[cfg(debug_assertions)]
    println!("[{}]API /score 被调用, 准备爬取数据", current_time());

    let mut scraper = AAOWebsite::new().map_err(|e| WebError::InternalError(e.to_string()))?;

    // 初始化会话, 获得 Cookie
    scraper.init().await?;
    scraper.login(&form.account, &form.password).await?;

    let (courses, _init_gpa) = scraper.get_grades().await?;

    #[cfg(debug_assertions)]
    println!("[{}]数据爬取成功, 共{}门课程. 存入 Session 中...", current_time(), courses.len());

    // 将课程存入 Session
    session.insert("courses", courses).await.map_err(|e| WebError::InternalError(e.to_string()))?;

    #[cfg(debug_assertions)]
    println!("[{}]存入 Session 成功", current_time());

    // 返回成功的信号
    Ok(Json(json!({"success": true})))
}

// 负责从 Session 读取数据并返回给前端
pub async fn show_result(session: Session, State(tera): State<Tera>) -> Result<impl IntoResponse, WebError> {
    #[cfg(debug_assertions)]
    println!("[{}]/result 被访问, 正在从 Session 中读取数据...", current_time());

    let courses: Vec<Course> = session.get("courses").await.map_err(|e| WebError::InternalError(e.to_string()))?.unwrap_or_default();

    if !courses.is_empty() {
        #[cfg(debug_assertions)]
        println!("[{}]成功从 Session 中读取到数据, 开始计算 GPA 并渲染页面...", current_time());

        let total_credits: Decimal = courses.iter().map(|c| c.credit).sum();
        let total_cg: Decimal = courses.iter().map(|c| c.credit_gpa).sum();
        let gpa = if total_credits > Decimal::ZERO {
            round_2decimal(total_cg / total_credits)
        } else {
            Decimal::ZERO
        };

        let mut context = tera::Context::new();
        context.insert("courses", &courses);
        context.insert("gpa", &gpa);

        let html = tera.render("result.html", &context).map_err(|e| WebError::TemplateError(e.to_string()))?;

        Ok(Html(html).into_response())
    } else {
        #[cfg(debug_assertions)]
        println!("[{}]Session 中未找到数据, 将重定向到登录页", current_time());

        Ok(Redirect::to("/").into_response())
    }
}