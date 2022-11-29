use actix_http::StatusCode;
use actix_web::cookie::{Cookie, SameSite};
use actix_web::{delete, get, post, web, Error as AWError, Result};
use actix_web::{
    web::{Data, Json, Path, Query},
    HttpRequest, HttpResponse,
};
use serde_json::json;

use crate::auth::{
    controller,
    controller::{
        ActivationInput, ChangeInput, ForgotInput, LoginInput, RegisterInput, ResetInput,
        COOKIE_NAME,
    },
    Auth, PaginationParams, ID,
};
use crate::Database;
use crate::Mailer;

#[get("/sessions")]
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
    db: Data<Database>,
    auth: Auth,
    Query(info): Query<PaginationParams>,
) -> Result<HttpResponse> {
    let result =
        web::block(move || controller::get_sessions(db.into_inner().as_ref(), &auth, &info))
            .await?;

    match result {
        Ok(sessions) => Ok(HttpResponse::Ok().json(sessions)),
        Err((status_code, error_message)) => Ok(HttpResponse::build(
            StatusCode::from_u16(status_code as u16).unwrap(),
        )
        .body(json!({ "message": error_message }).to_string())),
    }
}

#[delete("/sessions/{id}")]
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
    db: Data<Database>,
    item_id: Path<ID>,
    auth: Auth,
) -> Result<HttpResponse> {
    let result =
        web::block(move || controller::destroy_session(&db, &auth, item_id.into_inner())).await?;

    match result {
        Ok(_) => Ok(
            HttpResponse::build(StatusCode::OK).body(json!({"message": "Deleted."}).to_string())
        ),
        Err((status_code, error_message)) => Ok(HttpResponse::build(
            StatusCode::from_u16(status_code as u16).unwrap(),
        )
        .body(json!({ "message": error_message }).to_string())),
    }
}

#[delete("/sessions")]
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
async fn destroy_sessions(db: Data<Database>, auth: Auth) -> Result<HttpResponse, AWError> {
    let result = web::block(move || controller::destroy_sessions(&db, &auth)).await?;

    match result {
        Ok(_) => Ok(
            HttpResponse::build(StatusCode::OK).body(json!({"message": "Deleted."}).to_string())
        ),
        Err((status_code, error_message)) => Ok(HttpResponse::build(
            StatusCode::from_u16(status_code as u16).unwrap(),
        )
        .body(json!({ "message": error_message }).to_string())),
    }
}

#[post("/login")]
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
async fn login(db: Data<Database>, Json(item): Json<LoginInput>) -> Result<HttpResponse, AWError> {
    let result = web::block(move || controller::login(&db, &item)).await?;

    match result {
        Ok((access_token, refresh_token)) => Ok(HttpResponse::build(StatusCode::OK)
            .cookie(
                Cookie::build(COOKIE_NAME, refresh_token.clone())
                    .secure(true)
                    .http_only(true)
                    .same_site(SameSite::Strict)
                    .finish(),
            )
            .body(json!({ "access_token": access_token }).to_string())),
        Err((status_code, message)) => Ok(HttpResponse::build(
            StatusCode::from_u16(status_code as u16).unwrap(),
        )
        .body(json!({ "message": message }).to_string())),
    }
}

#[post("/logout")]
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
async fn logout(db: Data<Database>, req: HttpRequest) -> Result<HttpResponse, AWError> {
    let refresh_token = req
        .cookie(COOKIE_NAME)
        .map(|cookie| String::from(cookie.value()));

    let result =
        web::block(move || controller::logout(&db, refresh_token.as_ref().map(|t| t.as_ref())))
            .await?;

    match result {
        Ok(_) => {
            let mut cookie = Cookie::named(COOKIE_NAME);
            cookie.make_removal();

            Ok(HttpResponse::Ok().cookie(cookie).finish())
        }
        Err((status_code, message)) => Ok(HttpResponse::build(
            StatusCode::from_u16(status_code as u16).unwrap(),
        )
        .body(json!({ "message": message }).to_string())),
    }
}

#[post("/refresh")]
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
async fn refresh(db: Data<Database>, req: HttpRequest) -> Result<HttpResponse, AWError> {
    let refresh_token = req
        .cookie(COOKIE_NAME)
        .map(|cookie| String::from(cookie.value()));

    let result =
        web::block(move || controller::refresh(&db, refresh_token.as_ref().map(|t| t.as_ref())))
            .await?;

    match result {
        Ok((access_token, refresh_token)) => Ok(HttpResponse::build(StatusCode::OK)
            .cookie(
                Cookie::build(COOKIE_NAME, refresh_token)
                    .secure(true)
                    .http_only(true)
                    .same_site(SameSite::Strict)
                    .finish(),
            )
            .body(json!({ "access_token": access_token }).to_string())),
        Err((status_code, message)) => Ok(HttpResponse::build(
            StatusCode::from_u16(status_code as u16).unwrap(),
        )
        .body(json!({ "message": message }).to_string())),
    }
}

#[post("/register")]
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
    db: Data<Database>,
    Json(item): Json<RegisterInput>,
    mailer: Data<Mailer>,
) -> Result<HttpResponse, AWError> {
    let result = controller::register(&db, &item, &mailer);

    match result {
        Ok(()) => Ok(HttpResponse::build(StatusCode::OK)
            .body("{ \"message\": \"Registered! Check your email to activate your account.\" }")),
        Err((status_code, message)) => Ok(HttpResponse::build(
            StatusCode::from_u16(status_code as u16).unwrap(),
        )
        .body(json!({ "message": message }).to_string())),
    }
}

#[get("/activate")]
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
    db: Data<Database>,
    Query(item): Query<ActivationInput>,
    mailer: Data<Mailer>,
) -> Result<HttpResponse, AWError> {
    let result = controller::activate(&db, &item, &mailer);

    match result {
        Ok(()) => Ok(HttpResponse::build(StatusCode::OK).body("{ \"message\": \"Activated!\" }")),
        Err((status_code, message)) => Ok(HttpResponse::build(
            StatusCode::from_u16(status_code as u16).unwrap(),
        )
        .body(json!({ "message": message }).to_string())),
    }
}

#[post("/forgot")]
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
    db: Data<Database>,
    Json(item): Json<ForgotInput>,
    mailer: Data<Mailer>,
) -> Result<HttpResponse, AWError> {
    let result = controller::forgot_password(&db, &item, &mailer);

    match result {
        Ok(()) => Ok(HttpResponse::build(StatusCode::OK)
            .body("{ \"message\": \"Please check your email.\" }")),
        Err((status_code, message)) => Ok(HttpResponse::build(
            StatusCode::from_u16(status_code as u16).unwrap(),
        )
        .body(json!({ "message": message }).to_string())),
    }
}

#[post("/change")]
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
    db: Data<Database>,
    Json(item): Json<ChangeInput>,
    auth: Auth,
    mailer: Data<Mailer>,
) -> Result<HttpResponse, AWError> {
    let result = controller::change_password(&db, &item, &auth, &mailer);

    match result {
        Ok(()) => Ok(HttpResponse::build(StatusCode::OK)
            .body(json!({"message": "Password changed."}).to_string())),
        Err((status_code, message)) => Ok(HttpResponse::build(
            StatusCode::from_u16(status_code as u16).unwrap(),
        )
        .body(json!({ "message": message }).to_string())),
    }
}

#[post("/check")]
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
async fn check(auth: Auth) -> HttpResponse {
    controller::check(&auth);
    HttpResponse::Ok().finish()
}

#[post("/reset")]
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
    db: Data<Database>,
    Json(item): Json<ResetInput>,
    mailer: Data<Mailer>,
) -> Result<HttpResponse, AWError> {
    let result = controller::reset_password(&db, &item, &mailer);

    match result {
        Ok(()) => Ok(HttpResponse::build(StatusCode::OK)
            .body(json!({"message": "Password reset"}).to_string())),
        Err((status_code, message)) => Ok(HttpResponse::build(
            StatusCode::from_u16(status_code as u16).unwrap(),
        )
        .body(json!({ "message": message }).to_string())),
    }
}

/// returns the endpoints for the Auth service
pub fn endpoints(scope: actix_web::Scope) -> actix_web::Scope {
    return scope
        .service(sessions)
        .service(destroy_session)
        .service(destroy_sessions)
        .service(login)
        .service(logout)
        .service(check)
        .service(refresh)
        .service(register)
        .service(activate)
        .service(forgot_password)
        .service(change_password)
        .service(reset_password);
}
