// 库入口，供 main 与集成测试共用
use rust_embed::RustEmbed;

pub mod _url;
pub mod business;
pub mod handler;
pub mod models;
pub mod router;
pub mod scraping;

pub use router::create_router;

#[derive(RustEmbed)]
#[folder = "templates/"]
pub struct TemplateAsset;

#[derive(RustEmbed)]
#[folder = "assets/"]
pub struct BinaryAsset;
