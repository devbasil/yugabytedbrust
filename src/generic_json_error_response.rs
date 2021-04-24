use std::fmt::{Display, Formatter, Result as FmtResult};
use actix_web::{ResponseError, web, http::StatusCode};
use serde_json::{json, to_string_pretty};
use std::io;
use serde::{Deserialize, Serialize};

/// Generic Json ERROR RESPONSE
#[derive(Debug, Serialize)]
pub struct GenericJsonErrorResponse {
  pub  custom_status: String,
  pub  message: String,
  pub  status: u16,
}


impl Display for GenericJsonErrorResponse {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", to_string_pretty(self).unwrap())
    }
}

impl ResponseError for GenericJsonErrorResponse {
    // builds the actual response to send back when an error occurs
    fn error_response(&self) -> web::HttpResponse {
        let err_json = json!({"custom_status": self.custom_status, "message": self.message});
        web::HttpResponse::build(StatusCode::from_u16(self.status).unwrap())
            .json(err_json)
    }
}