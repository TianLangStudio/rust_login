use actix_web::{post, web, App, HttpServer, Responder, HttpResponse};
use serde::Deserialize;
#[derive(Deserialize)]
struct LoginInfo {
   username: String,
   password: String,
}
#[post("/login")]
async fn index(login_info: web::Json<LoginInfo>) -> impl Responder {
    if login_info.username == login_info.password {
        HttpResponse::Ok().json("success")
    } else {
        HttpResponse::Forbidden().json("password error")
    }

}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(index))
        .bind("127.0.0.1:8088")?
        .run()
        .await
}
