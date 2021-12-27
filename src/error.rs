use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use std::io::Cursor;

#[derive(serde::Serialize)]
pub struct ApiError {
    error_message: String,
}

impl ApiError {
    pub fn new(error_message: &String) -> ApiError {
        ApiError {
            error_message: error_message.clone(),
        }
    }
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let body: String = serde_json::to_string(&self).unwrap();
        Response::build()
            .status(Status::BadRequest)
            .header(ContentType::JSON)
            .sized_body(body.len(), Cursor::new(body))
            .ok()
    }
}

impl From<web3::Error> for ApiError {
    fn from(err: web3::Error) -> Self {
        Self {
            error_message: format!("{:?}", err),
        }
    }
}

impl From<web3::contract::Error> for ApiError {
    fn from(err: web3::contract::Error) -> Self {
        Self {
            error_message: format!("Smart contract error: {:?}", err),
        }
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        Self {
            error_message: format!("HTTP request error: {:?}", err),
        }
    }
}

impl From<std::num::ParseFloatError> for ApiError {
    fn from(err: std::num::ParseFloatError) -> Self {
        Self {
            error_message: format!("Error parsing string: {:?}", err),
        }
    }
}
