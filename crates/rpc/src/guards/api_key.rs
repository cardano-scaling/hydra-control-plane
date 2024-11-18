use std::env;

use rocket::{
    http::Status,
    request::{self, FromRequest, Request},
};
use tracing::error;

pub struct ApiKey;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let api_key = match env::var("API_KEY") {
            Ok(key) => key,
            Err(err) => {
                error!(error = err.to_string(), "Could not find API_KEY env var");
                return request::Outcome::Error((Status::InternalServerError, ()));
            }
        };

        match request.headers().get_one("Authorization") {
            Some(auth_key) if auth_key == api_key => request::Outcome::Success(ApiKey),
            _ => request::Outcome::Error((Status::Unauthorized, ())),
        }
    }
}
