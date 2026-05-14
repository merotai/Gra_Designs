


use anyhow::{
    Result,
    anyhow
};
use mongodb::{Client, Collection, bson, Database};

use dotenv::dotenv;
use std::env;


/// 用于数据库连接
#[derive(Clone)]
pub struct MongoDBClient {
    db_client: Client,
    db_name: String,
}

impl MongoDBClient {

    pub async fn db_connect() -> Result<Self> {
        dotenv().ok();

        let db_uri_l = env::var("MONGODB_URI").expect("MONGODB_URI env variable not set");
        let db_name_l = env::var("MONGODB_NAME").expect("MONGODB_NAME env variable not set");

        println!("[data_base:MongoDB] connecting to mongodb database: {}", db_uri_l);
        let db_client_l = Client::with_uri_str(&db_uri_l).await?;

        match db_client_l
            .database(&db_name_l)
            .run_command(bson::doc! {"ping":1})
            .await {
            Ok(_ok) => {
                println!("[data_base:MongoDB]成功连接到mongodb数据库");
                Ok(Self {
                    db_client: db_client_l,
                    db_name: db_name_l,
                })
            },
            Err(err) => {
                println!("[data_base:MongoDB]连接mongodb数据库失败");
                Err(anyhow!("连接数据库失败: {:?}", err))
            }
        }

    }

    pub fn get_db(&self) -> Database {
        self.db_client.database(&self.db_name)
    }
}

