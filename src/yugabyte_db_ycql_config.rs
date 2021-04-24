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

pub type CurrentYcqlDbSession = Session<RoundRobin<TcpConnectionPool>>;

/// Create DB session NOTE: ***in production consider using ***StaticPasswordAuthenticator**** instead of NoneAuthenticator
pub async fn configure_yugabyte_db_session() -> YugabyteDBResult<CurrentYcqlDbSession>{
    let node = NodeTcpConfigBuilder::new("localhost:9042", Arc::new(NoneAuthenticator {})).build();
    let cluster_config = ClusterTcpConfig(vec![node]);
    let newsession = new_session(&cluster_config, RoundRobin::new()).await?;
    Ok(newsession)
}
/// Create demo_ycql_service_keyspace or ***DATABASE*** if you are coming from a NoSQL world
pub async fn create_demo_ycql_service_keyspace(yugabyte_ycql_session: &mut CurrentYcqlDbSession) ->YugabyteDBResult<()> {
    let demo_ycql_service_keyspace: &'static str = "CREATE KEYSPACE IF NOT EXISTS demo_ycql_service_keyspace WITH REPLICATION = { \
        'class' : 'SimpleStrategy', 'replication_factor' : 1 };";
        yugabyte_ycql_session.query(demo_ycql_service_keyspace).await?; // ***Propagate error to the calling function
    Ok(())    
}
/// Create demo_ycql_user_profile_table
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




