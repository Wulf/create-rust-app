// use redis::Commands;
// use redis::Connection;

use redis::Client;
use redis::Commands;
use redis::Connection;
use redis::JsonCommands;
use redis::RedisError;
use redis::RedisResult;
use redis::ToRedisArgs;

use redis_macros::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

use std::cell::RefCell;

use serde_json::json;

#[derive(Debug, Serialize, Deserialize, Clone, FromRedisValue, ToRedisArgs)]
pub struct Todo {
    pub text: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct RedisDB {
    pub conn: RefCell<Connection>,
}

impl RedisDB {
    pub fn init() -> Result<RedisDB, redis::RedisError> {
        // "redis://127.0.0.1:6379"

        let client = redis::Client::open(Self::connection_url())?;
        Ok(RedisDB {
            conn: RefCell::new(client.get_connection()?),
        })
    }

    pub fn connection_url() -> String {
        std::env::var("REDIS_URL").expect("REDIS_URL environment variable expected.")
    }

    pub fn create(&mut self, item: &Todo) -> Result<isize, redis::RedisError> {
        let mut con = self.conn.borrow_mut();
        // throw away the result, just make sure it does not fail
        println!("1111{:?}", &json!(item).to_string());
        let res = con.lpush("todo_list", &json!(item).to_string())?;

        Ok(res)
    }

    pub fn getList(&mut self) -> Result<Vec<Todo>, redis::RedisError> {
        let mut con = self.conn.borrow_mut();
        // throw away the result, just make sure it does not fail
        let res_list: Vec<Todo> = con.lrange("todo_list", 0, 10)?;
        Ok(res_list)
    }
}
