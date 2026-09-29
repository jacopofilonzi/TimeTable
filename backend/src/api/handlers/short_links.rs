use std::collections::HashMap;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, header},
    response::{IntoResponse, Redirect, Response},
};
use serde::{Deserialize, Serialize};

use super::lessons_ics::ics_response;
use crate::{
    api::SharedState,
    errors::{AppError, Internal},
    state::AppState,
};

#[derive(Deserialize)]
pub struct CreateShortLink {
    uni: String,
    /// Schema fields, as for the lessons endpoints.
    params: HashMap<String, String>,
    weeks: Option<u8>,
}

#[derive(Serialize)]
pub struct CreatedShortLink {
    code: String,
}

/// `POST /api/short` `{uni, params, weeks}` → `{code}`; the link is `{BASE_PATH}/s/{code}`.
pub async fn create_short_link(
    State(state): State<SharedState>,
    Json(body): Json<CreateShortLink>,
) -> Result<Json<CreatedShortLink>, AppError> {
    let uni = state.registry.get(&body.uni)?;
    let mut query = body.params;
    if let Some(weeks) = body.weeks {
        query.insert("weeks".into(), weeks.to_string());
    }
    let code = state.create_short_link(uni, &query).await?;
    Ok(Json(CreatedShortLink { code }))
}

/// `GET /s/{code}` → redirect to the wizard result for the link's settings, in the visitor's
/// language. `GET /s/{code}.ics?name=…` → the iCalendar feed directly.
pub async fn open_short_link(
    state: &AppState,
    code: &str,
    query: &HashMap<String, String>,
    headers: &HeaderMap,
) -> Result<Response, AppError> {
    let suffix = code.len().checked_sub(4).filter(|&i| {
        code.get(i..)
            .is_some_and(|s| s.eq_ignore_ascii_case(".ics"))
    });
    let (code, ics) = match suffix {
        Some(i) => (&code[..i], true),
        None => (code, false),
    };
    let page = state.resolve_short_link(code).await?;

    if ics {
        let mut params: HashMap<String, String> = form_urlencoded::parse(page.as_bytes())
            .into_owned()
            .collect();
        let uni_id = params
            .remove("uni")
            .ok_or_else(|| Internal::new(format!("short link {code} has no university")))?;
        let uni = state.registry.get(&uni_id)?;
        if let Some(name) = query.get("name") {
            params.insert("name".into(), name.clone());
        }
        return ics_response(state, uni, &params).await;
    }

    let english = headers
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(prefers_english);
    let lang = if english { "en/" } else { "" };
    let location = format!("{}/{lang}?{page}&step=result", state.config.base_path);
    Ok((
        [
            (header::VARY, "Accept-Language"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        Redirect::temporary(&location),
    )
        .into_response())
}

/// Whether `Accept-Language` ranks English above Italian (the site default).
fn prefers_english(accept_language: &str) -> bool {
    let mut best: Option<(f32, bool)> = None;
    for part in accept_language.split(',') {
        let mut pieces = part.split(';');
        let tag = pieces
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        let english = match tag.split('-').next() {
            Some("en") => true,
            Some("it") => false,
            _ => continue,
        };
        let q = pieces
            .find_map(|p| p.trim().strip_prefix("q="))
            .and_then(|q| q.parse::<f32>().ok())
            .unwrap_or(1.0);
        if best.is_none_or(|(best_q, _)| q > best_q) {
            best = Some((q, english));
        }
    }
    best.is_some_and(|(_, english)| english)
}

#[cfg(test)]
mod tests {
    use super::prefers_english;

    #[test]
    fn accept_language() {
        assert!(prefers_english("en-US,en;q=0.9"));
        assert!(prefers_english("de-DE,en;q=0.5"));
        assert!(prefers_english("it;q=0.3,en-GB;q=0.8"));
        assert!(!prefers_english("it-IT,it;q=0.9,en;q=0.8"));
        assert!(!prefers_english("de-DE,fr"));
        assert!(!prefers_english(""));
    }
}
