use crate::default_strings::{CustomStatusMessage, UserErrorMessages};
use crate::generic_json_error_response::GenericJsonErrorResponse;
use crate::generic_json_success_response::GenericJsonSuccessResponse;
use crate::yugabyte_db_ycql_config::CurrentYcqlDbSession;
use crate::ycql_db_schema_and_queries::{UserProfileRowStruct, create_new_user_profile_query, select_user_profile_query, update_full_name_and_age_query, delete_user_profile_query};
use actix_web::{web, get, post, App, HttpServer, http::StatusCode, ResponseError, Error, HttpResponse, Responder};


use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};
use serde_json::{json, to_string_pretty};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use uuid::Uuid;
use uuid::v1::{Timestamp, Context};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
///Json Request Struct for |create_user_profile| FUNCTION 
#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct UserProfileJsonREQUEST {
    email_address: String,
    full_name: String,
    age: i8,
    comment: Option<String>
}
#[post("/create_user")]
pub async fn create_user_profile(
    ycql_connection_data: web::Data<Mutex<CurrentYcqlDbSession>>,
    get_user_profile: web::Json<UserProfileJsonREQUEST>
)-> Result<web::HttpResponse, GenericJsonErrorResponse> {

    let mut current_ycql_conn   = ycql_connection_data.lock().unwrap(); // get DB session

    // Create version 1 UUID(TimeUUID)
    let current_time = get_epoch_ms();
    let context = Context::new(42);
    let ts = Timestamp::from_rfc4122(current_time, 0);
    let uuid_version_1 = Uuid::new_v1(ts, &[1, 2, 3, 4, 5, 6]).expect("failed to generate UUID Version_1");
     //Create version 2 UUID
    let uuid_version_4 = Uuid::new_v4();

    let row = UserProfileRowStruct::build_user_profile(
        uuid_version_4, // user_id,
        uuid_version_1, // time_uuid_order,
        get_user_profile.email_address.to_string(), //email_address,
        get_user_profile.full_name.to_string(), // full name
        get_user_profile.age,
        None // insert null into YCQL DB for comment field, 
    );
 
    let new_insert = create_new_user_profile_query(&mut current_ycql_conn, row).await;


    match new_insert {
        Ok(inserted_ycql_result)  =>  {
            return Ok(web::HttpResponse::Ok()
            .content_type("application/json")
            .body(serde_json::to_string(
                &GenericJsonSuccessResponse::<String>::generic_success_response(CustomStatusMessage::USER_REQUEST_SUCCESS.to_string(), "User profile created successfuly".to_string(), 200, None)
            ).unwrap()))
    },
        Err(e) =>{
            // println!("{:?}", e); // You probably wanna log this error for better tracing and you don't wanna return this back to the user
            return Err(GenericJsonErrorResponse {
                custom_status: CustomStatusMessage::USER_REQUEST_FAILED.to_string(),
                message: UserErrorMessages::InputError{ reason: "We could not create your profile please try again later".to_string()}.to_string(),
                status: 401,
            })
        },
    };

}

///Json Request Struct for |read_user_profile| FUNCTION 
#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct ReadUserProfileJsonREQUEST {
    user_id: Uuid,
    email_address: String
}
#[post("/get_user_profile")]
pub async fn read_user_profile(
    ycql_connection_data: web::Data<Mutex<CurrentYcqlDbSession>>,
    get_id: web::Json<ReadUserProfileJsonREQUEST>
)-> Result<web::HttpResponse<>, GenericJsonErrorResponse>{

    let mut current_ycql_conn   = ycql_connection_data.lock().unwrap();

    let id = &get_id.user_id;
    let email = &get_id.email_address;
    let my_uuid =
    Uuid::parse_str(&id.to_string()).unwrap(); // string to type uuid for query
    // query
    let new_select = select_user_profile_query(&mut current_ycql_conn, my_uuid, email.to_string()).await;
   
    match new_select {
        Ok(selected_ycql_result)  =>  {
        return Ok(web::HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(
            &GenericJsonSuccessResponse::<Vec<UserProfileRowStruct>>::generic_success_response(CustomStatusMessage::USER_REQUEST_SUCCESS.to_string(), "success".to_string(), 200, Some(selected_ycql_result))
        ).unwrap()))
    
    },
        Err(e) =>{
            println!("{:?}", e); // You probably want to log this error for better tracing and you don't want to return this back to the user
            return Err(GenericJsonErrorResponse {
                custom_status: CustomStatusMessage::USER_REQUEST_FAILED.to_string(),
                message: UserErrorMessages::InputError{ reason: "We could not get your profile right noe".to_string()}.to_string(),
                status: 401,
            })
        },
    
    };
}
///Json Request Struct for |update_user_profile| FUNCTION 
#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct UpdateUserProfileJsonREQUEST {
    user_id: String,
    time_uuid_order: String,
    age: i8,
    full_name: String
}
#[post("/update_user_profile")]
pub async fn update_user_profile(   
 ycql_connection_data: web::Data<Mutex<CurrentYcqlDbSession>>,
 get_update_fields: web::Json<UpdateUserProfileJsonREQUEST>
)-> Result<web::HttpResponse, GenericJsonErrorResponse>{
    
    let mut current_ycql_conn   = ycql_connection_data.lock().unwrap();

    let user_id = &get_update_fields.user_id;
    let time_uuid_order = &get_update_fields.time_uuid_order;
    let age = &get_update_fields.age;
    let full_name = &get_update_fields.full_name;
   
    let new_user_id = match Uuid::parse_str(&user_id.to_string()) {
        Ok(new_uuid)  => { new_uuid }
        Err(e) => { 
            return Err(GenericJsonErrorResponse {
            custom_status: CustomStatusMessage::USER_REQUEST_FAILED.to_string(),
            message: UserErrorMessages::InputError{ reason: "Invalid user id format".to_string()}.to_string(),
            status: 401,
        }) }
    };

    let new_time_uuid_order = match Uuid::parse_str(&time_uuid_order.to_string()) {
        Ok(new_uuid)  => { new_uuid }
        Err(e) => { 
            return Err(GenericJsonErrorResponse {
            custom_status: CustomStatusMessage::USER_REQUEST_FAILED.to_string(),
            message: UserErrorMessages::InputError{ reason: "Invalid time_uuid_order format".to_string()}.to_string(),
            status: 401,
        }) }
    };
  
    // query
    let new_update = update_full_name_and_age_query(&mut current_ycql_conn, new_user_id, new_time_uuid_order, *age, full_name.to_string()).await;
    // let new_select = select_user_profile(&mut current_ycql_conn).await;
    match new_update {
        Ok(updated_ycql_result)  =>  {
    
        return Ok(web::HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(
            &GenericJsonSuccessResponse::<String>::generic_success_response(CustomStatusMessage::USER_REQUEST_SUCCESS.to_string(), "User profile updated successfully".to_string(), 200, None)
        ).unwrap()))
    
    },
        Err(e) =>{
            println!("{:?}", e);// You probably want to log this error for better tracing and you don't want to return this back to the user
            return Err(GenericJsonErrorResponse {
                custom_status: CustomStatusMessage::USER_REQUEST_FAILED.to_string(),
                message: UserErrorMessages::InputError{ reason: "Could not update profile".to_string()}.to_string(),
                status: 401,
            })
        },
    
    };
    
}

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct DeleteUserProfileJsonREQUEST {
  pub time_uuid_order: String,
  pub user_id: String
}

#[post("/delete_user")]
pub async fn delete_user_profile(   
 ycql_connection_data: web::Data<Mutex<CurrentYcqlDbSession>>,
 get_fields: web::Json<DeleteUserProfileJsonREQUEST>
)-> Result<web::HttpResponse, GenericJsonErrorResponse>{
    
    let mut current_ycql_conn   = ycql_connection_data.lock().unwrap();

    let time_uuid_order = &get_fields.time_uuid_order;
    let user_id = &get_fields.user_id;
    

   
    let uuid = match Uuid::parse_str(&user_id.to_string()) {
        Ok(new_uuid)  => { new_uuid }
        Err(e) => { 
            return Err(GenericJsonErrorResponse {
            custom_status: CustomStatusMessage::USER_REQUEST_FAILED.to_string(),
            message: UserErrorMessages::InputError{ reason: "Invalid user id format".to_string()}.to_string(),
            status: 401,
        }) }
    };
    let time_uuid = match Uuid::parse_str(&time_uuid_order.to_string()) {
        Ok(new_uuid)  => { new_uuid }
        Err(e) => { 
            return Err(GenericJsonErrorResponse {
            custom_status: CustomStatusMessage::USER_REQUEST_FAILED.to_string(),
            message: UserErrorMessages::InputError{ reason: "Invalid time_uuid_order format ".to_string()}.to_string(),
            status: 401,
        }) }
    };
    // query
    let new_update = delete_user_profile_query(&mut current_ycql_conn, uuid, time_uuid).await;
    // let new_select = select_user_profile(&mut current_ycql_conn).await;
    match new_update {
        Ok(updated_ycql_result)  =>  {
    
        return Ok(web::HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(
            &GenericJsonSuccessResponse::<String>::generic_success_response(CustomStatusMessage::USER_REQUEST_SUCCESS.to_string(), "Successfuly deleted profile".to_string(), 200, None)
        ).unwrap()))
    
    },
        Err(e) =>{
           // println!("{:?}", e); // You probably want to log this error for better tracing and you don't want to return this back to the user
            return Err(GenericJsonErrorResponse {
                custom_status: CustomStatusMessage::USER_REQUEST_FAILED.to_string(),
                message: UserErrorMessages::InputError{ reason: "Could not delete user profile".to_string()}.to_string(),
                status: 401,
            })
        },
    
    };
    
}