# Yugabyte DB Rust CRUD Examples

This is a rust crud example for connecting and interacting with yugabyte's database *YCQL API*.
[YugabyteDB](https://www.yugabyte.com/). is a high-performance, cloud-native distributed SQL database. It is fully Open Source and has great **default Enterprise Grade features** in the free version compared to other open source distributed database projects.


## Prerequisite

* We will use [cdrs-tokio](https://github.com/krojew/cdrs-tokio) asynchronous driver for connecting to Yugabyte DB.

    >**NOTE:** *the driver uses RUST's* **tokio runtime** therefore if you are going to use a rust WEB framework,
    >**Make sure** your web framework can integrate smoothly with the latest version of **tokio**, or atleast a minimum version 1 of the *tokio runtime*. 



* Get familiar with **Yugabyte's** data type and the **RUST Driver Type Mapping** 

    Primitive types (T) 

    | Yugabyte | RUST     |
    |----------|----------|
    |BIGINT    | i64      |
    |BOOLEAN   | bool     |
    |TEXT      | String   |
    |VARCHAR   | String   |
    |SMALLINT  | i16      |
    |UUID      |[Uuid](https://docs.rs/uuid/0.8.2/uuid/)         |
    |TIMEUUID  |[Uuid](https://docs.rs/uuid/0.8.2/uuid/) Version 1 of the UUID, but the type still remains Uuid in Rust,  this will be correctly mapped to TIMEUUID  |
    |TINYINT   | i8       |
    |INT       | i32      |
    |INTEGER   | i32      |
    |FLOAT     | f32      |
    |JSONB     | Treat them as RUST String according to Official [Doc] (https://docs.yugabyte.com/latest/api/ycql/type_jsonb/) under semantics section "JsonB can be compared to TEXT/VARCHAR", and then use [serde](https://docs.serde.rs/serde/index.html) for struct converstions..        |
    |UDT       | Supports struct Custom Implementations,        |
    |LIST      | Vec<T> see how to create udt using [list](https://docs.yugabyte.com/latest/api/ycql/ddl_create_type/)|
    |MAP       | HashMap<String, T>  |
   
*  **TO DO**
- [*] Add primitive types YCQL crud example
- [] Add Complex types example eg. UDT- List and Map
- [] Add JsonB example
- [] Add full list of *Yugabyte data type and corresponding RUST Driver Type Mapping* only after testing


## **EXAMPLES** 
*Read the Prerequisite before diving into the examples*


* Add dependencies

    ```rust
    [dependencies]
    # For YCQL connection (Mandatory)
    cdrs-tokio = "3.0.0"
    cdrs-tokio-helpers-derive = "2.0.0"

     # For handling UUID (Mandatory)
    uuid = { version = "0.8", features = ["serde", "v4", "v1"] } 

    # For Json deserealization (Mandatory)
    serde = { version = "1.0", features = ["derive"] }
    serde_json = "1.0"
    
    # For Handling timestamps (Mandatory)
    chrono = "0.4.19"

    # For validating struct fields (Optional)
    validator = { version = "0.12", features = ["derive"] }


    # Rest web framework (Optional)
    actix-files = "0.6.0-beta.4"
    actix-http = "=3.0.0-beta.5"
    actix-rt = "2.2.0"
    actix-service = "=2.0.0-beta.5"
    actix-web = "=4.0.0-beta.5"
    # JWT Authentication  (Optional)
    actix-web-httpauth = "0.5.1"

   
   # (Optional)
    derive_more = "0.99.11"
    r2d2 = "0.8.9"
    futures = "0.3.13"
    dotenv = "0.15.0" 
    time = "0.2.26"
    env_logger = "0.8"
    ```


## Section 1 

###  A
* Create a module file named **yugabyte_db_ycql_config.rs** to store our database configurations and add the following crates.

```rust
use cdrs_tokio::authenticators::NoneAuthenticator;
use cdrs_tokio::cluster::session::{new as new_session, Session};
use cdrs_tokio::cluster::{ClusterTcpConfig, NodeTcpConfigBuilder, TcpConnectionPool};
use cdrs_tokio::load_balancing::RoundRobin;
use cdrs_tokio::query::*;
use cdrs_tokio::types::prelude::*;
use cdrs_tokio_helpers_derive::*;
use cdrs_tokio::Result as YugabyteDBResult;
use std::sync::Arc;
use std::result::Result;
use std::error;
use std::sync::Mutex;

```
### B
* Let's add a Yugabyte Database connection Pool session which can be shared by all the controllers or end routes. *Create a connection pool type, and then import cdrs_tokio::Result as **YugabyteDBResult** type*.  In the **configure_yugabyte_db_session** function below, we have used the *? operator* to propagate any errors to the calling function and then returning the session which is of  type **YugabyteDBResult**.

In production consider using *StaticPasswordAuthenticator* instead of *NoneAuthenticator* used below.

In the **configure_yugabyte_db_session** function below, we also need to configure and build each node using the **NodeTcpConfigBuilder** struct, this will configure TCP connection for a node, we can then use **ClusterTcpConfig** which holds per node TCP config, in our use case we have only used one node.

We will use a *RoundRobin* load balancing Strategy implementation provided by *cdrs_tokio*, **NOTE: You may use your own load balancing strategy if this does not satsify your use case.**

```rust

/// Create a connection pool type
pub type CurrentYcqlDbSession = Session<RoundRobin<TcpConnectionPool>>;

pub async fn configure_yugabyte_db_session() -> YugabyteDBResult<CurrentYcqlDbSession>{
    let node = NodeTcpConfigBuilder::new("localhost:9042", Arc::new(NoneAuthenticator {})).build();
    let cluster_config = ClusterTcpConfig(vec![node]);
    let newsession = new_session(&cluster_config, RoundRobin::new()).await?; // Propagate any errors to the calling function
    Ok(newsession) // return the session if there are no errors
}

```

### C
* Create a demo keyspace named **demo_ycql_service_keyspace** or ***A DATABASE*** if you are coming from SQL world. And again propagate any errors to the calling function using the *? operator*. We don't need any result here, we only need to know whether the creation of the keyspace was successful or not that's why we are returning a unit type or an empty result. Notice we need to pass a *mutable session* in order to create the keyspace. The session will be created by calling **configure_yugabyte_db_session** function and passed as a parameter to **create_demo_ycql_service_keyspace** function. 

```rust
pub async fn create_demo_ycql_service_keyspace(yugabyte_ycql_session: &mut CurrentYcqlDbSession) ->YugabyteDBResult<()> {
    let demo_ycql_service_keyspace: &'static str = "CREATE KEYSPACE IF NOT EXISTS demo_ycql_service_keyspace WITH REPLICATION = { \
        'class' : 'SimpleStrategy', 'replication_factor' : 1 };";
        yugabyte_ycql_session.query(demo_ycql_service_keyspace).await?;
    Ok(())    
}

```

### D
* Create a demo table named **user_profile**  under **demo_ycql_service_keyspace** keyspace above. And again propagate any errors to the calling function using the ? operator. We don't need any result here, we only need to know whether the creation of the table was successful or not, that's why we are returning a unit type or an empty result*. 

We will use the **user_id** column as a **primary key** which in essence is the **Partition key** while the **time_uuid_order** column will be used  as  a **clustering key**

Read about [Yugabyte data modeling](https://docs.yugabyte.com/latest/develop/learn/data-modeling-ycql/) here.

**NOTE:** the use of primitive types below, which will be automatically converted to and from the corresponding RUST data types under the hood by the Rust driver, **READ Prerequisites section above about *Yugabyte and RUST Driver Type Mapping*, the complete list will be added only after testing, the listed ones has been tested**.

```rust
pub async fn create_demo_ycql_user_profile_table(yugabyte_ycql_session: &mut CurrentYcqlDbSession) ->YugabyteDBResult<()> {
        let create_user_profile: &'static str = r#"
        CREATE TABLE IF NOT EXISTS demo_ycql_service_keyspace.user_profile(
            user_id UUID,
            time_uuid_order TIMEUUID,
            email_address TEXT,
            full_name TEXT,
            age TINYINT,
            comment TEXT,

            PRIMARY KEY ((user_id), time_uuid_order))
            WITH transactions = { 'enabled' : true };
            "#;
            yugabyte_ycql_session.query(create_user_profile).await?; //***Propagate error to the calling function
    Ok(())  
}
```


## Section B 

###  A
* Create a module file named **ycql_db_schema_and_queries.rs** to store our query functions and database access schema. add the following crates.

```rust
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

```

### B
* Create an access and retrieval struct schema, Notice the **IntoCDRSValue** and **TryFromRow** derive macros used for conversion and retrieval of data to and from the yugabytes's YCQL rows and Rust Struct Rows. Rust's Macro system is beyond the scope of this lesson you can read more about [rust macro use](https://doc.rust-lang.org/book/ch19-06-macros.html) here. 

**Hint** *When creating a common access and retrieval struct schema for different micro-services, consider using **Option** Rust data Type for non-mandatory struct fields eg. the **comment field** in the **UserProfileRowStruct** struct below*.

The function named **into_query_values** takes Rust Struct fields and convert them to values that can be read by Yugabyte Database.


```rust
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
```

### C
* The **create** new user function named **create_new_user_profile_query**, takes two arguments, the first one is a mutable yugabyte connetion session of type *CurrentYcqlDbSession*  returned by calling the *configure_yugabyte_db_session* function. The second argument  is a an implementation function **build_user_profile** for **UserProfileRowStruct** struct above. 
```rust
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
```

### D
* **Read** function named **select_user_profile_query**, takes three arguments, the first one is a mutable yugabyte connetion session of type *CurrentYcqlDbSession*  returned by calling the **configure_yugabyte_db_session** function. The second one is **user_id** of type *Uuid* column and an email address column named **email_address** of type *text*. You can then query data from *user_profile* WHERE the given values matches.

The **get_body** fuction extracts the Yugabyte result body and then **into_rows** function converts the body into RUST struct, we then get an array of the rows and return results to the calling fuction.

```rust

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
```

### E
* **Update** function named **update_full_name_and_age_query**, takes five arguments, *ycql_session, user_id, time_uuid_order, age, and full_name*, the session is the mutable yugabyte connetion session of type *CurrentYcqlDbSession*  returned by calling the **configure_yugabyte_db_session** function. 

**NOTE** time_uuid_order is of type *Uuid*, in the query but in yugabyte it is natively type *TIMEUUID* this will be correctly mapped as you will be using V1 of Uuid as the TIMEUUID.

The **query_values!** prepares the values, do the query and then propagate any errors to the calling function.

```rust
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
```

### F
* **Delete** function named **delete_user_profile_query**, takes 3 arguments, *ycql_session, user_id and time_uuid_order*, the session is the mutable yugabyte connetion session of type *CurrentYcqlDbSession*  returned by calling the **configure_yugabyte_db_session** function. Everything else is self explanatory if you have been followed from the start.

```rust
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
```


## Initializing the database, creating a session connection pool and passing the connection session to crud controllers

**Important Note, Read the tips Below**

### TIPS
> TIP 1: See how **yugabyte_db_connection** is passed to all CRUD API controller routes using MUTEX, in the main function below. 
> This will however be **dependent** on the rust web framework you are using, for our tests we used Actix-web 4.0.0-beta.5 web framework


> TIP 2: It is upto you to choose the web framework you are most comfortable working with.


>TIP 4: If you are using Actix-web 4.0.0-beta.5 or above framework here is how you would extract the session data from all your controller function use the following
> extract session connection by passing  **ycql_connection_data: web::Data<Mutex<CurrentYcqlDbSession>>** as an argument
> and then use it like this  **let mut current_ycql_conn   = ycql_connection_data.lock().unwrap();**


##  main function example using Actix Web 

```rust
#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    // TIP You may use match statements to avoid panics
    let mut yugabyte_db_connection = configure_yugabyte_db_session().await.expect("Fatal YCQL session DB connection Error");
    // TIP You may use match statements to avoid panics
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

```


> TIP 3: Depending on the Rust web framework of your choice interacting with the yugabyte's YCQL API is as simple as calling the following functions and then passing the appropriate parameters. example



> create_new_user_profile_query() inside your controller function route eg http://127.0.0.1:4055/api_v1/create


> select_user_profile_query() inside your controller function route eg http://127.0.0.1:4055/api_v1/read


> update_full_name_and_age_query() inside your controller function route eg http://127.0.0.1:4055/api_v1/update


> delete_user_profile_query() inside your controller function route eg http://127.0.0.1:4055/api_v1/delete



### FOR A WORKING APP EXAMPLE USING ACTIX WEB

You can check how to invoke the query functions with appropriate parameters by cloning a working REST JSON API example below

>see the crud_controller.rs
>git clone https://github.com/devbasil/yugabytedbrust.git



























