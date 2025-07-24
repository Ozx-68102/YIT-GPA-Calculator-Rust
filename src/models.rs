// 结构体与自定义异常
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response}
};

// 课程信息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    pub name: String,       // 课程名称
    pub score: String,      // 总分
    pub credit: Decimal,    // 学分
    pub grade: Decimal,     // 绩点
    pub credit_gpa: Decimal // 加权绩点, 学分 × 绩点
}

// 网页爬取异常
#[derive(Debug, Error)]
pub enum WebScrapingError {
    #[error("HTTP 请求失败: {0}")]
    HttpRequest(String),

    #[error("Cookie无效或不存在。")]
    CookieInvalid,

    #[error("登录失败")]
    LoginFailed,

    #[error("解析异常: {0}")]
    ParseError(String)
}

// 网页服务异常
#[derive(Debug, Error)]
pub enum WebError {
    #[error("模板渲染失败: {0}")]
    TemplateError(String),

    #[error("网页爬取错误: {0}")]
    WebScrapingError(#[from] WebScrapingError),

    #[error("内部错误: {0}")]
    InternalError(String)
}

// 根据 Axum 库的要求, 需要实现 IntoResponse
impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            WebError::TemplateError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("模板错误: {}", msg)
            ),
            WebError::WebScrapingError(scraper_err) => match scraper_err {
                WebScrapingError::LoginFailed => (
                    StatusCode::UNAUTHORIZED,
                    scraper_err.to_string()
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    scraper_err.to_string()
                )
            },
            WebError::InternalError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("内部错误: {}", msg)
            )
        };

        (status, message).into_response()
    }
}