use poem::{
    delete, get, handler,
    http::StatusCode,
    post,
    web::{
        cookie::{CookieJar, SameSite},
        Data, Json, Path, Query,
    },
    Error, IntoResponse, Request, Response, Result, Route,
};

use crate::{Database, Mailer};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use chrono::{DateTime, Utc};
use poem::web::cookie::Cookie;
use crate::auth::Role;
use super::{mail, Permission, user::{User, UserChangeset}, user_session::{UserSession, UserSessionChangeset}, Auth, PaginationParams, ID, UTC, AccessTokenClaims, UserSessionResponse, UserSessionJson};

const COOKIE_NAME: &'static str = "request_token";

type Seconds = i64;

#[derive(Deserialize, Serialize)]
struct LoginInput {
    email: String,
    password: String,
    device: Option<String>,
    ttl: Option<Seconds>
}

#[derive(Debug, Serialize, Deserialize)]
struct RefreshTokenClaims {
    exp: usize,
    sub: ID,
    token_type: String,
}

#[derive(Serialize, Deserialize)]
struct RegisterInput {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RegistrationClaims {
    exp: usize,
    sub: ID,
    token_type: String,
}

#[derive(Serialize, Deserialize)]
struct ActivationInput {
    activation_token: String,
}

#[derive(Serialize, Deserialize)]
struct ForgotInput {
    email: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResetTokenClaims {
    exp: usize,
    sub: ID,
    token_type: String,
}

#[derive(Serialize, Deserialize)]
struct ChangeInput {
    old_password: String,
    new_password: String,
}

#[derive(Serialize, Deserialize)]
struct ResetInput {
    reset_token: String,
    new_password: String,
}

#[handler]
async fn sessions(
    db: Data<&Database>,
    auth: Auth,
    Query(info): Query<PaginationParams>,
) -> Result<impl IntoResponse> {
    let db = db.pool.get().unwrap();

    let sessions = UserSession::read_all(&db, &info, auth.user_id);

    if sessions.is_err() {
        return Err(Error::from_string(
            "Could not fetch sessions.",
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    let sessions: std::vec::Vec<UserSession> = sessions.unwrap();
    let mut sessions_json: Vec<UserSessionJson> = vec![];

    for session in sessions {
        let session_json = UserSessionJson {
            id: session.id,
            device: session.device,
            created_at: session.created_at,
            updated_at: session.updated_at,
        };

        sessions_json.push(session_json);
    }

    let num_sessions = UserSession::count_all(&db, auth.user_id);
    if num_sessions.is_err() {
        return Err(Error::from_string(
            "Could not fetch sessions.",
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    let num_sessions = num_sessions.unwrap();
    let num_pages = (num_sessions / info.page_size)
        + (if num_sessions % info.page_size != 0 {
        1
    } else {
        0
    });

    let resp = UserSessionResponse {
        sessions: sessions_json,
        num_pages,
    };

    Ok(Json(resp))
}

#[handler]
async fn destroy_sessions(db: Data<&Database>, auth: Auth) -> Result<impl IntoResponse> {
    let db = db.pool.get().unwrap();

    if UserSession::delete_all_for_user(&db, auth.user_id).is_err() {
        return Err(Error::from_string(
            "Could not delete sessions.",
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    Ok(Response::builder().status(StatusCode::OK).finish())
}

#[handler]
async fn destroy_session(
    db: Data<&Database>,
    Path(item_id): Path<ID>,
    auth: Auth,
) -> Result<impl IntoResponse> {
    let db = db.pool.get().unwrap();

    let user_session = UserSession::read(&db, item_id);

    if user_session.is_err() {
        return Err(Error::from_string(
            "Internal error.",
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    let user_session = user_session.unwrap();

    if user_session.user_id != auth.user_id {
        return Err(Error::from_string(
            "Session not found.",
            StatusCode::NOT_FOUND,
        ));
    }

    if UserSession::delete(&db, user_session.id).is_err() {
        return Err(Error::from_string(
            "Could not delete session.",
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    Ok(Response::builder().status(StatusCode::OK).finish())
}

#[handler]
async fn login(
    db: Data<&Database>,
    Json(item): Json<LoginInput>,
    cookie_jar: &CookieJar,
) -> Result<impl IntoResponse> {
    let db = db.pool.get().unwrap();

    // verify device
    let mut device = None;
    if item.device.is_some() {
        let device_string = item.device.unwrap();
        if device_string.len() > 256 {
            return Err(Error::from_string(
                "'device' cannot be longer than 256 characters.",
                StatusCode::BAD_REQUEST,
            ));
        }
    }

    let user = User::find_by_email(&db, item.email);

    if user.is_err() {
        return Err(Error::from_string(
            "Invalid credentials.",
            StatusCode::UNAUTHORIZED,
        ));
    }

    let user = user.unwrap();

    if !user.activated {
        return Err(Error::from_string(
            "Account has not been activated.",
            StatusCode::BAD_REQUEST,
        ));
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
        return Err(Error::from_string(
            "Invalid credentials.",
            StatusCode::UNAUTHORIZED,
        ));
    }

    let permissions = Permission::fetch_all(&db, user.id);
    if permissions.is_err() {
        println!("{:#?}", permissions.err());
        return Err(Error::from_string(
            "An internal server error occurred.",
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }
    let permissions = permissions.unwrap();

    let roles = Role::fetch_all(&db, user.id);
    if roles.is_err() {
        println!("{:#?}", roles.err());
        return Err(Error::from_string(
            "An internal server error occurred.",
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }
    let roles = roles.unwrap();

    let access_token_duration = chrono::Duration::seconds(
        if item.ttl.is_some() {
            std::cmp::max(item.ttl.unwrap(), 1)
        } else {
            /* 15 minutes */
            15 * 60
        }
    );

    let access_token_claims = AccessTokenClaims {
        exp: (chrono::Utc::now() + access_token_duration).timestamp() as usize,
        sub: user.id,
        token_type: "access_token".to_string(),
        roles,
        permissions,
    };

    let refresh_token_claims = RefreshTokenClaims {
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
        sub: user.id,
        token_type: "refresh_token".to_string(),
    };

    let access_token = encode(
        &Header::default(),
        &access_token_claims,
        &EncodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref()),
    )
        .unwrap();

    let refresh_token = encode(
        &Header::default(),
        &refresh_token_claims,
        &EncodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref()),
    )
        .unwrap();

    let user_session = UserSession::create(
        &db,
        &UserSessionChangeset {
            user_id: user.id,
            refresh_token: refresh_token.clone(),
            device,
        },
    );

    if user_session.is_err() {
        return Err(Error::from_string(
            "Could not create a session.",
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    let mut cookie = poem::web::cookie::Cookie::new(COOKIE_NAME, refresh_token.clone());
    cookie.set_secure(true);
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Strict);
    cookie_jar.add(cookie);

    let json = serde_json::json!({ "access_token": access_token }).to_string();
    let response = Response::builder().status(StatusCode::OK).body(json);

    Ok(response)
}

#[handler]
async fn logout(db: Data<&Database>, req: &Request) -> Result<impl IntoResponse> {
    let db = db.pool.get().unwrap();
    let jar = req.cookie();

    let refresh_token_cookie = jar.get(COOKIE_NAME);

    if refresh_token_cookie.is_none() {
        return Err(Error::from_string(
            "Invalid session.",
            StatusCode::UNAUTHORIZED,
        ));
    }

    let refresh_token_cookie_unwrapped = refresh_token_cookie.clone().unwrap();
    let refresh_token_str = refresh_token_cookie_unwrapped.value();

    if refresh_token_str.is_err() {
        return Err(Error::from_string(
            "Invalid session cookie",
            StatusCode::UNAUTHORIZED,
        ));
    }

    let refresh_token_str = refresh_token_str.unwrap();

    let session = UserSession::find_by_refresh_token(&db, refresh_token_str);

    if session.is_err() {
        return Err(Error::from_string(
            "Invalid session.",
            StatusCode::UNAUTHORIZED,
        ));
    }

    let session = session.unwrap();

    let is_deleted = UserSession::delete(&db, session.id);

    if is_deleted.is_err() {
        return Err(Error::from_string(
            "Could not delete session.",
            StatusCode::UNAUTHORIZED,
        ));
    }

    let mut builder = Response::builder().status(StatusCode::OK);

    if let Some(ref cookie) = refresh_token_cookie {
        let mut removal_cookie = cookie.clone();
        removal_cookie.make_removal();
        jar.add(removal_cookie);
    }

    Ok(builder.finish())
}

#[handler]
async fn refresh(db: Data<&Database>, req: &Request) -> Result<impl IntoResponse> {
    let db = db.pool.get().unwrap();

    let jar = req.cookie();
    let cookie = jar.get(COOKIE_NAME);

    if cookie.is_none() {
        return Ok(Response::builder().status(StatusCode::UNAUTHORIZED).body(
            json!({
              "message": "Invalid session."
            })
                .to_string(),
        ));
    }

    let cookie = cookie.unwrap();

    let refresh_token_str = cookie.value::<&str>();

    if refresh_token_str.is_err() {
        return Err(Error::from_string(
            "Invalid session cookie.",
            StatusCode::UNAUTHORIZED,
        ));
    }

    let refresh_token_str = refresh_token_str.unwrap();

    let refresh_token = decode::<RefreshTokenClaims>(
        &refresh_token_str,
        &DecodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref()),
        &Validation::default(),
    );

    if refresh_token.is_err() {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body("{ \"message\": \"Invalid token.\" }"));
    }

    let refresh_token = refresh_token.unwrap();

    if !refresh_token
        .claims
        .token_type
        .eq_ignore_ascii_case("refresh_token")
    {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body("{ \"message\": \"Invalid token.\" }"));
    }

    let session = UserSession::find_by_refresh_token(&db, &refresh_token_str);

    if session.is_err() {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body("{ \"message\": \"Invalid session.\" }"));
    }

    let session = session.unwrap();

    let permissions = Permission::fetch_all(&db, session.user_id);
    if permissions.is_err() {
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("{ \"message\": \"An internal server error occurred.\" }"));
    }
    let permissions = permissions.unwrap();

    let roles = Role::fetch_all(&db, session.user_id);
    if roles.is_err() {
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("{ \"message\": \"An internal server error occurred.\" }"));
    }
    let roles = roles.unwrap();

    let access_token_claims = AccessTokenClaims {
        exp: (chrono::Utc::now() + chrono::Duration::minutes(15)).timestamp() as usize,
        sub: session.user_id,
        token_type: "access_token".to_string(),
        roles,
        permissions,
    };

    let refresh_token_claims = RefreshTokenClaims {
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
        sub: session.user_id,
        token_type: "refresh_token".to_string(),
    };

    let access_token = encode(
        &Header::default(),
        &access_token_claims,
        &EncodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref()),
    )
        .unwrap();

    let refresh_token_str = encode(
        &Header::default(),
        &refresh_token_claims,
        &EncodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref()),
    )
        .unwrap();

    // update session with the new refresh token
    let session_update = UserSession::update(
        &db,
        session.id,
        &UserSessionChangeset {
            user_id: session.user_id,
            refresh_token: refresh_token_str.clone(),
            device: session.device,
        },
    );

    if session_update.is_err() {
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("{ \"message\": \"Could not update the session.\" }"));
    }

    let mut cookie = Cookie::new(COOKIE_NAME, refresh_token_str);
    cookie.set_secure(true);
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Strict);
    jar.add(cookie);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(json!({ "access_token": access_token }).to_string()))
}

#[handler]
async fn register(
    db: Data<&Database>,
    Json(item): Json<RegisterInput>,
    mailer: Data<&Mailer>,
) -> Result<impl IntoResponse> {
    let db = db.pool.get().unwrap();

    let user = User::find_by_email(&db, (&item.email).to_string());

    if user.is_ok() {
        let user = user.unwrap();
        if !user.activated {
            User::delete(&db, user.id).unwrap();
        } else {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("{ \"message\": \"Already registered.\" }"));
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

    let user = User::create(
        &db,
        &UserChangeset {
            activated: false,
            email: item.email,
            hash_password: hash,
        },
    )
        .unwrap();

    let registration_claims = RegistrationClaims {
        exp: (chrono::Utc::now() + chrono::Duration::days(30)).timestamp() as usize,
        sub: user.id,
        token_type: "activation_token".to_string(),
    };

    let token = encode(
        &Header::default(),
        &registration_claims,
        &EncodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref()),
    )
        .unwrap();

    mail::auth_register::send(
        &mailer,
        &user.email,
        &format!(
            "http://localhost:8080/activate?token={token}",
            token = token
        ),
    );

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body("{ \"message\": \"Registered! Check your email to activate your account.\" }"))
}

#[handler]
async fn activate(
    db: Data<&Database>,
    Query(item): Query<ActivationInput>,
    mailer: Data<&Mailer>,
) -> Result<impl IntoResponse> {
    let db = db.pool.get().unwrap();

    let token = decode::<RegistrationClaims>(
        &item.activation_token,
        &DecodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref()),
        &Validation::default(),
    );

    if token.is_err() {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body("{ \"message\": \"Invalid token.\" }"));
    }

    let token = token.unwrap();

    if !token
        .claims
        .token_type
        .eq_ignore_ascii_case("activation_token")
    {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body("{ \"message\": \"Invalid token.\" }"));
    }

    let user = User::read(&db, token.claims.sub);

    if user.is_err() {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("{ \"message\": \"Invalid token.\" }"));
    }

    let user = user.unwrap();

    if user.activated {
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .body("{ \"message\": \"Already activated!\" }"));
    }

    let activated_user = User::update(
        &db,
        user.id,
        &UserChangeset {
            activated: true,
            email: user.email.clone(),
            hash_password: user.hash_password,
        },
    );

    if activated_user.is_err() {
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("{ \"message\": \"Could not activate user.\" }"));
    }

    mail::auth_activated::send(&mailer, &user.email);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body("{ \"message\": \"Activated!\" }"))
}

#[handler]
async fn forgot_password(
    db: Data<&Database>,
    Json(item): Json<ForgotInput>,
    mailer: Data<&Mailer>,
) -> Result<impl IntoResponse> {
    let db = db.pool.get().unwrap();

    let user_result = User::find_by_email(&db, item.email.clone());

    if user_result.is_ok() {
        let user = user_result.unwrap();

        // if !user.activated {
        //   return Ok(Response::builder().status(StatusCode::BAD_REQUEST).body("{ \"message\": \"Account has not been activated\" }"))
        // }

        let reset_token_claims = ResetTokenClaims {
            exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
            sub: user.id,
            token_type: "reset_token".to_string(),
        };

        let reset_token = encode(
            &Header::default(),
            &reset_token_claims,
            &EncodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref()),
        )
            .unwrap();

        let link = &format!(
            "http://localhost:8080/reset?token={reset_token}",
            reset_token = reset_token
        );
        mail::auth_recover_existent_account::send(&mailer, &user.email, link);
    } else {
        let link = &format!("http://localhost:8080/register");
        mail::auth_recover_nonexistent_account::send(&mailer, &item.email, link);
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body("{ \"message\": \"Please check your email.\" }"))
}

#[handler]
async fn change_password(
    db: Data<&Database>,
    Json(item): Json<ChangeInput>,
    auth: Auth,
    mailer: Data<&Mailer>,
) -> Result<impl IntoResponse> {
    if item.old_password.len() == 0 || item.new_password.len() == 0 {
        return Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(
            json!({
              "message": "Missing password"
            })
                .to_string(),
        ));
    }

    if item.old_password.eq(&item.new_password) {
        return Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(
            json!({
              "message": "The new password must be different"
            })
                .to_string(),
        ));
    }

    let db = db.pool.get().unwrap();

    let user = User::read(&db, auth.user_id);

    if user.is_err() {
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(
                json!({
                  "message": "Could not find user"
                })
                    .to_string(),
            ));
    }

    let user = user.unwrap();

    if !user.activated {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("{ \"message\": \"Account has not been activated\" }"));
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
        return Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(
            json!({
              "message": "Invalid credentials"
            })
                .to_string(),
        ));
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

    let updated_user = User::update(
        &db,
        auth.user_id,
        &UserChangeset {
            email: user.email.clone(),
            hash_password: new_hash,
            activated: user.activated,
        },
    );

    if updated_user.is_err() {
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(
                json!({
                  "message": "Could not update password"
                })
                    .to_string(),
            ));
    }

    mail::auth_password_changed::send(&mailer, &user.email);

    Ok(Response::builder().status(StatusCode::OK).body(
        json!({
          "message": "Password changed"
        })
            .to_string(),
    ))
}

#[handler]
async fn reset_password(
    db: Data<&Database>,
    Json(item): Json<ResetInput>,
    mailer: Data<&Mailer>,
) -> Result<impl IntoResponse> {
    let db = db.pool.get().unwrap();

    if item.new_password.len() == 0 {
        return Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(
            json!({
              "message": "Missing password"
            })
                .to_string(),
        ));
    }

    let token = decode::<ResetTokenClaims>(
        &item.reset_token,
        &DecodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref()),
        &Validation::default(),
    );

    if token.is_err() {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body("{ \"message\": \"Invalid token.\" }"));
    }

    let token = token.unwrap();

    if !token.claims.token_type.eq_ignore_ascii_case("reset_token") {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body("{ \"message\": \"Invalid token.\" }"));
    }

    let user = User::read(&db, token.claims.sub);

    if user.is_err() {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("{ \"message\": \"Invalid token.\" }"));
    }

    let user = user.unwrap();

    if !user.activated {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("{ \"message\": \"Account has not been activated\" }"));
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

    let update = User::update(
        &db,
        user.id,
        &UserChangeset {
            email: user.email.clone(),
            hash_password: new_hash,
            activated: user.activated,
        },
    );

    if update.is_err() {
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("{ \"message\": \"Could not update password\" }"));
    }

    mail::auth_password_reset::send(&mailer, &user.email);

    Ok(Response::builder().status(StatusCode::OK).body(
        json!({
          "message": "Password reset"
        })
            .to_string(),
    ))
}

#[handler]
async fn check(auth: Auth) -> Result<impl IntoResponse> {
    Ok(Response::builder().status(StatusCode::OK).finish())
}

pub fn api() -> Route {
    Route::new()
        .at("/sessions", get(sessions).delete(destroy_sessions))
        .at("/sessions/:id", delete(destroy_session))
        .at("/login", post(login))
        .at("/logout", post(logout))
        .at("/check", post(check))
        .at("/refresh", post(refresh))
        .at("/register", post(register))
        .at("/activate", get(activate))
        .at("/forgot", post(forgot_password))
        .at("/change", post(change_password))
        .at("/reset", post(reset_password))
}
