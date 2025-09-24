// 纯路由层
use crate::handler::{download_temp, first_result, login, logout, next_result, score_from_file, score_from_official, shutdown, static_file};

use axum::{routing::{get, post}, Router};
use tera::Tera;

pub fn create_router(tera: Tera) -> Router {
    Router::new()
        .route("/", get(login))    // 根目录是登录页面
        .route("/score-from-official-website", post(score_from_official))    // 这是回传登录数据的 API 接口
        .route("/score-from-file", post(score_from_file))  // 免登录 API 接口
        .route("/download-template", get(download_temp)) // 获取文件
        .route("/result", get(first_result)) // 显示计算后学分
        .route("/recalc", post(next_result))   // 重新计算 GPA 的 API 接口
        .route("/logout", post(logout))     // 退出登录
        .route("/shutdown", post(shutdown)) // 关闭服务器
        .fallback(static_file)   // 自动加载并注册 static 的资源
        .with_state(tera)   // 将 Tera 模板引擎作为共享状态以便所有路由处理器都能访问
}