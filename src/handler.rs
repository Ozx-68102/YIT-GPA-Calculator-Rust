// 路由控制器
use crate::{
    business::{calculate_gpa_from_list, print_error, print_info, GPAMode},
    models::{Course, WebError},
    scraping::{AAOWebsite, USER_AGENT},
    Asset,
};

use axum::{
    extract::{Form, State},
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse, Redirect, Response},
    Extension,
    Json
};
use fake_user_agent::get_rua;
use mime_guess;

// 反序列化解析表单数据, 类似隔壁的 request.form
use serde::Deserialize;
use serde_json::json;

// 模板引擎, 类似 Jinja2
use tera::Tera;
use tokio::sync::broadcast;
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
pub async fn show_login(session: Session, State(tera): State<Tera>) -> Result<Html<String>, WebError> {
    #[cfg(debug_assertions)]
    print_info("开始渲染登录界面");

    let mut context = tera::Context::new();

    let flash_msg: Option<String> = session.remove("flash_msg").await.map_err(|e| WebError::InternalError(e.to_string()))?;
    if let Some(msg) = flash_msg {
        context.insert("flash_msg", &msg);
        print_error(&format!("检测到异常消息: {}", msg));
    }

    let html = tera.render("login.html", &context).map_err(|e| WebError::TemplateError(e.to_string()))?;

    #[cfg(debug_assertions)]
    print_info("渲染成功");

    #[cfg(not(debug_assertions))]
    print_info("登录界面被访问");

    Ok(Html(html))
}

// 负责登录与爬取数据, 然后将数据存入 Session, 并返回 JSON
pub async fn score_handler(session: Session, Form(form): Form<LoginForm>) -> Result<Json<serde_json::Value>, WebError> {
    #[cfg(debug_assertions)]
    print_info("API /score 被调用, 准备爬取数据");

    #[cfg(not(debug_assertions))]
    print_info("正在登录中...");

    let mut scraper = AAOWebsite::new().map_err(|e| WebError::InternalError(e.to_string()))?;

    // 初始化会话, 获得 Cookie
    scraper.init().await?;
    scraper.login(&form.account, &form.password).await?;

    #[cfg(not(debug_assertions))]
    print_info("登录成功");

    let courses = scraper.get_grades().await?;

    #[cfg(debug_assertions)]
    print_info(&format!("数据爬取成功, 共{}门课程. 存入 Session 中...", courses.len()));

    // 将课程存入 Session
    session.insert("courses", courses).await.map_err(|e| WebError::InternalError(e.to_string()))?;

    #[cfg(debug_assertions)]
    print_info("存入 Session 成功");

    // 返回成功的信号
    Ok(Json(json!({"success": true})))
}

// 负责从 Session 读取数据并返回给前端
pub async fn show_first_result(session: Session, State(tera): State<Tera>) -> Result<impl IntoResponse, WebError> {
    #[cfg(debug_assertions)]
    print_info("/result 被访问, 正在从 Session 中读取数据...");

    #[cfg(not(debug_assertions))]
    print_info("正在显示数据...");

    let courses_whole_list: Vec<Course> = session.get("courses").await.map_err(|e| WebError::InternalError(e.to_string()))?.unwrap_or_default();

    if !courses_whole_list.is_empty() {
        #[cfg(debug_assertions)]
        print_info("成功从 Session 中读取到数据, 正在调用业务逻辑模块进行计算...");

        let (gpa, courses_final_list) = calculate_gpa_from_list(&courses_whole_list, GPAMode::Default);

        let mut context = tera::Context::new();
        context.insert("courses", &courses_final_list);
        context.insert("gpa", &gpa);

        #[cfg(debug_assertions)]
        print_info("计算成功, 开始尝试渲染查询页面...");

        let html = tera.render("result.html", &context).map_err(|e| WebError::TemplateError(e.to_string()))?;

        #[cfg(not(debug_assertions))]
        print_info("数据显示成功");

        #[cfg(debug_assertions)]
        print_info("渲染成功");

        Ok(Html(html).into_response())
    } else {
        #[cfg(debug_assertions)]
        print_error("Session 中未找到数据, 将重定向到登录页");

        session.insert("flash_msg", "会话状态异常。").await.map_err(|e| WebError::InternalError(e.to_string()))?;

        Ok(Redirect::to("/").into_response())
    }
}

// 根据前端按钮重新计算 GPA
pub async fn show_next_result(session: Session, Json(cal_mode): Json<CalculateMode>) -> Result<Json<serde_json::Value>, WebError> {
    print_info("尝试切换计算模式...");

    let courses_whole_list: Vec<Course> = session.get("courses").await.map_err(|e| WebError::InternalError(e.to_string()))?.unwrap_or_default();

    if courses_whole_list.is_empty() {
        return Err(WebError::InternalError("数据异常".to_string()));
    }

    let mode = match cal_mode.mode.as_str() {
        "all" => GPAMode::All,
        _ => GPAMode::Default
    };

    let (gpa, course_final_list) = calculate_gpa_from_list(&courses_whole_list, mode);

    print_info("已切换计算模式");

    Ok(Json(json!({"gpa": gpa, "courses": course_final_list})))
}

// 关闭服务器
pub async fn shutdown_handler(Extension(shutdown_tx): Extension<broadcast::Sender<()>>) -> (StatusCode, &'static str) {
    let _ = shutdown_tx.send(());

    (StatusCode::OK, "服务器正在关闭...")
}

// 退出登录
pub async fn logout_handler(session: Session) -> Result<Json<serde_json::Value>, WebError> {
    session.delete().await.map_err(|e| WebError::InternalError(e.to_string()))?;

    print_info("用户退出登录, 会话已销毁");

    // 创建变量遮蔽来确保锁能被尽快释放
    {
        // 获取互斥锁
        let mut user_agent_guard = USER_AGENT.lock().unwrap();

        // 生成新 UA
        let new_user_agent = get_rua().to_string();

        // 使用星号(*)解引用修改在锁保护下的数据
        *user_agent_guard = new_user_agent.clone();

        #[cfg(debug_assertions)]
        print_info(&format!("UA 已被刷新: {}", new_user_agent.clone()));
    }
    // 超出遮蔽区域, 锁被释放

    Ok(Json(json!({"success": true})))
}