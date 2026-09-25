use axum::{
    Router,
    extract::Request,
    http::{StatusCode, Uri},
    response::{IntoResponse, Redirect},
};
use tower::ServiceExt;

/// Mounts `app` under `base` (e.g. `/timetable`), stripping the prefix before routing.
/// `/` and `/timetable` redirect to `/timetable/`; anything else outside the prefix is a 404.
///
/// Done by hand because axum 0.8's `nest()` does not match `/timetable/` (trailing slash).
pub fn mount(base: String, app: Router) -> Router {
    Router::new().fallback(move |mut req: Request| {
        let app = app.clone();
        let base = base.clone();
        async move {
            let path = req.uri().path();
            if path == "/" || path == base {
                return Redirect::permanent(&format!("{base}/")).into_response();
            }
            let Some(rest) = path.strip_prefix(&base).filter(|r| r.starts_with('/')) else {
                return StatusCode::NOT_FOUND.into_response();
            };
            let rewritten = match req.uri().query() {
                Some(q) => format!("{rest}?{q}"),
                None => rest.to_string(),
            };
            match rewritten.parse::<Uri>() {
                Ok(uri) => *req.uri_mut() = uri,
                Err(_) => return StatusCode::BAD_REQUEST.into_response(),
            }
            match app.oneshot(req).await {
                Ok(res) => res,
                Err(never) => match never {},
            }
        }
    })
}
