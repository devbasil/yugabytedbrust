use crate::yugabyte_db_ycql_config::CurrentYcqlDbSession;
use cdrs_tokio::query::*;
use cdrs_tokio::query_values;
use cdrs_tokio::types::prelude::*;
use cdrs_tokio::frame::AsBytes;
use cdrs_tokio_helpers_derive::*;
use cdrs_tokio::types::from_cdrs::FromCDRSByName;
use uuid::Uuid;
use cdrs_tokio::Result as YugabyteDBResult;
use serde_json::Value as JsonValue;
use serde::{Deserialize, Serialize};
// use chrono::DateTime::timestamp;
use uuid::v1::{Timestamp, Context};
// use std::u32;

///YCQL User Profile Database Schema
#[derive(Clone, Serialize, Debug, IntoCDRSValue, TryFromRow, PartialEq)]
pub struct UserProfileRowStruct {
  pub user_id: Uuid,
  pub time_uuid_order: Uuid,

  pub email_address: String,
  pub full_name: String,
  pub age: i8,

  pub comment: Option<String>,
}

impl UserProfileRowStruct {
  pub  fn into_query_values(self) -> QueryValues {
        query_values!(
            "user_id" => self.user_id, 
            "time_uuid_order" => self.time_uuid_order, 

            "email_address" => self.email_address, 
            "full_name" => self.full_name,
            "age" => self.age,
            "comment"=> self.comment
        )
  }
 pub fn build_user_profile(
  user_id: Uuid,
  time_uuid_order: Uuid,

  email_address: String,
  full_name: String,
  age: i8,

  comment: Option<String>
  )-> UserProfileRowStruct{
        UserProfileRowStruct {
            user_id,
            time_uuid_order,
            email_address,
            full_name,
            age,
            comment
        }

    }

}

///Insert or Create a new user function
pub async fn create_new_user_profile_query(
  ycql_session: &mut CurrentYcqlDbSession, 
  user_profile_row: UserProfileRowStruct
) ->YugabyteDBResult<()>{
    static CREATE_USER_PROFILE: &'static str = r#"
    INSERT INTO demo_ycql_service_keyspace.user_profile (
            user_id,
            time_uuid_order,
            email_address,
            full_name,
            age,
            comment      
    )
      VALUES (?, ?, ?, ?, ?, ?);
    "#;
    ycql_session.query_with_values(CREATE_USER_PROFILE, user_profile_row.into_query_values()).await?;
    Ok(())  
}


pub async fn select_user_profile_query(
  ycql_session: &mut CurrentYcqlDbSession,
   user_id: Uuid,
   email_address: String
  ) ->YugabyteDBResult<Vec<UserProfileRowStruct>>{

    static SELECT_USER_PROFILE: &'static str = r#"
    SELECT * FROM demo_ycql_service_keyspace.user_profile
      WHERE user_id = ? AND  email_address = ?;
    "#;
   let values = query_values!(user_id, email_address);

   let res = ycql_session.query_with_values(SELECT_USER_PROFILE, values).await?;
  
   let body = res.get_body()?;
   
   let new_rows= body.into_rows().expect("Could not get Row Body");

   let mut user_profile: Vec<UserProfileRowStruct> = Vec::with_capacity(new_rows.len());

   for row in new_rows {
    user_profile.push(UserProfileRowStruct::try_from_row(row)?);
   }

   Ok(user_profile)
  }

  pub async fn update_full_name_and_age_query(
     ycql_session: &mut CurrentYcqlDbSession,
     user_id: Uuid,
     time_uuid_order: Uuid,
     age: i8,
     full_name: String
    ) ->YugabyteDBResult<()>{
    
    static UPDATE_USER_PROFILE: &'static str = r#"
    UPDATE demo_ycql_service_keyspace.user_profile SET age = ? , full_name = ? WHERE user_id = ? AND time_uuid_order = ?;
    "#;

     let values = query_values!(age, full_name, user_id, time_uuid_order);

     ycql_session.query_with_values(UPDATE_USER_PROFILE, values).await?;

     Ok(())
    }

    pub async fn delete_user_profile_query(
      ycql_session: &mut CurrentYcqlDbSession,
      user_id: Uuid,
      time_uuid: Uuid
     ) ->YugabyteDBResult<()>{
 
      //  let  my_uuid = Uuid::parse_str("607ad0f4-0000-1000-8000-010203040506").unwrap();
       
       static DELETE_USER_PROFILE: &'static str = r#"
       DELETE FROM demo_ycql_service_keyspace.user_profile  WHERE user_id = ? AND time_uuid_order = ?;
       "#;

      let values = query_values!(user_id, time_uuid);
   
      ycql_session.query_with_values(DELETE_USER_PROFILE, values).await?;
      Ok(())
     }