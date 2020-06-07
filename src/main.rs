use actix_web::{post, web, App, HttpServer, Responder, HttpResponse};
use actix_session::{CookieSession, Session};
use serde::{Deserialize, Serialize};
use blake2::{Blake2b, Digest};

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

fn sign(text: &str) -> String {
    let sign  = Blake2b::new()
        .chain(b"change me every day")
        .chain(text)
        .result();

    format!("{:X}", sign)
}

const SESSION_USER_KEY: &str = "user_info";
const SESSION_USER_KEY_SIGN: &str = "user_info_sign";

#[post("/login")]
async fn index(session: Session, login_info: web::Json<LoginInfo>) -> impl Responder {

    match session.get::<String>(SESSION_USER_KEY) {
        Ok(Some(user_info)) if user_info == login_info.username => {
            println!("already logged in");
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
            println!("login now");
            if login_info.username == login_info.password {
                let user_key_sign =  sign(&login_info.username);
                session.set::<String>(SESSION_USER_KEY_SIGN, user_key_sign);
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
