use anyhow::{Context, Result};
use axum::serve;
use rust_embed::RustEmbed;
use std::net::SocketAddr;
use tera::Tera;
use tokio::net::TcpListener;
use utils::current_time;
use webbrowser;

mod models;
mod utils;
mod web_scraping;
mod web;

// 使用 RustEmbed 宏来嵌入整个 templates 文件夹
// folder 路径是相对于 Cargo.toml 文件的
#[derive(RustEmbed)]
#[folder = "templates/"]
struct Asset;   // 虚拟结构体, 用于持有嵌入的模板文件

#[tokio::main]
async fn main() -> Result<()> {
    println!("[{}]初始化服务器中...", current_time());

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
            // 这里的 content 已经是借用的形式了(&str), 因此可以不需要借用符号(&)
            tera.add_raw_template(&file_path, content).with_context(|| format!("[{}]导入嵌入文件失败: {}", current_time(), file_path))?;
        }
    }

    // 构建 Tera 的继承链
    tera.build_inheritance_chains().context(format!("[{}]构建Tera继承链失败.", current_time()))?;

    // 创建路由
    let app = web::create_router(tera);

    // 绑定地址到 TCP 监听器
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await.with_context(|| format!("[{}]无法绑定到地址 {}", current_time(), addr))?;
    println!("[{}]服务器将运行于 http://{} ，如不小心关闭浏览器，重新打开浏览器输入该网址即可", current_time(), addr);

    // 自动打开浏览器
    let _ = webbrowser::open(&format!("http://{}", addr));

    // 监听器启动服务
    serve(listener, app.into_make_service()).await.with_context(|| format!("[{}]服务器启动失败", current_time()))?;

    println!("[{}]服务器启动成功！注意：请勿关闭此窗口，否则程序将终止运行", current_time());

    Ok(())
}
