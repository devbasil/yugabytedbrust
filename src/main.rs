#![allow(unused)]
#[macro_use]

mod generic_json_error_response;
mod generic_json_success_response;
mod crud_controller;
mod default_strings;
mod yugabyte_db_ycql_config;
mod ycql_db_schema_and_queries;

use default_strings::{UserErrorMessages, CustomStatusMessage};
use yugabyte_db_ycql_config::{configure_yugabyte_db_session, create_demo_ycql_service_keyspace, create_demo_ycql_user_profile_table};
use crud_controller::{create_user_profile, read_user_profile, delete_user_profile, update_user_profile};
use actix_web::middleware::{ErrorHandlers, ErrorHandlerResponse};
use actix_web::{middleware, dev, get, error, http, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web::rt::System;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};
use serde_json::{json, to_string_pretty};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

#[derive(Debug, Serialize)]
pub struct CustomJsonErrorResponse {
    custom_status: String,
    message: String
}

impl CustomJsonErrorResponse {
    fn json_error_response(custom_status: String, message: String) -> CustomJsonErrorResponse {
        CustomJsonErrorResponse {
            custom_status,
            message
        }
    }
}


fn json_error_handler(err: error::JsonPayloadError, _req: &web::HttpRequest) -> error::Error {
    let detail = err.to_string();
    let response = match &err {
        error::JsonPayloadError::ContentType => {
            web::HttpResponse::UnsupportedMediaType()
            .content_type("application/json")
            .body(
                serde_json::to_string(&CustomJsonErrorResponse::json_error_response(CustomStatusMessage::USER_REQUEST_FAILED.to_string(), UserErrorMessages::UnsupportedMediaType.to_string())).unwrap()
            )

        }
        _ => {  
            web::HttpResponse::BadRequest()
            .content_type("application/json")
            .body(serde_json::to_string(&CustomJsonErrorResponse::json_error_response(CustomStatusMessage::USER_REQUEST_FAILED.to_string(), UserErrorMessages::BadClientData.to_string())).unwrap())

        },
    };
    error::InternalError::from_response(err, response).into()
}


#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let mut yugabyte_db_connection = configure_yugabyte_db_session().await.expect("Fatal YCQL session DB connection Error");
 

    create_demo_ycql_service_keyspace(&mut yugabyte_db_connection).await;
    create_demo_ycql_user_profile_table(&mut yugabyte_db_connection).await;
  

    #[allow(clippy::mutex_atomic)]
    let ycql_connection_data = web::Data::new(Mutex::new(yugabyte_db_connection));


    HttpServer::new(move || {
        App::new()
        .app_data(ycql_connection_data.clone()) // add shared state
        .app_data(web::JsonConfig::default()
        // limit request payload size
        .limit(4096)
        // use custom error handler
        .error_handler(json_error_handler)
        )
            // enable logger
            .wrap(middleware::Logger::default())
        .service(
            web::scope("/api_v1")
            .service(create_user_profile)
            .service(read_user_profile)
            .service(delete_user_profile)
            .service(update_user_profile)

        )    
    })
    .bind("127.0.0.1:4055")?
    .run()
    .await
}
