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
async fn login(db: Data<Database>, Json(item): Json<LoginInput>) -> Result<HttpResponse, AWError> {
    let result = web::block(move || controller::login(&db, &item)).await?;

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

#[post("/logout")]
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
async fn check(auth: Auth) -> HttpResponse {
    controller::check(&auth);
    HttpResponse::Ok().finish()
}

#[post("/reset")]
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

pub fn endpoints(scope: actix_web::Scope) -> actix_web::Scope {
    scope
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
        .service(reset_password)
}
