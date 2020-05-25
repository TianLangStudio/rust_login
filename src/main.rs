use actix_web::{post, web, App, HttpServer, Responder, HttpResponse};
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

#[post("/login")]
async fn index(login_info: web::Json<LoginInfo>) -> impl Responder {
    if login_info.username == login_info.password {
        HttpResponse::Ok().json(AjaxResult::<bool>::success_without_data())
    } else {
        HttpResponse::Forbidden().json(AjaxResult::<bool>::fail("password must match username".to_string()))
    }

}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(index))
        .bind("127.0.0.1:8088")?
        .run()
        .await
}
