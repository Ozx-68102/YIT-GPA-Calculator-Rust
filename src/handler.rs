// 路由控制器
use crate::{
    business::{calculate_gpa_from_list, current_time, GPAMode},
    models::{Course, WebError},
    scraping::AAOWebsite,
    Asset,
};

use axum::{
    extract::{Form, State},
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse, Redirect, Response},
    Json
};
use mime_guess;

// 反序列化解析表单数据, 类似隔壁的 request.form
use serde::Deserialize;
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

// GPA 计算模式
#[derive(Debug, Deserialize)]
pub struct CalculateMode {
    mode: String,    // default 或 all
}

/// 用于处理 static 文件夹模板文件
pub async fn static_handler(uri: Uri) -> impl IntoResponse {
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


// 登录页面
pub async fn show_login(State(tera): State<Tera>) -> Result<Html<String>, WebError> {
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
pub async fn score_handler(session: Session, Form(form): Form<LoginForm>) -> Result<Json<serde_json::Value>, WebError> {
    #[cfg(debug_assertions)]
    println!("[{}]API /score 被调用, 准备爬取数据", current_time());

    let mut scraper = AAOWebsite::new().map_err(|e| WebError::InternalError(e.to_string()))?;

    // 初始化会话, 获得 Cookie
    scraper.init().await?;
    scraper.login(&form.account, &form.password).await?;

    let courses = scraper.get_grades().await?;

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
pub async fn show_first_result(session: Session, State(tera): State<Tera>) -> Result<impl IntoResponse, WebError> {
    #[cfg(debug_assertions)]
    println!("[{}]/result 被访问, 正在从 Session 中读取数据...", current_time());

    let courses_whole_list: Vec<Course> = session.get("courses").await.map_err(|e| WebError::InternalError(e.to_string()))?.unwrap_or_default();

    if !courses_whole_list.is_empty() {
        #[cfg(debug_assertions)]
        println!("[{}]成功从 Session 中读取到数据, 正在调用业务逻辑模块进行计算...", current_time());

        let (gpa, courses_final_list) = calculate_gpa_from_list(&courses_whole_list, GPAMode::Default);

        let mut context = tera::Context::new();
        context.insert("courses", &courses_final_list);
        context.insert("gpa", &gpa);

        let html = tera.render("result.html", &context).map_err(|e| WebError::TemplateError(e.to_string()))?;

        Ok(Html(html).into_response())
    } else {
        #[cfg(debug_assertions)]
        println!("[{}]Session 中未找到数据, 将重定向到登录页", current_time());

        Ok(Redirect::to("/").into_response())
    }
}

// 根据前端按钮重新计算 GPA
pub async fn show_next_result(session: Session, Json(cal_mode): Json<CalculateMode>) -> Result<Json<serde_json::Value>, WebError> {
    let courses_whole_list: Vec<Course> = session.get("courses").await.map_err(|e| WebError::InternalError(e.to_string()))?.unwrap_or_default();

    if courses_whole_list.is_empty() {
        return Err(WebError::InternalError("数据异常".to_string()));
    }

    let mode = match cal_mode.mode.as_str() {
        "all" => GPAMode::All,
        _ => GPAMode::Default
    };

    let (gpa, course_final_list) = calculate_gpa_from_list(&courses_whole_list, mode);

    Ok(Json(json!({"gpa": gpa, "courses": course_final_list})))
}