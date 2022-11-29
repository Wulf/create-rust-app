use poem::{
    delete, get, handler,
    http::StatusCode,
    post,
    web::{
        cookie::{Cookie, CookieJar, SameSite},
        Data, Json, Path, Query,
    },
    Error, IntoResponse, Response, Result, Route,
};
use serde_json::json;

use crate::auth::controller::{
    ActivationInput, ChangeInput, ForgotInput, LoginInput, RegisterInput, ResetInput, COOKIE_NAME,
};
use crate::auth::{controller, Auth, PaginationParams, ID};
use crate::{Database, Mailer};

fn error_response(status_code: i32, message: &'static str) -> Error {
    Error::from_string(
        json!({ "message": message }).to_string(),
        StatusCode::from_u16(status_code as u16).unwrap(),
    )
}

#[handler]
/// handler for GET requests at the .../sessions endpoint,
///
/// requires auth
///
/// request should be a query that contains [`PaginationParams`]
///
/// see [`controller::get_sessions`]
///
/// # Responses
/// | StatusCode | content |
/// |:------------|---------|
/// | 200 | [`UserSessionResponse`](`crate::auth::UserSessionResponse`) deserialized into a Json payload
/// | 500 | Json payload : {"message": "Could not fetch sessions."}
/// TODO: document the rest of the possible StatusCodes
async fn sessions(
    db: Data<&Database>,
    auth: Auth,
    Query(info): Query<PaginationParams>,
) -> Result<impl IntoResponse> {
    let result = controller::get_sessions(db.0, &auth, &info);

    match result {
        Ok(sessions) => Ok(Json(sessions)),
        Err((status_code, message)) => Err(error_response(status_code, message)),
    }
}

#[handler]
/// handler for DELETE requests at the .../sessions enpoint
///
/// requires auth
///
/// deletes all current sessions belonging to the user
///
/// see [`controller::destroy_sessions`]
///
/// # Responses
/// | StatusCode | content |
/// |:------------|---------|
/// | 200 | Json payload : {"message": "Deleted."}
/// | 500 | Json payload : {"message": "Could not delete sessions."}
/// TODO: document the rest of the possible StatusCodes
async fn destroy_sessions(db: Data<&Database>, auth: Auth) -> Result<impl IntoResponse> {
    let result = controller::destroy_sessions(db.0, &auth);

    match result {
        Ok(_) => Ok(Response::builder().status(StatusCode::OK).finish()),
        Err((s, m)) => Err(error_response(s, m)),
    }
}

#[handler]
/// handler for DELETE requests at the .../sessions/{id} endpoint.
///
/// requires auth
///
/// see [`controller::destroy_session`]
///
///
/// # Responses
/// | StatusCode | content |
/// |:------------|---------|
/// | 200 | Json payload : {"message": "Deleted."}
/// | 404 | Json payload : {"message": "Session not found."}
/// | 500 | Json payload : {"message": "Internal error."}
/// | 500 | Json payload : {"message": "Could not delete session.`}
/// TODO: document the rest of the possible StatusCodes
async fn destroy_session(
    db: Data<&Database>,
    Path(item_id): Path<ID>,
    auth: Auth,
) -> Result<impl IntoResponse> {
    let result = controller::destroy_session(db.0, &auth, item_id);

    match result {
        Ok(_) => Ok(Response::builder().status(StatusCode::OK).finish()),
        Err((s, m)) => Err(error_response(s, m)),
    }
}

#[handler]
/// handler for POST requests at the .../login endpoint
///
/// request must have the `Content-Type: application/json` header, and a Json payload that can be deserialized into [`LoginInput`]
///
/// see [`controller::login`]
///
/// # Responses
/// | StatusCode | content |
/// |:------------|---------|
/// | 200 | Json payload with an "assess_token" field containing a JWT associated with the user
/// | 400 | Json payload : {"message": "'device' cannot be longer than 256 characters."}
/// | 400 | Json payload : {"message": "Account has not been activated."}
/// | 401 | Json payload : {"message": "Invalid credentials."}
/// | 500 | Json payload : {"message": "An internal server error occurred."}
/// | 500 | Json payload : {"message": "Could not create a session."}
/// TODO: document the rest of the possible StatusCodes
async fn login(
    db: Data<&Database>,
    Json(item): Json<LoginInput>,
    cookie_jar: &CookieJar,
) -> Result<impl IntoResponse> {
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
        Err((s, m)) => Err(error_response(s, m)),
    }
}

#[handler]
/// handler for POST requests to the .../logout endpount
///
/// see [`controller::logout']
///
/// # Responses
/// | StatusCode | content |
/// |:------------|---------|
/// | 200 | command to delete the "refresh_token" cookie
/// | 401 | Json payload : {"message": "Invalid session."}
/// | 401 | Json payload : {"message": "Could not delete session."}
/// TODO: document the rest of the possible StatusCodes
async fn logout(db: Data<&Database>, cookie_jar: &CookieJar) -> Result<impl IntoResponse> {
    let refresh_token = cookie_jar
        .get(COOKIE_NAME)
        .map(|cookie| String::from(cookie.value_str()));

    let result = controller::logout(db.0, refresh_token.as_ref().map(|t| t.as_str()));

    match result {
        Ok(_) => {
            let mut cookie = Cookie::named(COOKIE_NAME);
            cookie.make_removal();

            cookie_jar.add(cookie);

            Ok(Response::builder().status(StatusCode::OK).finish())
        }
        Err((s, m)) => Err(error_response(s, m)),
    }
}

#[handler]
/// handler for POST requests to the .../refresh endpoint
///
/// see [`controller::refresh`]
///
/// # Responses
/// | StatusCode | content |
/// |:------------|---------|
/// | 200 | Json payload with an "assess_token" field containing a JWT associated with the user
/// | 401 | Json payload : {"message": "Invalid session."}
/// | 401 | Json payload : {"message": "Invalid token."}
/// TODO: document the rest of the possible StatusCodes
async fn refresh(db: Data<&Database>, cookie_jar: &CookieJar) -> Result<impl IntoResponse> {
    let refresh_token = cookie_jar
        .get(COOKIE_NAME)
        .map(|cookie| String::from(cookie.value_str()));

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
        Err((s, m)) => Err(error_response(s, m)),
    }
}

#[handler]
/// handler for POST requests to the .../register endpoint
///
/// request must have the `Content-Type: application/json` header, and a Json payload that can be deserialized into [`RegisterInput`]
///
/// see [`controller::register`]
///
/// # Responses
/// | StatusCode | content |
/// |:------------|---------|
/// | 200 | Json payload : {"message": "Registered! Check your email to activate your account."}
/// | 400 | Json payload : {"message": "Already registered."}
/// TODO: document the rest of the possible StatusCodes
async fn register(
    db: Data<&Database>,
    Json(item): Json<RegisterInput>,
    mailer: Data<&Mailer>,
) -> Result<impl IntoResponse> {
    let result = controller::register(db.0, &item, mailer.0);

    match result {
        Ok(_) => Ok(Response::builder()
            .status(StatusCode::OK)
            .body("{ \"message\": \"Registered! Check your email to activate your account.\" }")),
        Err((s, m)) => Err(error_response(s, m)),
    }
}

#[handler]
/// handler for GET requests to the .../activate endpoint
///
/// see [`controller:: activate`]
///
/// # Responses
/// | StatusCode | content |
/// |:------------|---------|
/// | 200 | Json payload : {"message": "Activated."}
/// | 200 | Json payload : {"message": "Already activated."}
/// | 400 | Json payload : {"message": "Invalid token."}
/// | 401 | Json payload : {"message": "Invalid token"}
/// | 500 | Json payload : {"message": "Could not activate user. "}
/// TODO: document the rest of the possible StatusCodes
async fn activate(
    db: Data<&Database>,
    Query(item): Query<ActivationInput>,
    mailer: Data<&Mailer>,
) -> Result<impl IntoResponse> {
    let result = controller::activate(db.0, &item, mailer.0);

    match result {
        Ok(_) => Ok(Response::builder()
            .status(StatusCode::OK)
            .body("{ \"message\": \"Activated!\" }")),

        Err((s, m)) => Err(error_response(s, m)),
    }
}

#[handler]
/// handler for POST requests to the .../forgot endpoint
///
/// request must have the `Content-Type: application/json` header, and a Json payload that can be deserialized into [`ForgotInput`]
///
/// see [`controller::forgot_password`]
///
/// # Responses
/// | StatusCode | content |
/// |:------------|---------|
/// | 200 | Json payload : {"message": "Please check your email."}
/// TODO: document the rest of the possible StatusCodes
async fn forgot_password(
    db: Data<&Database>,
    Json(item): Json<ForgotInput>,
    mailer: Data<&Mailer>,
) -> Result<impl IntoResponse> {
    let result = controller::forgot_password(db.0, &item, mailer.0);

    match result {
        Ok(_) => Ok(Response::builder()
            .status(StatusCode::OK)
            .body("{ \"message\": \"Please check your email.\" }")),
        Err((s, m)) => Err(error_response(s, m)),
    }
}

#[handler]
/// handler for POST requests to the .../change endpoint
///
/// requires auth
///
/// request must have the `Content-Type: application/json` header, and a Json payload that can be deserialized into [`ChangeInput`]
///
/// see [`controller::change_password`]
///
/// # Responses
/// | StatusCode | content |
/// |:------------|---------|
/// | 200 | Json payload : {"message": "Password changed."}
/// | 400 | Json payload : {"message": "Missing password."}
/// | 400 | Json payload : {"message": "The new password must be different."}
/// | 400 | Json payload : {"message": "Account has not been activated."}
/// | 400 | Json payload : {"message": "Invalid credentials."}
/// | 500 | Json payload : {"message": "Could not find user."}
/// | 500 | Json payload : {"message": "Could not update password."}
/// TODO: document the rest of the possible StatusCodes
async fn change_password(
    db: Data<&Database>,
    Json(item): Json<ChangeInput>,
    auth: Auth,
    mailer: Data<&Mailer>,
) -> Result<impl IntoResponse> {
    let result = controller::change_password(db.0, &item, &auth, mailer.0);

    match result {
        Ok(_) => Ok(Response::builder()
            .status(StatusCode::OK)
            .body(json!({"message": "Password changed"}).to_string())),
        Err((s, m)) => Err(error_response(s, m)),
    }
}

#[handler]
/// handler for POST requests to the .../reset endpoint
///
/// request must have the `Content-Type: application/json` header, and a Json payload that can be deserialized into [`ResetInput`]
///
/// see [`controller::reset_password`]
///
/// # Responses
/// | StatusCode | content |
/// |:------------|---------|
/// | 200 | Json payload : {"message": "Password changed."}
/// | 400 | Json payload : {"message": "Invalid token."}
/// | 400 | Json payload : {"message": "Account has not been activated."}
/// | 400 | Json payload : {"message": "The new password must be different."}
/// | 401 | Json payload : {"message": "Invalid token."}
/// | 500 | Json payload : {"message": "Could not update password."}
async fn reset_password(
    db: Data<&Database>,
    Json(item): Json<ResetInput>,
    mailer: Data<&Mailer>,
) -> Result<impl IntoResponse> {
    let result = controller::reset_password(db.0, &item, mailer.0);

    match result {
        Ok(_) => Ok(Response::builder()
            .status(StatusCode::OK)
            .body(json!({"message": "Password reset"}).to_string())),
        Err((s, m)) => Err(error_response(s, m)),
    }
}

#[handler]
/// handler for POST requests to the .../check endpoint
///
/// requires auth, but doesn't match it to a user
///
/// see [`controller::check`]
///
/// # Responses
/// | StatusCode | content |
/// |:------------|---------|
/// | 200 | ()
async fn check(auth: Auth) -> Result<impl IntoResponse> {
    controller::check(&auth);
    Ok(Response::builder().status(StatusCode::OK).finish())
}

/// returns endpoints for the Auth service
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
