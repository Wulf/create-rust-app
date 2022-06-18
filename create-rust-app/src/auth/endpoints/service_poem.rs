use poem::{delete, Error, get, handler, http::StatusCode, IntoResponse, post, Response, Result, Route, web::{cookie::{Cookie, CookieJar, SameSite}, Data, Json, Path, Query}};
use serde_json::json;

use crate::{Database, Mailer};
use crate::auth::{Auth, controller, ID, PaginationParams};
use crate::auth::controller::{ActivationInput, ChangeInput, COOKIE_NAME, ForgotInput, LoginInput, RegisterInput, ResetInput};

fn error_response(status_code: i32, message: &'static str) -> Error {
    Error::from_string(json!({"message": message}).to_string(), StatusCode::from_u16(status_code as u16).unwrap())
}

#[handler]
async fn sessions(db: Data<&Database>, auth: Auth, Query(info): Query<PaginationParams>) -> Result<impl IntoResponse> {
    let result = controller::get_sessions(db.0, &auth, &info);

    match result {
        Ok(sessions) => Ok(Json(sessions)),
        Err((status_code, message)) => Err(error_response(status_code, message))
    }
}

#[handler]
async fn destroy_sessions(db: Data<&Database>, auth: Auth) -> Result<impl IntoResponse> {
    let result = controller::destroy_sessions(db.0, &auth);

    match result {
        Ok(_) => Ok(Response::builder().status(StatusCode::OK).finish()),
        Err((s, m)) => Err(error_response(s, m))
    }
}

#[handler]
async fn destroy_session(db: Data<&Database>, Path(item_id): Path<ID>, auth: Auth) -> Result<impl IntoResponse> {
    let result = controller::destroy_session(db.0, &auth, item_id);

    match result {
        Ok(_) => Ok(Response::builder().status(StatusCode::OK).finish()),
        Err((s, m)) => Err(error_response(s, m))
    }
}

#[handler]
async fn login(db: Data<&Database>, Json(item): Json<LoginInput>, cookie_jar: &CookieJar) -> Result<impl IntoResponse> {
    let result = controller::login(db.0, &item);

    match result {
        Ok((access_token, refresh_token)) => {
            let mut cookie = Cookie::new(COOKIE_NAME, refresh_token.clone());
            cookie.set_secure(true);
            cookie.set_http_only(true);
            cookie.set_same_site(SameSite::Strict);
            cookie_jar.add(cookie);

            let json = json!({ "access_token": access_token }).to_string();
            let response = Response::builder().status(StatusCode::OK).body(json);

            Ok(response)
        }
        Err((s, m)) => Err(error_response(s, m))
    }
}

#[handler]
async fn logout(db: Data<&Database>, cookie_jar: &CookieJar) -> Result<impl IntoResponse> {
    let refresh_token = cookie_jar.get(COOKIE_NAME).map(|cookie| String::from(cookie.value_str()));

    let result = controller::logout(db.0, refresh_token.as_ref().map(|t| t.as_str()));

    match result {
        Ok(_) => {
            let mut cookie = Cookie::named(COOKIE_NAME);
            cookie.make_removal();

            cookie_jar.add(cookie);

            Ok(Response::builder().status(StatusCode::OK).finish())
        }
        Err((s, m)) => Err(error_response(s, m))
    }
}

#[handler]
async fn refresh(db: Data<&Database>, cookie_jar: &CookieJar) -> Result<impl IntoResponse> {
    let refresh_token = cookie_jar.get(COOKIE_NAME).map(|cookie| String::from(cookie.value_str()));

    let result = controller::refresh(db.0, refresh_token.as_ref().map(|t| t.as_str()));

    match result {
        Ok((access_token, refresh_token)) => {
            let mut cookie = Cookie::new(COOKIE_NAME, refresh_token);
            cookie.set_secure(true);
            cookie.set_http_only(true);
            cookie.set_same_site(SameSite::Strict);
            cookie_jar.add(cookie);

            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(json!({ "access_token": access_token }).to_string()))
        }
        Err((s, m)) => Err(error_response(s, m))
    }
}

#[handler]
async fn register(db: Data<&Database>, Json(item): Json<RegisterInput>, mailer: Data<&Mailer>) -> Result<impl IntoResponse> {
    let result = controller::register(db.0, &item, mailer.0);

    match result {
        Ok(_) => Ok(Response::builder()
            .status(StatusCode::OK)
            .body("{ \"message\": \"Registered! Check your email to activate your account.\" }")),
        Err((s, m)) => Err(error_response(s, m))
    }
}

#[handler]
async fn activate(db: Data<&Database>, Query(item): Query<ActivationInput>, mailer: Data<&Mailer>) -> Result<impl IntoResponse> {
    let result = controller::activate(db.0, &item, mailer.0);

    match result {
        Ok(_) => Ok(Response::builder()
            .status(StatusCode::OK)
            .body("{ \"message\": \"Activated!\" }")),

        Err((s, m)) => Err(error_response(s, m))
    }
}

#[handler]
async fn forgot_password(db: Data<&Database>, Json(item): Json<ForgotInput>, mailer: Data<&Mailer>) -> Result<impl IntoResponse> {
    let result = controller::forgot_password(db.0, &item, mailer.0);

    match result {
        Ok(_) => Ok(Response::builder()
            .status(StatusCode::OK)
            .body("{ \"message\": \"Please check your email.\" }")),
        Err((s, m)) => Err(error_response(s, m))
    }
}

#[handler]
async fn change_password(db: Data<&Database>, Json(item): Json<ChangeInput>, auth: Auth, mailer: Data<&Mailer>) -> Result<impl IntoResponse> {
    let result = controller::change_password(db.0, &item, &auth, mailer.0);

    match result {
        Ok(_) => Ok(Response::builder().status(StatusCode::OK).body(json!({"message": "Password changed"}).to_string())),
        Err((s, m)) => Err(error_response(s, m))
    }
}

#[handler]
async fn reset_password(db: Data<&Database>, Json(item): Json<ResetInput>, mailer: Data<&Mailer>) -> Result<impl IntoResponse> {
    let result = controller::reset_password(db.0, &item, mailer.0);

    match result {
        Ok(_) => Ok(Response::builder().status(StatusCode::OK).body(json!({"message": "Password reset"}).to_string())),
        Err((s, m)) => Err(error_response(s, m))
    }
}

#[handler]
async fn check(auth: Auth) -> Result<impl IntoResponse> {
    controller::check(&auth);
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
