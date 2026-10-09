//! `POST /v1/scans`: a fridge photo in, what the model saw out.
//!
//! The photo goes to the model and is then dropped, unless the cook agreed
//! to the dataset: then a copy without metadata is kept with what was seen.

use super::CurrentUser;
use crate::ai::FridgeQuestion;
use crate::db::consents;
use crate::domain::consent::Purpose;
use crate::domain::fridge::{self, SeenItem};
use crate::domain::photo::{Photo, PhotoError, MAX_PHOTO_BYTES};
use crate::error::AppError;
use crate::AppState;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, FromRequest, Request, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde::Serialize;
use uuid::Uuid;

mod keep;

pub fn router() -> Router<AppState> {
    let route = post(scan).layer(DefaultBodyLimit::max(MAX_PHOTO_BYTES));
    Router::new().route("/v1/scans", route)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ScanResponse {
    /// Send it back with the confirmed ingredients when asking for a special.
    scan_id: Uuid,
    /// Whether the photo was kept for the dataset.
    kept: bool,
    items: Vec<SeenItem>,
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
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    PhotoUpload(photo): PhotoUpload,
) -> Result<Json<ScanResponse>, AppError> {
    if !consents::is_current(&state.db, user_id, Purpose::PhotoScan).await? {
        return Err(AppError::ConsentRequired(Purpose::PhotoScan));
    }
    let items = read_fridge(&state, &photo).await?;
    let scan_id = Uuid::new_v4();
    let scan = keep::Scan { user_id, scan_id, photo: &photo, items: &items };
    let kept = keep::if_agreed(&state, &scan).await?;
    Ok(Json(ScanResponse { scan_id, kept, items }))
}

async fn read_fridge(state: &AppState, photo: &Photo) -> Result<Vec<SeenItem>, AppError> {
    let ingredients = state.library.table.names();
    let question = FridgeQuestion { photo, ingredients };
    let answer = state.ai.read_fridge(question).await;
    let items = answer.map_err(|_| AppError::AiUnavailable)?;
    Ok(fridge::checked(items, &state.library.table))
}
