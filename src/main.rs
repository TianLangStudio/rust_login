use actix_web::{post, web, App, HttpServer, Responder, HttpResponse};
use actix_session::{CookieSession, Session};
use serde::{Deserialize, Serialize};


#[derive(Deserialize)]
struct LoginInfo {
   username: String,
   password: String,
}

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

const SESSION_USER_KEY: &str = "user_info";
#[post("/login")]
async fn index(session: Session, login_info: web::Json<LoginInfo>) -> impl Responder {

    match session.get::<String>(SESSION_USER_KEY) {
        Ok(Some(user_info)) if user_info == login_info.username => {
            println!("already logged in");
            HttpResponse::Ok().json(AjaxResult::<bool>::success_without_data())
        }
        _ => {
            println!("login now");
            if login_info.username == login_info.password {
                session.set::<String>(SESSION_USER_KEY, login_info.username.clone());
                HttpResponse::Ok().json(AjaxResult::<bool>::success_without_data())
            } else {
                HttpResponse::Forbidden().json(AjaxResult::<bool>::fail("password must match username".to_string()))
            }
        }
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new()
        .wrap(
            CookieSession::signed(&[0; 32]) // <- create cookie based session middleware
                .secure(false),
        ).service(index))
        .bind("127.0.0.1:8088")?
        .run()
        .await
}
