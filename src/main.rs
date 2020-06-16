use std::fs::File;
use std::io::BufReader;
use actix_files::Files;
use actix_web::{post, web, App, HttpServer, Responder, HttpResponse};
use actix_session::{CookieSession, Session};
use serde::{Deserialize, Serialize};
use blake2::{Blake2b, Digest};
use rustls::internal::pemfile::{certs, rsa_private_keys};
use rustls::{NoClientAuth, ServerConfig};
use log::{error,info,warn};
use log4rs;


use diesel::prelude::*;
//use diesel::mysql::MysqlConnection;
use diesel::r2d2::{self, ConnectionManager};


use rust_login::models::{LoginInfo, NewLoginInfo, LoginInfoModel};
type Pool = r2d2::Pool<ConnectionManager<MysqlConnection>>;

#[derive(Deserialize)]
#[derive(Serialize)]
struct AjaxResult<T> {
    msg: String,
    data: Option<Vec<T>>,
}

const MSG_SUCCESS: &str = "success";
impl<T> AjaxResult<T> {

    pub fn success(data_opt: Option<Vec<T>>) -> Self{
         Self {
             msg: MSG_SUCCESS.to_string(),
             data: data_opt
         }
    }

    pub fn success_without_data() -> Self {
        Self::success(Option::None)
    }
    pub fn success_with_single(single: T) -> Self{
        Self {
            msg:  MSG_SUCCESS.to_string(),
            data: Option::Some(vec![single])
        }
    }

    pub fn fail(msg: String) -> Self {
        Self {
            msg,
            data: None
        }
    }

}

fn sign(text: &str) -> String {
    let sign  = Blake2b::new()
        .chain(b"change me every day")
        .chain(text)
        .result();

    format!("{:X}", sign)
}

const SESSION_USER_KEY: &str = "user_info";
const SESSION_USER_KEY_SIGN: &str = "user_info_sign";

#[post("/register")]
async fn register(
    pool: web::Data<Pool>,
    login_info: web::Json<LoginInfo>
) -> impl Responder {
        use rust_login::schema::tb_login_info;
        let conn:&MysqlConnection = &pool.get().unwrap();
        let new_login_info = NewLoginInfo {
                username: &login_info.username,
                password: &sign(&login_info.password),
        };
        match diesel::insert_into(tb_login_info::table)
                     .values(&new_login_info)
                     .execute(conn) {
                            Ok(info) => HttpResponse::Ok().json(AjaxResult::success_with_single(info)),
                            Err(err) =>  HttpResponse::Forbidden().json(AjaxResult::<String>::fail(err.to_string()))
                     }
}

#[post("/login")]
async fn login(
    session: Session, 
    pool: web::Data<Pool>,
    login_info: web::Json<LoginInfo>) -> impl Responder {

    match session.get::<String>(SESSION_USER_KEY) {
        Ok(Some(user_info)) if user_info == login_info.username => {
            info!("already logged in");
            let user_key_sign = sign(&user_info);
            match session.get::<String>(SESSION_USER_KEY_SIGN) {
                Ok(Some(user_key_sign_session)) if user_key_sign == user_key_sign_session => {
                    HttpResponse::Ok().json(AjaxResult::<bool>::success_without_data())
                }
                _ => {
                    session.remove(SESSION_USER_KEY_SIGN);
                    session.remove(SESSION_USER_KEY);
                    HttpResponse::Forbidden().json(AjaxResult::<bool>::fail("Login time expired".to_string()))
                }
            }

        }
        _ => {
            info!("{} login now", login_info.username);
              
            use rust_login::schema::tb_login_info::dsl::*;
            let conn:&MysqlConnection = &pool.get().unwrap();
            match tb_login_info.filter(username.eq(&login_info.username))
                                        .load::<LoginInfoModel>(conn)   {
                        Ok(login_infos)  if login_infos.len() == 1 && login_infos[0].password == login_info.password =>  {
                            let user_key_sign =  sign(&login_info.username);
                            session.set::<String>(SESSION_USER_KEY_SIGN, user_key_sign);
                            session.set::<String>(SESSION_USER_KEY, login_info.username.clone());
                            HttpResponse::Ok().json(AjaxResult::<bool>::success_without_data())
                        },
                        Ok(login_infos) if login_infos.len() == 0 =>  {
                            HttpResponse::Forbidden().json(AjaxResult::<bool>::fail("username is not exists".to_string() ))
                        }
                        Ok(login_infos) if login_infos.len() > 1 => {
                            error!("username {} duplication", login_info.username);
                            HttpResponse::InternalServerError().json(AjaxResult::<bool>::fail("Illegal username ".to_string() ))
                        }
                        _ =>   HttpResponse::Forbidden().json(AjaxResult::<bool>::fail("password does not match username".to_string()))
            }
        }
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    log4rs::init_file("conf/log4rs.yaml", Default::default()).unwrap();
    let mut app_config = config::Config::new();
    app_config.merge(config::File::with_name("conf/application")).unwrap();

    let is_prod = match app_config.get_str("tl.app.mode")  {
        Ok(value) => {
            let config_file_name = format!("conf/application_{}", value);
            app_config.merge(config::File::with_name(&config_file_name)).unwrap();
            if value == "prod" {true} else {false}
        }
        _ => {
            app_config.merge(config::File::with_name("conf/application_dev")).unwrap();
            false
        }
    };
    app_config.merge(config::Environment::with_prefix("TL_APP")).unwrap();

    //db
    let connspec = app_config.get_str("tl.app.db.url") .expect("db url is required");
         
    let manager = ConnectionManager::<MysqlConnection>::new(connspec);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");


   
    let server = HttpServer::new(move || App::new()
            .data(pool.clone())
                .wrap(
                    CookieSession::signed(&[0; 32]) // <- create cookie based session middleware
                        .secure(is_prod),
                ).service(login)
                 .service(register)
    );

    if is_prod  {

        let mut config = ServerConfig::new(NoClientAuth::new());
        let cert_file = &mut BufReader::new(File::open("./conf/cert.pem").unwrap());
        let key_file = &mut BufReader::new(File::open("./conf/key.pem").unwrap());
        let cert_chain = certs(cert_file).unwrap();
        let mut keys = rsa_private_keys(key_file).unwrap();
        config.set_single_cert(cert_chain, keys.remove(0)).unwrap();

        server.bind_rustls("127.0.0.1:8443", config)?
            .run()
            .await
    }else {
        server.bind("127.0.0.1:8088")?
            .run()
            .await
    }


}

#[cfg(test)]
mod test {
    use blake2::{Blake2b, Digest};
    //第一次使用blake2 先写个测试锻炼下
    // 开发rust编写测试的方法 可以参考
    // https://edu.51cto.com/sd/eed8c
    #[test]
    fn black2_test() {
        let sign_valid = Blake2b::new()
            .chain(b"change me every day")
            .chain("username")
            .result();
        let sign_str = format!("{:X}", sign_valid);
        assert_eq!("176F4287DD8011D78B2A38E201D9CBAF8AD0E237A475570B4583818BE80E8DFA92764B322FCDD2583D0E28D2B940F8F281B31B6999D05D5F2F8AF9FF8AA6BBA6",
                   sign_str);
    }
}
