use serde::{Deserialize, Serialize};
use super::schema::tb_login_info;


#[derive(Deserialize, Serialize, Queryable)]
pub struct LoginInfo {
    pub id: Option<i64>,
    pub username: String,
    pub password: String,
}
#[derive(Queryable)]
pub struct LoginInfoModel {
    pub id: i64,
    pub username: String,
    pub password: String,
}

#[derive(Insertable)]
#[table_name="tb_login_info"]
pub struct NewLoginInfo <'a>{
       pub username:  &'a str,
       pub password: &'a str,
}