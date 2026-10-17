//! `POST /v1/scans`: a fridge photo in, what the model saw out.
//!
//! The photo goes to the model and is kept as it arrived, with what was seen,
//! when photo storage is set up. Proof of concept: there is no consent step yet.

use super::keep;
use crate::ai::FridgeQuestion;
use crate::error::AppError;
use crate::http::{Caller, CookLanguage};
use crate::kernel::display::Namer;
use crate::scans::fridge::{self, SeenItem, ShownItem};
use crate::scans::photo::{Photo, PhotoError, MAX_PHOTO_BYTES};
use crate::AppState;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, FromRequest, Request};
use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde::Serialize;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    let route = post(scan).layer(DefaultBodyLimit::max(MAX_PHOTO_BYTES));
    Router::new().route("/v1/scans", route)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ScanResponse {
    /// Send it back with the confirmed ingredients when asking for a special.
    scan_id: Uuid,
    /// Whether the photo was kept: it is whenever photo storage is set up.
    kept: bool,
    items: Vec<ShownItem>,
}

/// The request body read as a photo, typed by its `Content-Type`.
pub struct PhotoUpload(Photo);

impl FromRequest<AppState> for PhotoUpload {
    type Rejection = AppError;

    async fn from_request(request: Request, state: &AppState) -> Result<Self, AppError> {
        let content_type = request.headers().get(CONTENT_TYPE).cloned();
        let content_type = content_type.as_ref().and_then(|value| value.to_str().ok());
        let bytes = match Bytes::from_request(request, state).await {
            Ok(bytes) => bytes,
            Err(rejection) if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE => {
                return Err(PhotoError::TooLarge.into());
            }
            Err(_) => return Err(PhotoError::Unreadable.into()),
        };
        Ok(PhotoUpload(Photo::parse(content_type, bytes.to_vec())?))
    }
}

async fn scan(
    caller: Caller,
    CookLanguage(language): CookLanguage,
    PhotoUpload(photo): PhotoUpload,
) -> Result<Json<ScanResponse>, AppError> {
    let Caller { state, user_id } = caller;
    let items = read_fridge(&state, &photo).await?;
    let scan_id = Uuid::new_v4();
    let scan = keep::Scan { user_id, scan_id, photo: &photo, items: &items };
    let kept = keep::photo(&state, &scan).await;
    let namer = Namer { library: &state.library, foods: state.foods(), language: &language };
    let items = items.iter().map(|item| item.shown(&namer)).collect();
    Ok(Json(ScanResponse { scan_id, kept, items }))
}

async fn read_fridge(state: &AppState, photo: &Photo) -> Result<Vec<SeenItem>, AppError> {
    let graph = state.graph()?;
    let ingredients = graph.foods.ids();
    let question = FridgeQuestion { photo, ingredients: &ingredients };
    let answer = state.ai.read_fridge(question).await;
    let items = answer.map_err(|_| AppError::AiUnavailable)?;
    Ok(fridge::checked(items, &graph.foods))
}
