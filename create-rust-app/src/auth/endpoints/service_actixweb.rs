#[cfg(feature = "plugin_utoipa")]
use crate::auth::{
    AuthMessageResponse, AuthTokenResponse, JwtSecurityAddon, UserSessionJson, UserSessionResponse,
};
use actix_http::StatusCode;
use actix_web::cookie::{Cookie, SameSite};
use actix_web::{delete, get, post, web, Error as AWError, Result};
use actix_web::{
    web::{Data, Json, Path, Query},
    HttpRequest, HttpResponse,
};
use serde_json::json;
#[cfg(feature = "plugin_utoipa")]
use utoipa::OpenApi;

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

/// handler for GET requests at the .../sessions endpoint,
///
/// requires auth
///
/// queries [`db`](`Database`) for all sessions owned by the User
/// associated with [`auth`](`Auth`)
///
/// breaks up the results of that query as defined by [`info`](`PaginationParams`)
///
/// Items are arranged in the database in such a way that the most recently added or updated items are last
/// and are paginated accordingly
#[cfg_attr(feature = "plugin_utoipa", utoipa::path(
    context_path = "/api/auth",
    params(PaginationParams),
    responses(
        (status = 200, description = "success, returns a json payload with all the sessions belonging to the authenticated user", body = UserSessionResponse),
        (status = 401, description = "Error: Unauthorized"),
        (status = 500, description = "Could not fetch sessions."),
    ),
    tag = "Sessions",
    security ( ("JWT" = []))
))]
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

/// handler for DELETE requests at the .../sessions/{id} endpoint.
///
/// requires auth
///
/// deletes the entry in the `user_session` with the specified [`item_id`](`ID`) from
/// [`db`](`Database`) if it's owned by the User associated with [`auth`](`Auth`)
#[cfg_attr(feature = "plugin_utoipa", utoipa::path(
    context_path = "/api/auth",
    responses(
        (status = 200, description = "Deleted", body = AuthMessageResponse),
        (status = 401, description = "User not authenticated"),
        (status = 404, description = "User session could not be found, or does not belong to authenticated user.", body = AuthMessageResponse),
        (status = 500, description = "Internal Error.", body = AuthMessageResponse),
        (status = 500, description = "Could not delete session.", body = AuthMessageResponse),
    ),
    tag = "Sessions",
    security ( ("JWT" = []))
))]
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

/// handler for DELETE requests at the .../sessions enpoint
///
/// requires auth
///
/// destroys all entries in the `user_session` table in [`db`](`Database`) owned
/// by the User associated with [`auth`](`Auth`)
#[cfg_attr(feature = "plugin_utoipa", utoipa::path(
    context_path = "/api/auth",
    responses(
        (status = 200, description = "Deleted", body = AuthMessageResponse),
        (status = 401, description = "User not authenticated"),
        (status = 500, description = "Could not delete sessions.", body = AuthMessageResponse),
    ),
    tag = "Sessions",
    security ( ("JWT" = []))
))]
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

/// handler for POST requests at the .../login endpoint
///
/// creates a user session for the user associated with [`item`](`LoginInput`)
/// in the request body (have the `content-type` header set to `application/json` and content that can be deserialized into [`LoginInput`])
#[cfg_attr(feature = "plugin_utoipa", utoipa::path(
    context_path = "/api/auth",
    request_body(content = LoginInput, content_type = "application/json"),
    responses(
        (status = 200, description = "session created", body = AuthTokenResponse),
        (status = 400, description = "'device' cannot be longer than 256 characters.", body = AuthMessageResponse),
        (status = 400, description = "Account has not been activated.", body = AuthMessageResponse),
        (status = 401, description = "Invalid credentials.", body = AuthMessageResponse),
        (status = 500, description = "An internal server error occurred.", body = AuthMessageResponse),
        (status = 500, description = "Could not create a session.", body = AuthMessageResponse),
    ),
    tag = "Sessions",
))]
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

/// handler for POST requests to the .../logout endpount
///
/// If this is successful, delete the cookie storing the refresh token
///
/// TODO: document that it creates a refresh_token
#[cfg_attr(feature = "plugin_utoipa", utoipa::path(
    context_path = "/api/auth",
    responses(
        (status = 200, description = "deletes the \"refresh_token\" cookie"),
        (status = 401, description = "Invalid session.", body = AuthMessageResponse),
        (status = 401, description = "Could not delete session.", body = AuthMessageResponse),
    ),
    tag = "Sessions",
))]
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

/// handler for POST requests to the .../refresh endpoint
///
/// refreshes the user session associated with the clients refresh_token cookie
///
/// TODO: document that it needs a refresh_token cookie
#[cfg_attr(feature = "plugin_utoipa", utoipa::path(
    context_path = "/api/auth",
    responses(
        (status = 200, description = "uses the \"refresh_token\" cookie to give the user a new session", body=AuthTokenResponse),
        (status = 401, description = "Invalid session.", body = AuthMessageResponse),
        (status = 401, description = "Invalid token.", body = AuthMessageResponse),
    ),
    tag = "Sessions",
))]
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

/// handler for POST requests to the .../register endpoint
///
/// creates a new User with the information in [`item`](`RegisterInput`)
///
/// sends an email, using [`mailer`](`Mailer`), to the email address in [`item`](`RegisterInput`)
/// that contains a unique link that allows the recipient to activate the account associated with
/// that email address
#[cfg_attr(feature = "plugin_utoipa", utoipa::path(
    context_path = "/api/auth",
    request_body(content = RegisterInput, content_type = "application/json"),
    responses(
        (status = 200, description = "Success, sends an email to the user with a link that will let them activate their account", body=AuthMessageResponse),
        (status = 400, description = "Already registered.", body = AuthMessageResponse),
    ),
    tag = "Users",
))]
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

/// handler for GET requests to the .../activate endpoint
///
/// activates the account associated with the token in [`item`](`ActivationInput`)
#[cfg_attr(feature = "plugin_utoipa", utoipa::path(
    context_path = "/api/auth",
    params(ActivationInput),
    responses(
        (status = 200, description = "Success, account associated with activation_token is activated", body=AuthMessageResponse),
        (status = 200, description = "Already activated.", body = AuthMessageResponse),
        (status = 400, description =  "Invalid token.", body = AuthMessageResponse),
        (status = 401, description =  "Invalid token", body = AuthMessageResponse),
        (status = 500, description =  "Could not activate user. ", body = AuthMessageResponse),
    ),
    tag = "Users",
))]
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

/// handler for POST requests to the .../forgot endpoint
///
/// sends an email to the email in the ['ForgotInput'] Json in the request body
/// that will allow the user associated with that email to change their password
///
/// sends an email, using [`mailer`](`Mailer`), to the email address in [`item`](`RegisterInput`)
/// that contains a unique link that allows the recipient to reset the password
/// of the account associated with that email address (or create a new account if there is
/// no accound accosiated with the email address)
#[cfg_attr(feature = "plugin_utoipa", utoipa::path(
    context_path = "/api/auth",
    request_body(content = ForgotInput, content_type = "application/json"),
    responses(
        (status = 200, description = "Success, password reset email is sent to users email", body=AuthMessageResponse),
    ),
    tag = "Users",
))]
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

/// handler for POST requests to the .../change endpoint
///
/// requires auth
///
/// change the password of the User associated with [`auth`](`Auth`)
/// from [`item.old_password`](`ChangeInput`) to [`item.new_password`](`ChangeInput`)
#[cfg_attr(feature = "plugin_utoipa", utoipa::path(
    context_path = "/api/auth",
    request_body(content = ChangeInput, content_type = "application/json"),
    responses(
        (status = 200, description = "Success, password changed", body=AuthMessageResponse),
        (status = 400, description = "Missing password.", body=AuthMessageResponse),
        (status = 400, description = "The new password must be different.", body=AuthMessageResponse),
        (status = 400, description = "Account has not been activated.", body=AuthMessageResponse),
        (status = 400, description = "Invalid credentials.", body=AuthMessageResponse),
        (status = 500, description = "Could not find user.", body=AuthMessageResponse),
        (status = 500, description = "Could not update password.", body=AuthMessageResponse),
    ),
    tag = "Users",
    security ( ("JWT" = []))
))]
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

/// handler for POST requests to the .../check endpoint
///
/// requires auth, but doesn't match it to a user
#[cfg_attr(feature = "plugin_utoipa", utoipa::path(
    context_path = "/api/auth",
    responses(
        (status = 200, description = "Success, API is running"),
    ),
    tag = "Users",
    security ( ("JWT" = []))
))]
#[post("/check")]
async fn check(auth: Auth) -> HttpResponse {
    controller::check(&auth);
    HttpResponse::Ok().finish()
}

/// handler for POST requests to the .../reset endpoint
///
/// changes the password of the user associated with [`item.reset_token`](`ResetInput`)
/// to [`item.new_password`](`ResetInput`)
#[cfg_attr(feature = "plugin_utoipa", utoipa::path(
    context_path = "/api/auth",
    request_body(content = ResetInput, content_type = "application/json"),
    responses(
        (status = 200, description = "Password changed.", body=AuthMessageResponse),
        (status = 400, description = "Invalid token.", body=AuthMessageResponse),
        (status = 400, description = "Account has not been activated.", body=AuthMessageResponse),
        (status = 400, description = "The new password must be different.", body=AuthMessageResponse),
        (status = 401, description = "Invalid token.", body=AuthMessageResponse),
        (status = 500, description = "Could not update password.", body=AuthMessageResponse),
    ),
    tag = "Users",
))]
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

/// returns the endpoints for the Auth service
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

// swagger
#[cfg(feature = "plugin_utoipa")]
#[derive(OpenApi)]
#[openapi(
    paths(sessions, destroy_session, destroy_sessions, login, logout, refresh, register, activate, forgot_password, change_password, check, reset_password),
    components(
        schemas(UserSessionResponse, UserSessionJson, AuthMessageResponse, AuthTokenResponse, LoginInput, RegisterInput, ForgotInput, ChangeInput, ResetInput)
    ),
    tags(
        (name = "Auth", description = "users and user_sessions management endpoints"),
        (name = "Sessions", description = "Endpoints for user_sessions management"),
        (name = "Users", description = "Endpoints for useres management"),
    ),
    modifiers(&JwtSecurityAddon)
)]
pub struct ApiDoc;
