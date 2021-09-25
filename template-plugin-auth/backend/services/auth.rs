extern crate argonautica;
use crate::extractors::auth::Auth;
use actix_http::http::StatusCode;
use crate::models::user::{User, UserChangeset};
use crate::models::user_session::{UserSession, UserSessionChangeset};
use crate::models::permissions::Permission;
use crate::models::{UTC, ID, PaginationParams};
use crate::Pool;
use crate::mail;
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde_json::json;

use actix_web::{delete, get, post, Error as AWError};
use actix_web::{web, HttpResponse, HttpMessage};
use actix_http::cookie::{Cookie, SameSite};

const COOKIE_NAME: &'static str = "request_token";

#[tsync::tsync]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSessionJson {
  pub id: ID,
  pub device: Option<String>,
  pub created_at: UTC,
  pub updated_at: UTC,
}

#[tsync::tsync]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSessionResponse {
  pub sessions: Vec<UserSessionJson>,
  pub num_pages: i64
}

#[get("/sessions")]
async fn sessions(
  pool: web::Data<Pool>,
  auth: Auth,
  web::Query(info): web::Query<PaginationParams>
) -> Result<HttpResponse, AWError> {
  let db = pool.get().unwrap();

  let sessions = UserSession::read_all(&db, &info, auth.user_id);

  if sessions.is_err() {
    return Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body("{ \"message\": \"Could not fetch sessions.\" }"));
  }

  let sessions: std::vec::Vec<crate::models::user_session::UserSession> = sessions.unwrap();
  let mut sessions_json: Vec<UserSessionJson> = vec![];

  for session in sessions {
    let session_json = UserSessionJson {
      id: session.id,
      device: session.device,
      created_at: session.created_at,
      updated_at: session.updated_at
    };

    sessions_json.push(session_json);
  }

  let num_sessions = UserSession::count_all(&db, auth.user_id);
  if num_sessions.is_err() {
    return Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body("{ \"message\": \"Could not fetch sessions.\" }"));
  }

  let num_sessions = num_sessions.unwrap();
  let num_pages = (num_sessions / info.page_size) + (if num_sessions % info.page_size != 0 { 1 } else { 0 });

  let resp = UserSessionResponse {
    sessions: sessions_json,
    num_pages: num_pages
  };

  Ok(HttpResponse::Ok().json(resp))
}

#[delete("/sessions/{id}")]
async fn destroy_session(
  pool: web::Data<Pool>,
  web::Path(item_id): web::Path<ID>,
  auth: Auth
) -> Result<HttpResponse, AWError> {
  let db = pool.get().unwrap();

  let user_session = UserSession::read(&db, item_id);

  if user_session.is_err() {
    return Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body(json!({
      "message": "Internal error."
    })))
  }
  
  let user_session = user_session.unwrap();
  
  if user_session.user_id != auth.user_id {
    return Ok(HttpResponse::build(StatusCode::NOT_FOUND).body(json!({
      "message": "Session not found."
    })))
  }
  
  if UserSession::delete(&db, user_session.id).is_err() {
    return Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body(json!({
      "message": "Could not delete session."
    })))
  }
  
  Ok(HttpResponse::build(StatusCode::OK).body(json!({
    "message": "Deleted."
  })))
}

#[delete("/sessions")]
async fn destroy_sessions(
  pool: web::Data<Pool>,
  auth: Auth
) -> Result<HttpResponse, AWError> {
  let db = pool.get().unwrap();

  if UserSession::delete_all_for_user(&db, auth.user_id).is_err() {
    return Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body(json!({
      "message": "Could not delete sessions."
    })))
  }

  Ok(HttpResponse::build(StatusCode::OK).body(json!({
    "message": "Deleted."
  })))
}

#[derive(Deserialize, Serialize)]
struct LoginInput {
  email: String,
  password: String,
  device: Option<String>
}

#[tsync::tsync]
#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub exp: usize,
    pub sub: ID,
    pub token_type: String,
    pub permissions: Vec<Permission>
}

#[derive(Debug, Serialize, Deserialize)]
struct RefreshTokenClaims {
    exp: usize,
    sub: ID,
    token_type: String
}

#[post("/login")]
async fn login(
  pool: web::Data<Pool>,
  web::Json(item): web::Json<LoginInput>
) -> Result<HttpResponse, AWError> {
  let db = pool.get().unwrap();

  // verify device
  let mut device = None;
  if item.device.is_some() {
    device = match item.device.unwrap().as_str() {
      "web" => Some("web".to_string()),
      "mobile" => Some("mobile".to_string()),
      _ => None
    };
  }
  
  let user = User::find_by_email(&db, item.email);
  
  if user.is_err() {
    return Ok(HttpResponse::build(StatusCode::UNAUTHORIZED).body("{ \"message\": \"Invalid credentials.\" }"));
  }
  
  let user = user.unwrap();

  if !user.activated {
    return Ok(HttpResponse::build(StatusCode::BAD_REQUEST).body("{ \"message\": \"Account has not been activated.\" }"))
  }
  
  let mut verifier = argonautica::Verifier::default();
  let is_valid = verifier
    .with_hash(&user.hash_password)
    .with_password(&item.password)
    .with_secret_key(match std::env::var("SECRET_KEY") {
      Ok(s) => s,
      Err(_) => panic!("No SECRET_KEY environment variable set!"),
    })
    .verify()
    .unwrap();
  
  if !is_valid {
    return Ok(HttpResponse::build(StatusCode::UNAUTHORIZED).body("{ \"message\": \"Invalid credentials.\" }"));
  }

  let permissions = Permission::for_user(&db, user.id);
  if permissions.is_err() { println!("{:#?}", permissions.err()); return Ok(HttpResponse::InternalServerError().body("{ \"message\": \"An internal server error occurred.\" }")) }
  let permissions = permissions.unwrap();
  
  let access_token_claims = AccessTokenClaims {
    exp: (chrono::Utc::now() + chrono::Duration::minutes(15)).timestamp() as usize,
    sub: user.id,
    token_type: "access_token".to_string(),
    permissions
  };

  let refresh_token_claims = RefreshTokenClaims {
    exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    sub: user.id,
    token_type: "refresh_token".to_string()
  };
  
  let access_token = encode(
    &Header::default(),
    &access_token_claims,
    &EncodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref())
  ).unwrap();

  let refresh_token = encode(
    &Header::default(),
    &refresh_token_claims,
    &EncodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref())
  ).unwrap();

  let user_session = UserSession::create(&db, &UserSessionChangeset {
    user_id: user.id,
    refresh_token: refresh_token.clone(),
    device: device
  });

  if user_session.is_err() {
    return Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body("{ \"message\": \"Could not create a session.\" }"));
  }
  
  Ok(HttpResponse::build(StatusCode::OK)
    .cookie(Cookie::build(COOKIE_NAME, refresh_token.clone())
              .secure(true)
              .http_only(true)
              .same_site(SameSite::Strict)
              .finish())
    .body(json!({
      "access_token": access_token
    }))
  )
}

#[post("/logout")]
async fn logout(
  pool: web::Data<Pool>,
  req: web::HttpRequest
) -> Result<HttpResponse, AWError> {
  let db = pool.get().unwrap();

  let refresh_token_cookie = req.cookie(COOKIE_NAME);
  
  if refresh_token_cookie.is_none() {
    return Ok(HttpResponse::build(StatusCode::UNAUTHORIZED).body(json!({
      "message": "Invalid session"
    })))
  }

  let refresh_token_cookie_unwrapped = refresh_token_cookie.clone().unwrap();
  let refresh_token_str = refresh_token_cookie_unwrapped.value();

  let session = UserSession::find_by_refresh_token(&db, refresh_token_str);

  if session.is_err() {
    return Ok(HttpResponse::build(StatusCode::UNAUTHORIZED).body("{ \"message\": \"Invalid session.\" }"));
  }

  let session = session.unwrap();

  let is_deleted = UserSession::delete(&db, session.id);

  if is_deleted.is_err() {
    return Ok(HttpResponse::build(StatusCode::UNAUTHORIZED).body("{ \"message\": \"Could not delete session.\" }"));
  }
  
  let mut builder = HttpResponse::Ok();

  if let Some(ref cookie) = refresh_token_cookie {
      builder.del_cookie(cookie);
  }

  Ok(builder.finish())
}

#[post("/refresh")]
async fn refresh(
  pool: web::Data<Pool>,
  req: web::HttpRequest
) -> Result<HttpResponse, AWError> {
  let db = pool.get().unwrap();
  
  let cookie = req.cookie(COOKIE_NAME);

  if cookie.is_none() {
    return Ok(HttpResponse::build(StatusCode::UNAUTHORIZED).body(json!({
      "message": "Invalid session"
    })))
  }

  let cookie: Cookie = cookie.unwrap();

  let refresh_token_str = cookie.value();

  let refresh_token = decode::<RefreshTokenClaims>(
    &refresh_token_str,
    &DecodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref()),
    &Validation::default()
  );

  if refresh_token.is_err() {
    return Ok(HttpResponse::build(StatusCode::UNAUTHORIZED).body("{ \"message\": \"Invalid token.\" }"));
  }

  let refresh_token = refresh_token.unwrap();

  if !refresh_token.claims.token_type.eq_ignore_ascii_case("refresh_token") {
    return Ok(HttpResponse::build(StatusCode::UNAUTHORIZED).body("{ \"message\": \"Invalid token.\" }"));
  }

  let session = UserSession::find_by_refresh_token(&db, refresh_token_str);
  
  if session.is_err() {
    return Ok(HttpResponse::build(StatusCode::UNAUTHORIZED).body("{ \"message\": \"Invalid session.\" }"));
  }

  let session = session.unwrap();

  let permissions = Permission::for_user(&db, session.user_id);
  if permissions.is_err() { return Ok(HttpResponse::InternalServerError().body("{ \"message\": \"An internal server error occurred.\" }")) }
  let permissions = permissions.unwrap();

  let access_token_claims = AccessTokenClaims {
    exp: (chrono::Utc::now() + chrono::Duration::minutes(15)).timestamp() as usize,
    sub: session.user_id,
    token_type: "access_token".to_string(),
    permissions
  };

  let refresh_token_claims = RefreshTokenClaims {
    exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    sub: session.user_id,
    token_type: "refresh_token".to_string()
  };
  
  let access_token = encode(
    &Header::default(),
    &access_token_claims,
    &EncodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref())
  ).unwrap();

  let refresh_token_str = encode(
    &Header::default(),
    &refresh_token_claims,
    &EncodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref())
  ).unwrap();

  // update session with the new refresh token
  let session_update = UserSession::update(&db, session.id, &UserSessionChangeset {
    user_id: session.user_id,
    refresh_token: refresh_token_str.clone(),
    device: session.device
  });
  
  if session_update.is_err() {
    return Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body("{ \"message\": \"Could not update the session.\" }"));
  }
  
  Ok(HttpResponse::build(StatusCode::OK)
    .cookie(Cookie::build(COOKIE_NAME, refresh_token_str)
              .secure(true)
              .http_only(true)
              .same_site(SameSite::Strict)
              .finish())
    .body(json!({
      "access_token": access_token
    }))
  )
}

#[derive(Serialize, Deserialize)]
struct RegisterInput {
  email: String,
  password: String
}

#[derive(Debug, Serialize, Deserialize)]
struct RegistrationClaims {
    exp: usize,
    sub: ID,
    token_type: String
}

#[post("/register")]
async fn register(
  pool: web::Data<Pool>,
  web::Json(item): web::Json<RegisterInput>,
  mailer: web::Data<mail::Mailer>
) -> Result<HttpResponse, AWError> {
  let db = pool.get().unwrap();

  let user = User::find_by_email(&db, (&item.email).to_string());

  if user.is_ok() {
    let user = user.unwrap();
    if !user.activated {
      User::delete(&db, user.id).unwrap();
    } else {
      return Ok(HttpResponse::build(StatusCode::BAD_REQUEST).body("{ \"message\": \"Already registered.\" }"))
    }
  }

  let mut hasher = argonautica::Hasher::default();
  let hash = hasher
      .with_password(&item.password)
      .with_secret_key(match std::env::var("SECRET_KEY") {
        Ok(s) => s,
        Err(_) => panic!("No SECRET_KEY environment variable set!"),
      })
      .hash()
      .unwrap();

  let user = User::create(&db, &UserChangeset {
    activated: false,
    email: item.email,
    hash_password: hash
  }).unwrap();

  let registration_claims = RegistrationClaims {
    exp: (chrono::Utc::now() + chrono::Duration::days(30)).timestamp() as usize,
    sub: user.id,
    token_type: "activation_token".to_string()
  };
  
  let token = encode(
    &Header::default(),
    &registration_claims,
    &EncodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref())
  ).unwrap();
  
  mail::auth_register::send(&mailer, &user.email, &format!("http://localhost:8080/auth/activate?token={token}", token=token));
  
  Ok(HttpResponse::build(StatusCode::OK).body("{ \"message\": \"Registered! Check your email to activate your account.\" }"))
}

#[derive(Serialize, Deserialize)]
struct ActivationInput {
  activation_token: String
}

#[get("/activate")]
async fn activate(
  pool: web::Data<Pool>,
  web::Query(item): web::Query<ActivationInput>,
  mailer: web::Data<mail::Mailer>
) -> Result<HttpResponse, AWError> {
  let db = pool.get().unwrap();

  let token = decode::<RegistrationClaims>(
    &item.activation_token,
    &DecodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref()),
    &Validation::default()
  );

  if token.is_err() {
    return Ok(HttpResponse::build(StatusCode::UNAUTHORIZED).body("{ \"message\": \"Invalid token.\" }"));
  }
  
  let token = token.unwrap();

  if !token.claims.token_type.eq_ignore_ascii_case("activation_token") {
    return Ok(HttpResponse::build(StatusCode::UNAUTHORIZED).body("{ \"message\": \"Invalid token.\" }"));
  }

  let user = User::read(&db, token.claims.sub);

  if user.is_err() {
    return Ok(HttpResponse::build(StatusCode::BAD_REQUEST).body("{ \"message\": \"Invalid token.\" }"));
  }
  
  let user = user.unwrap();

  if user.activated {
    return Ok(HttpResponse::build(StatusCode::OK).body("{ \"message\": \"Already activated!\" }"))
  }

  let activated_user = User::update(&db, user.id, &UserChangeset {
    activated: true,
    email: user.email.clone(),
    hash_password: user.hash_password
  });
  
  if activated_user.is_err() {
    return Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body("{ \"message\": \"Could not activate user.\" }"));
  }

  mail::auth_activated::send(&mailer, &user.email);
  
  Ok(HttpResponse::build(StatusCode::OK).body("{ \"message\": \"Activated!\" }"))
}

#[derive(Serialize, Deserialize)]
struct ForgotInput {
  email: String
}

#[derive(Debug, Serialize, Deserialize)]
struct ResetTokenClaims {
  exp: usize,
  sub: ID,
  token_type: String
}

#[post("/forgot")]
async fn forgot_password(
  pool: web::Data<Pool>,
  web::Json(item): web::Json<ForgotInput>,
  mailer: web::Data<mail::Mailer>
) -> Result<HttpResponse, AWError> {
  let db = pool.get().unwrap();

  let user_result = User::find_by_email(&db, item.email.clone());

  if user_result.is_ok() {
    let user = user_result.unwrap();

    // if !user.activated {
    //   return Ok(HttpResponse::build(StatusCode::BAD_REQUEST).body("{ \"message\": \"Account has not been activated\" }"))
    // }
    
    let reset_token_claims = ResetTokenClaims {
      exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
      sub: user.id,
      token_type: "reset_token".to_string()
    };
  
    let reset_token = encode(
      &Header::default(),
      &reset_token_claims,
      &EncodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref())
    ).unwrap();
    
    let link = &format!("http://localhost:8080/reset?token={reset_token}", reset_token=reset_token);
    mail::auth_recover_existent_account::send(&mailer, &user.email, link);
  } else {
    let link = &format!("http://localhost:8080/register");
    mail::auth_recover_nonexistent_account::send(&mailer, &item.email, link);
  }
  
  Ok(HttpResponse::build(StatusCode::OK).body("{ \"message\": \"Please check your email.\" }"))
}

#[derive(Serialize, Deserialize)]
struct ChangeInput {
  old_password: String,
  new_password: String
}

#[post("/change")]
async fn change_password(
  pool: web::Data<Pool>,
  web::Json(item): web::Json<ChangeInput>,
  auth: Auth,
  mailer: web::Data<mail::Mailer>
) -> Result<HttpResponse, AWError> {
  if item.old_password.len() == 0 || item.new_password.len() == 0 {
    return Ok(HttpResponse::build(StatusCode::BAD_REQUEST).body(json!({
      "message": "Missing password"
    })))
  }

  if item.old_password.eq(&item.new_password) {
    return Ok(HttpResponse::build(StatusCode::BAD_REQUEST).body(json!({
      "message": "The new password must be different"
    })))
  }
  
  let db = pool.get().unwrap();

  let user = User::read(&db, auth.user_id);

  if user.is_err() {
    return Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body(json!({
      "message": "Could not find user"
    })))
  }

  let user = user.unwrap();

  if !user.activated {
    return Ok(HttpResponse::build(StatusCode::BAD_REQUEST).body("{ \"message\": \"Account has not been activated\" }"))
  }

  let mut verifier = argonautica::Verifier::default();
  let is_old_password_valid = verifier
    .with_hash(&user.hash_password)
    .with_password(&item.old_password)
    .with_secret_key(match std::env::var("SECRET_KEY") {
      Ok(s) => s,
      Err(_) => panic!("No SECRET_KEY environment variable set!"),
    })
    .verify()
    .unwrap();
  
  if !is_old_password_valid {
    return Ok(HttpResponse::build(StatusCode::BAD_REQUEST).body(json!({
      "message": "Invalid credentials"
    })));
  }
  
  let mut hasher = argonautica::Hasher::default();
  let new_hash = hasher
      .with_password(&item.new_password)
      .with_secret_key(match std::env::var("SECRET_KEY") {
        Ok(s) => s,
        Err(_) => panic!("No SECRET_KEY environment variable set!"),
      })
      .hash()
      .unwrap();

  let updated_user = User::update(&db, auth.user_id, &UserChangeset {
    email: user.email.clone(),
    hash_password: new_hash,
    activated: user.activated
  });

  if updated_user.is_err() {
    return Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body(json!({
      "message": "Could not update password"
    })))
  }
  
  mail::auth_password_changed::send(&mailer, &user.email);

  Ok(HttpResponse::build(StatusCode::OK).body(json!({
    "message": "Password changed"
  })))
}

#[derive(Serialize, Deserialize)]
struct ResetInput {
  reset_token: String,
  new_password: String
}

#[post("/reset")]
async fn reset_password(
  pool: web::Data<Pool>,
  web::Json(item): web::Json<ResetInput>,
  mailer: web::Data<mail::Mailer>
) -> Result<HttpResponse, AWError> {
  let db = pool.get().unwrap();

  if item.new_password.len() == 0 {
    return Ok(HttpResponse::build(StatusCode::BAD_REQUEST).body(json!({
      "message": "Missing password"
    })))
  }
  
  let token = decode::<ResetTokenClaims>(
    &item.reset_token,
    &DecodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref()),
    &Validation::default()
  );

  if token.is_err() {
    return Ok(HttpResponse::build(StatusCode::UNAUTHORIZED).body("{ \"message\": \"Invalid token.\" }"));
  }

  let token = token.unwrap();

  if !token.claims.token_type.eq_ignore_ascii_case("reset_token") {
    return Ok(HttpResponse::build(StatusCode::UNAUTHORIZED).body("{ \"message\": \"Invalid token.\" }"));
  }

  let user = User::read(&db, token.claims.sub);

  if user.is_err() {
    return Ok(HttpResponse::build(StatusCode::BAD_REQUEST).body("{ \"message\": \"Invalid token.\" }"));
  }
  
  let user = user.unwrap();

  if !user.activated {
    return Ok(HttpResponse::build(StatusCode::BAD_REQUEST).body("{ \"message\": \"Account has not been activated\" }"))
  }
  
  let mut hasher = argonautica::Hasher::default();
  let new_hash = hasher
      .with_password(&item.new_password)
      .with_secret_key(match std::env::var("SECRET_KEY") {
        Ok(s) => s,
        Err(_) => panic!("No SECRET_KEY environment variable set!"),
      })
      .hash()
      .unwrap();

  let update = User::update(&db, user.id, &UserChangeset {
    email: user.email.clone(),
    hash_password: new_hash,
    activated: user.activated
  });

  if update.is_err() {
    return Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body("{ \"message\": \"Could not update password\" }"))
  }
  
  mail::auth_password_reset::send(&mailer, &user.email);
  
  Ok(HttpResponse::build(StatusCode::OK).body(json!({
    "message": "Password reset"
  })))
}

pub fn endpoints(scope: actix_web::Scope) -> actix_web::Scope {
  return scope
    .service(sessions)
    .service(destroy_session)
    .service(destroy_sessions)
    .service(login)
    .service(logout)
    .service(refresh)
    .service(register)
    .service(activate)
    .service(forgot_password)
    .service(change_password)
    .service(reset_password);
}
