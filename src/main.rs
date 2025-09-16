use crate::business::{format_log_msg, print_info};

use anyhow::{Context, Result};
use axum::{
    extract::Request,
    middleware::{self, Next},
    serve,
    Extension
};
use rand::Rng;
use rust_embed::RustEmbed;
use std::net::SocketAddr;
use tera::Tera;
use tokio::{net::TcpListener, sync::broadcast};
use tower_cookies::{CookieManagerLayer, Key};
use tower_sessions::{MemoryStore, SessionManagerLayer};
use webbrowser;

mod models;
mod business;
mod scraping;
mod handler;
mod router;

// 使用 RustEmbed 宏来嵌入整个 templates 文件夹
// folder 路径是相对于 Cargo.toml 文件的
#[derive(RustEmbed)]
#[folder = "templates/"]
pub struct Asset;   // 虚拟结构体, 用于持有嵌入的模板文件

#[tokio::main]
async fn main() -> Result<()> {
    print_info("初始化服务器中...");

    let (shutdown_tx, mut shutdown_rx) = broadcast::channel::<()>(1);

    // 初始化模板引擎
    let mut tera = Tera::default();

    // 遍历所有嵌入的文件
    for file_path in Asset::iter() {
        // 获取文件内容
        if let Some(embedded_file) = Asset::get(&file_path) {
            // embedded_file.data 是文件内容, 类型为 Vec<u8>
            // embedded_file.metadata 是文件元数据, 比如说是否为目录
            // 将 Vec<u8> 转换为 &str
            let content = std::str::from_utf8(embedded_file.data.as_ref())?;

            // 将 HTML 模板添加到 Tera 实例
            // 这里的 content 已经是借用的形式了(类型 &str), 因此可以不需要借用符号(&)
            tera.add_raw_template(&file_path, content).with_context(|| format_log_msg(&format!("导入嵌入文件失败: {}", file_path)))?;
        }
    }

    // 构建 Tera 的继承链
    tera.build_inheritance_chains().with_context(|| format_log_msg("构建Tera继承链失败"))?;

    // 创建 Session 存储
    let store = MemoryStore::default();

    // 创建 Session 层
    let session_layer = SessionManagerLayer::new(store);

    // 创建用于签名的 Cookie 密钥
    let key = Key::from(&rand::rng().random::<[u8; 64]>());

    // 创建路由
    let app = router::create_router(tera)
        .layer(Extension(shutdown_tx))  // 增加关闭服务器的扩展
        .layer(middleware::from_fn(move |mut req: Request, next: Next| {
            req.extensions_mut().insert(key.clone());
            async move { next.run(req).await }
        })).layer(session_layer)
        .layer(CookieManagerLayer::new());

    // 绑定地址到 TCP 监听器
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await.with_context(|| format_log_msg(&format!("无法绑定到地址 {}", addr)))?;
    print_info(&format!("服务器将运行于 http://{} ，如不小心关闭浏览器，重新打开浏览器输入该网址即可", addr));

    // 自动打开浏览器
    let _ = webbrowser::open(&format!("http://{}", addr));

    print_info("服务器启动成功！注意：请勿关闭此窗口，否则程序将终止运行");

    // 监听器启动服务
    let server = serve(listener, app.into_make_service()).with_graceful_shutdown(async move {
        shutdown_rx.recv().await.ok();
        print_info("服务器正在关闭...");
    });

    server.await.with_context(|| format_log_msg("服务器运行时发生致命错误"))?;

    #[cfg(debug_assertions)]
    print_info("服务器已成功关闭");

    Ok(())
}
