// 路由控制器
use crate::{
    business::{
        print_error, print_info, process_scraped_course_results, round_2decimal, score_trans_grade,
        ProcessedGPAResults, ResultSource, EXCLUDED_COURSES_KEYWORD,
        NATURE_EXCLUSIONS, PERMANENT_IGNORED_COURSES,
    },
    models::{Course, FileError, WebError},
    scraping::{AAOWebsite, USER_AGENT},
    BinaryAsset, TemplateAsset
};

use axum::{
    extract::{Form, Multipart, State},
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse, Redirect, Response},
    Extension,
    Json
};
use calamine::{Reader, Xlsx};
use fake_user_agent::get_rua;
use mime_guess;
use rust_decimal::Decimal;
use std::io::Cursor;

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
pub async fn static_file(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches("/");

    if path.is_empty() {
        return (StatusCode::NOT_FOUND, "Not Found").into_response();
    }

    match TemplateAsset::get(path) {
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
pub async fn login(session: Session, State(tera): State<Tera>) -> Result<Html<String>, WebError> {
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

// 负责从登录网站中获取数据
pub async fn score_from_official(session: Session, Form(form): Form<LoginForm>) -> Result<Json<serde_json::Value>, WebError> {
    #[cfg(debug_assertions)]
    print_info("准备爬取数据");

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
    print_info(&format!("数据爬取成功, 共{}门课程", courses.len()));

    let results: ProcessedGPAResults = process_scraped_course_results(&courses, ResultSource::OfficialWebsite);
    let default_result = results.default.unwrap();   // 因为 ResultSource::OfficialWebsite, 所以在这里总会返回 Some
    let all_result = results.all;


    // Default 模式数据
    session.insert("gpa_default", default_result.gpa).await.map_err(|e| WebError::InternalError(e.to_string()))?;
    session.insert("courses_default", default_result.courses).await.map_err(|e| WebError::InternalError(e.to_string()))?;

    // All 模式数据
    session.insert("gpa_all", all_result.gpa).await.map_err(|e| WebError::InternalError(e.to_string()))?;
    session.insert("courses_all", all_result.courses).await.map_err(|e| WebError::InternalError(e.to_string()))?;

    // 数据模式
    session.insert("result_mode", "login").await.map_err(|e| WebError::InternalError(e.to_string()))?;

    #[cfg(debug_assertions)]
    print_info("存入 Session 成功");

    // 返回成功的信号
    Ok(Json(json!({"success": true})))
}

// 负责从文件中获取数据
pub async fn score_from_file(session: Session, mut multipart: Multipart) -> Result<Json<serde_json::Value>, WebError> {
    let mut courses: Vec<Course> = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("gpa_file") {   // 和前端 formData 的键名一致
            let data = field.bytes().await.map_err(|e| FileError::OpenError(e.to_string()))?;
            let reader = Cursor::new(data);
            let mut worksheet: Xlsx<_> = Xlsx::new(reader).map_err(|e| FileError::OpenError(e.to_string()))?;

            if let Ok(range) = worksheet.worksheet_range("Sheet1") {
                for row in range.rows().skip(3) {
                    let name = row.get(0).map(|c| c.to_string()).unwrap_or_default().trim().to_string();
                    let credit_str = row.get(1).map(|c| c.to_string()).unwrap_or_default().trim().to_string();
                    let score_str = row.get(2).map(|c| c.to_string()).unwrap_or_default().trim().to_string();

                    if name.is_empty() || credit_str.is_empty() || score_str.is_empty() { continue; }
                    if let Ok(credit) = credit_str.parse::<Decimal>() {
                        if let Some(grade) = score_trans_grade(&score_str) {
                            let credit_gpa = round_2decimal(grade * credit);
                            courses.push(Course {
                                name,
                                nature: "".to_string(),
                                score: score_str,
                                credit,
                                grade,
                                credit_gpa,
                            });
                        }
                    }
                }
            }
        }
    }

    if courses.is_empty() {
        return Err(FileError::NoValidDataFound.into());
    }

    print_info(&format!("从 Excel 文件中成功解析{}门课程", courses.len()));

    // 只关心 All 模式的数据
    let (gpa, courses_for_use) = {
        let results: ProcessedGPAResults = process_scraped_course_results(&courses, ResultSource::InputFile);

        (results.all.gpa, results.all.courses)
    };

    session.insert("courses_all", courses_for_use).await.map_err(|e| WebError::InternalError(e.to_string()))?;
    session.insert("gpa_all", gpa).await.map_err(|e| WebError::InternalError(e.to_string()))?;

    // 数据模式
    session.insert("result_mode", "file").await.map_err(|e| WebError::InternalError(e.to_string()))?;

    #[cfg(debug_assertions)]
    print_info("计算结果已存入 Session");

    Ok(Json(json!({"success": true})))
}

// 负责从 Session 读取 Default 模式数据并返回给前端
pub async fn first_result(session: Session, State(tera): State<Tera>) -> Result<impl IntoResponse, WebError> {
    #[cfg(debug_assertions)]
    print_info("正在从 Session 中读取数据...");

    #[cfg(not(debug_assertions))]
    print_info("正在显示数据...");

    let result_mode: String = session.get("result_mode").await?.unwrap_or("file".to_string());

    // 适配免登录模式
    let (gpa, courses): (Decimal, Vec<Course>) = match result_mode.as_str() {
        "login" => {
            (
                session.get("gpa_default").await?.unwrap_or_default(),
                session.get("courses_default").await?.unwrap_or_default()
            )
        }
        _ => {
            (
                session.get("gpa_all").await?.unwrap_or_default(),
                session.get("courses_all").await?.unwrap_or_default()
            )
        }
    };

    if courses.is_empty() {
        #[cfg(debug_assertions)]
        print_error("Session 中未找到数据, 将重定向到登录页");

        session.insert("flash_msg", "请先登录或使用免登录模式获取绩点数据。").await.map_err(|e| WebError::InternalError(e.to_string()))?;

        return Ok(Redirect::to("/").into_response());
    }

    #[cfg(debug_assertions)]
    print_info("成功从 Session 中读取到数据, 开始尝试渲染查询页面...");

    let mut context = tera::Context::new();
    context.insert("courses", &courses);
    context.insert("gpa", &gpa);
    context.insert("result_mode", &result_mode);

    // 将排除的变量也传给前端
    context.insert("excluded_courses", EXCLUDED_COURSES_KEYWORD);
    context.insert("permanent_ignored_courses", PERMANENT_IGNORED_COURSES);
    context.insert("nature_exclusions", NATURE_EXCLUSIONS);

    let html = tera.render("result.html", &context).map_err(|e| WebError::TemplateError(e.to_string()))?;

    #[cfg(not(debug_assertions))]
    print_info("数据显示成功");

    #[cfg(debug_assertions)]
    print_info("渲染成功");

    Ok(Html(html).into_response())
}

// 根据前端按钮重新计算 GPA
pub async fn next_result(session: Session, Json(cal_mode): Json<CalculateMode>) -> Result<Json<serde_json::Value>, WebError> {
    print_info("尝试切换计算模式...");

    let (gpa, courses): (Decimal, Vec<Course>) = match cal_mode.mode.as_str() {
        "all" => (
            session.get("gpa_all").await?.unwrap_or_default(),
            session.get("courses_all").await?.unwrap_or_default()
        ),
        _ => (
            session.get("gpa_default").await?.unwrap_or_default(),
            session.get("courses_default").await?.unwrap_or_default()
        )
    };

    print_info("已切换计算模式");

    Ok(Json(json!({"gpa": gpa, "courses": courses})))
}

// 关闭服务器
pub async fn shutdown(Extension(shutdown_tx): Extension<broadcast::Sender<()>>) -> (StatusCode, &'static str) {
    let _ = shutdown_tx.send(());

    (StatusCode::OK, "服务器正在关闭...")
}

// 退出登录
pub async fn logout(session: Session) -> Result<Json<serde_json::Value>, WebError> {
    session.delete().await.map_err(|e| WebError::InternalError(e.to_string()))?;

    print_info("用户退出登录, Session 会话已销毁");

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

// 下载 xlsx 文件
pub async fn download_temp() -> Result<impl IntoResponse, WebError> {
    print_info("正在下载上传模板文件...");

    match BinaryAsset::get("CoursesList.xlsx") {
        Some(content) => {
            let body = content.data;
            let headers = [
                (header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
                (header::CONTENT_DISPOSITION, "attachment; filename=CoursesList.xlsx")
            ];
            Ok((headers, body).into_response())
        }
        None => Err(WebError::InternalError("未找到模板文件".to_string()))
    }
}