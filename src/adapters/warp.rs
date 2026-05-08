use warp::{Filter, Reply};

use crate::ui;

pub fn routes() -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    let health = warp::path("health").and(warp::get()).map(|| "ok");
    let admin = warp::path!("auth" / "admin")
        .and(warp::get())
        .map(|| warp::reply::html(ui::ADMIN_UI_HTML));
    let auth = warp::path!("auth" / "ui")
        .and(warp::get())
        .map(|| warp::reply::html(ui::AUTH_UI_HTML));
    let auth_js = warp::path!("auth" / "ui" / "auth.js")
        .and(warp::get())
        .map(|| warp::reply::with_header(ui::AUTH_JS, "content-type", "application/javascript"));
    let auth_ui_config = warp::path!("auth" / "ui" / "config")
        .and(warp::get())
        .map(|| warp::reply::with_header(ui::AUTH_UI_CONFIG_JSON, "content-type", "application/json"));

    health.or(admin).or(auth).or(auth_js).or(auth_ui_config)
}
