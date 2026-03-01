//! 关键路由集成测试：无 session 访问 /result 重定向、错误码等

use axum::{
    body::Body,
    extract::Request,
    http::Request as HttpRequest,
    middleware::{self, Next},
};
use tower::util::ServiceExt;
use tower_cookies::CookieManagerLayer;
use tower_sessions::{MemoryStore, SessionManagerLayer};
use yit_gpa_tool::create_router;

fn test_app() -> axum::Router {
    let tera = tera::Tera::default();
    let store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(store);
    let key = tower_cookies::Key::from(&[0u8; 64]);

    create_router(tera)
        .layer(middleware::from_fn(move |mut req: Request, next: Next| {
            req.extensions_mut().insert(key.clone());
            async move { next.run(req).await }
        }))
        .layer(session_layer)
        .layer(CookieManagerLayer::new())
}

#[tokio::test]
async fn get_result_without_session_redirects_to_root() {
    let app = test_app();

    let response = app
        .oneshot(
            HttpRequest::get("/result")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status().is_redirection(),
        "无 session 访问 /result 应返回重定向 (302/303)，实际: {}",
        response.status()
    );
    let location = response
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok());
    assert!(
        location == Some("/") || location.map(|s| s.ends_with('/')).unwrap_or(false),
        "Location 应为 / 或以 / 结尾的 URL，实际: {:?}",
        location
    );
}
