use warp::http::StatusCode;
use warp::{Filter, Reply};

use crate::ui;

pub fn routes() -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    let health = warp::path("health").and(warp::get()).map(|| "ok");
    let admin = warp::path!("auth" / "admin")
        .and(warp::get())
        .map(|| warp::reply::html(ui::render_admin_html()));
    let admin_assets = warp::path!("auth" / "admin" / "assets" / String)
        .and(warp::get())
        .map(|asset: String| match asset.as_str() {
            "admin.css" => warp::reply::with_header(
                ui::ADMIN_CSS,
                "content-type",
                "text/css; charset=utf-8",
            )
            .into_response(),
            "admin.js" => {
                warp::reply::with_header(ui::ADMIN_JS, "content-type", "application/javascript")
                    .into_response()
            }
            _ => warp::reply::with_status("not found", StatusCode::NOT_FOUND).into_response(),
        });
    let auth = warp::path!("auth" / "ui")
        .and(warp::get())
        .map(|| warp::reply::html(ui::AUTH_LOGIN_HTML));
    let auth_ui_config = warp::path!("auth" / "ui" / "config")
        .and(warp::get())
        .map(|| warp::reply::with_header(ui::AUTH_UI_CONFIG_JSON, "content-type", "application/json"));
    let auth_ui_tail = warp::path("auth")
        .and(warp::path("ui"))
        .and(warp::path::tail())
        .and(warp::get())
        .map(|tail: warp::path::Tail| {
            let path = tail.as_str().trim_matches('/');
            if let Some((content_type, content)) = ui::auth_ui_asset(path) {
                return warp::reply::with_header(content, "content-type", content_type).into_response();
            }
            if let Some(page) = ui::auth_ui_page(path) {
                return warp::reply::html(page).into_response();
            }
            warp::reply::with_status("not found", StatusCode::NOT_FOUND).into_response()
        });

    health
        .or(admin)
        .or(admin_assets)
        .or(auth_ui_config)
        .or(auth)
        .or(auth_ui_tail)
}
