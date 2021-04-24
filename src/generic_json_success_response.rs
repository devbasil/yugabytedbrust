use std::fmt::{Display, Formatter, Result as FmtResult};
use serde_json::{json, to_string_pretty};
use std::io;
use serde::{Deserialize, Serialize};



/// USER--Generic Json SUCCESS RESPONSE
#[derive(Debug, Serialize)]
pub struct GenericJsonSuccessResponse<T> {
  pub  custom_status: String,
  pub  message: String,
  pub  status: u16,
  pub  data: Option<T>
}


impl<T> GenericJsonSuccessResponse<T> {
   pub fn generic_success_response(custom_status: String, message: String, status: u16, data: Option<T>) -> GenericJsonSuccessResponse<T> {
       GenericJsonSuccessResponse {
            custom_status,
            message,
            status,
            data
        }
    }
}