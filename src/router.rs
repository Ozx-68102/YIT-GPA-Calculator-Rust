// 纯路由层
use crate::handler::{
    logout_handler, score_handler, show_first_result, show_login,
    show_next_result, shutdown_handler, static_handler,
};

use axum::{routing::{get, post}, Router};
use tera::Tera;

pub fn create_router(tera: Tera) -> Router {
    Router::new()
        .route("/", get(show_login))    // 根目录是登录页面
        .route("/score", post(score_handler))    // 这是回传登录数据的 API 接口
        .route("/result", get(show_first_result)) // 显示计算后学分
        .route("/recalc", post(show_next_result))   // 重新计算 GPA 的 API 接口
        .route("/logout", post(logout_handler))     // 退出登录
        .route("/shutdown", post(shutdown_handler)) // 关闭服务器
        .fallback(static_handler)   // 自动加载并注册 static 的资源
        .with_state(tera)   // 将 Tera 模板引擎作为共享状态以便所有路由处理器都能访问
}