use anyhow::{Context, Result};
use std::net::SocketAddr;
use tera::Tera;
use tokio::net::TcpListener;
use webbrowser;
use axum::serve;

mod models;
mod utils;
mod web_scraping;
mod web;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化模板引擎
    let tera = Tera::new("templates/**/*.html")?;

    // 创建路由
    let app = web::create_router(tera);

    // 绑定地址到 TCP 监听器
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await.with_context(|| format!("无法绑定到地址 {}", addr))?;
    println!("服务器运行于 http://{}", addr);

    // 自动打开浏览器
    let _ = webbrowser::open(&format!("http://{}", addr));

    // 监听器启动服务
    serve(listener, app.into_make_service()).await.with_context(|| "服务器启动失败")?;

    Ok(())
}
