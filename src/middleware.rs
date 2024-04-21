use axum::{body::to_bytes, extract::Request, middleware::Next, response::IntoResponse, Json};
use email_address::EmailAddress;
use hyper::StatusCode;

use crate::authentication::Authentication;

pub async fn verify_email_middleware(
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    let (parts, body) = request.into_parts();
    let bytes = to_bytes(body, usize::MAX).await;
    match bytes {
        Ok(bytes) => {
            let json = Json::<Authentication>::from_bytes(&bytes);
            match json {
                Ok(Json(mut payload)) => {
                    payload.email = payload.email.trim().to_lowercase();
                    let valid = EmailAddress::is_valid(&payload.email);
                    if valid {
                        let body = Json(payload).into_response().into_body();
                        let req = Request::from_parts(parts, body);
                        let response = next.run(req).await;
                        Ok(response)
                    } else {
                        Err(StatusCode::BAD_REQUEST)
                    }
                }
                Err(_) => Err(StatusCode::BAD_REQUEST),
            }
        }
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}
