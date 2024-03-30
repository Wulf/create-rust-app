use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

use crate::auth::{
    AccessTokenClaims, Auth, PaginationParams, Permission, Role, User, UserChangeset, UserSession,
    UserSessionChangeset, UserSessionJson, UserSessionResponse, ID,
};
use crate::{Connection, Database, Mailer};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

pub const COOKIE_NAME: &str = "refresh_token";

lazy_static! {
    pub static ref ARGON_CONFIG: argon2::Config<'static> = argon2::Config {
        variant: argon2::Variant::Argon2id,
        version: argon2::Version::Version13,
        secret: std::env::var("SECRET_KEY").map_or_else(|_| panic!("No SECRET_KEY environment variable set!"), |s| Box::leak(s.into_boxed_str()).as_bytes()),
        ..Default::default()
    };
    // TODO: instead of initializing EncodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref()) repeatedly in the code, just initialize it here and use it everywhere
}

#[cfg(not(debug_assertions))]
type Seconds = i64;
type StatusCode = u16;
type Message = &'static str;

#[derive(Deserialize, Serialize)]
#[cfg_attr(feature = "plugin_utoipa", derive(utoipa::ToSchema))]
/// Rust struct representing the Json body of
/// POST requests to the .../login endpoint
pub struct LoginInput {
    email: String,
    password: String,
    device: Option<String>,
    #[cfg(not(debug_assertions))]
    ttl: Option<Seconds>, // Seconds
    #[cfg(debug_assertions)]
    ttl: Option<i64>, // Seconds
}

#[derive(Debug, Serialize, Deserialize)]
/// TODO: documentation
pub struct RefreshTokenClaims {
    exp: usize,
    sub: ID,
    token_type: String,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "plugin_utoipa", derive(utoipa::ToSchema))]
/// Rust struct representing the Json body of
/// POST requests to the .../register endpoint
pub struct RegisterInput {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
/// TODO: documentation
pub struct RegistrationClaims {
    exp: usize,
    sub: ID,
    token_type: String,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "plugin_utoipa", derive(utoipa::IntoParams))]
/// Rust struct representing the Json body of
/// GET requests to the .../activate endpoint
pub struct ActivationInput {
    activation_token: String,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "plugin_utoipa", derive(utoipa::ToSchema))]
/// Rust struct representing the Json body of
/// POST requests to the /forgot endpoint
pub struct ForgotInput {
    email: String,
}

#[derive(Debug, Serialize, Deserialize)]
/// TODO: documentation
pub struct ResetTokenClaims {
    exp: usize,
    sub: ID,
    token_type: String,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "plugin_utoipa", derive(utoipa::ToSchema))]
/// Rust struct representing the Json body of
/// POST requests to the /change endpoint
pub struct ChangeInput {
    old_password: String,
    new_password: String,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "plugin_utoipa", derive(utoipa::ToSchema))]
/// Rust struct representing the Json body of
/// POST requests to the /reset endpoint
pub struct ResetInput {
    reset_token: String,
    new_password: String,
}

/// /sessions
///
/// queries [`db`](`Database`) for all sessions owned by the User
/// associated with [`auth`](`Auth`)
///
/// breaks up the results of that query as defined by [`info`](`PaginationParams`)
///
/// # Returns [`Result`]
/// - Ok([`UserSessionResponse`])
///     - the results of the query paginated according to [`info`](`PaginationParams`)
/// - Err([`StatusCode`], [`Message`])
///
/// # Errors
/// - 500: Could not fetch sessions
///
/// # Panics
/// - could not connect to database
///
/// TODO: don't panic if db connection fails, just return an error
pub fn get_sessions(
    db: &Database,
    auth: &Auth,
    info: &PaginationParams,
) -> Result<UserSessionResponse, (StatusCode, Message)> {
    let mut db = db.get_connection().unwrap();

    let Ok(sessions) = UserSession::read_all(&mut db, info, auth.user_id) else {
        return Err((500, "Could not fetch sessions."));
    };

    let sessions_json: Vec<UserSessionJson> = sessions
        .iter()
        .map(|s| UserSessionJson {
            id: s.id,
            device: s.device.clone(),
            created_at: s.created_at,
            #[cfg(not(feature = "database_sqlite"))]
            updated_at: s.updated_at,
        })
        .collect();

    let Ok(num_sessions) = UserSession::count_all(&mut db, auth.user_id) else {
        return Err((500, "Could not fetch sessions."));
    };

    let num_pages = (num_sessions / info.page_size) + i64::from(num_sessions % info.page_size != 0);

    let resp = UserSessionResponse {
        sessions: sessions_json,
        num_pages,
    };

    Ok(resp)
}

/// /sessions/{id}
///
/// deletes the entry in the `user_session` with the specified [`item_id`](`ID`) from
/// [`db`](`Database`) if it's owned by the User associated with [`auth`](`Auth`)
///
/// # Errors
/// - 404: Session not found
/// - 500: Internal error
/// - 500: Could not delete session
///
/// # Panics
/// - could not connect to database
///
/// TODO: don't panic if db connection fails, just return an error
pub fn destroy_session(
    db: &Database,
    auth: &Auth,
    item_id: ID,
) -> Result<(), (StatusCode, Message)> {
    let mut db = db.get_connection().unwrap();

    let user_session = match UserSession::read(&mut db, item_id) {
        Ok(user_session) if user_session.user_id == auth.user_id => user_session,
        Ok(_) => return Err((404, "Session not found.")),
        Err(_) => return Err((500, "Internal error.")),
    };

    UserSession::delete(&mut db, user_session.id)
        .map_err(|_| (500, "Could not delete session."))?;

    Ok(())
}

/// /sessions
///
/// destroys all entries in the `user_session` table in [`db`](`Database`) owned
/// by the User associated with [`auth`](`Auth`)
///
/// # Errors
/// - 500: Could not delete sessions
///
/// # Panics
/// - could not connect to database
///
/// TODO: don't panic if db connection fails, just return an error
pub fn destroy_sessions(db: &Database, auth: &Auth) -> Result<(), (StatusCode, Message)> {
    let mut db = db.get_connection().unwrap();

    UserSession::delete_all_for_user(&mut db, auth.user_id)
        .map_err(|_| (500, "Could not delete sessions."))?;

    Ok(())
}

type AccessToken = String;
type RefreshToken = String;

/// /login
///
/// creates a user session for the user associated with [`item`](`LoginInput`)
/// in the request body (have the `content-type` header set to `application/json` and content that can be deserialized into [`LoginInput`])
///
/// # Returns [`Result`]
/// - Ok([`AccessToken`], [`RefreshToken`])
///     - an access token that should be sent to the user in the response body,
///     - a reset token that should be sent as a secure, http-only, and `same_site=strict` cookie.
/// - Err([`StatusCode`], [`Message`])
///
/// # Errors
/// - 400: 'device' cannot be longer than 256 characters.
/// - 400: Account has not been activated.
/// - 401: Invalid credentials.
///
/// # Panics
/// - could not connect to database
/// - verifying the password hash fails
///
/// TODO: neither of these should panic, just return an error
pub fn login(
    db: &Database,
    item: &LoginInput,
) -> Result<(AccessToken, RefreshToken), (StatusCode, Message)> {
    let mut db = db.get_connection().unwrap();

    // verify device
    let device = match item.device {
        Some(ref device) if device.len() > 256 => {
            return Err((400, "'device' cannot be longer than 256 characters."));
        }
        Some(ref device) => Some(device.clone()),
        None => None,
    };

    let user = match User::find_by_email(&mut db, item.email.clone()) {
        Ok(user) if user.activated => user,
        Ok(_) => return Err((400, "Account has not been activated.")),
        Err(_) => return Err((401, "Invalid credentials.")),
    };

    let is_valid = argon2::verify_encoded_ext(
        &user.hash_password,
        item.password.as_bytes(),
        ARGON_CONFIG.secret,
        ARGON_CONFIG.ad,
    )
    .unwrap();

    if !is_valid {
        return Err((401, "Invalid credentials."));
    }

    create_user_session(&mut db, device, None, user.id)
}

// TODO: Wrap this in a database transaction
/// create a user session for the user with [`user_id`](`i32`)
///
/// # Errors
/// - 400: 'device' cannot be longer than 256 characters.
/// - 500: An internal server error occurred.
/// - 500: Could not create session.
///
/// # Panics
/// - could not connect to database
/// - could not get `SECRET_KEY` from environment
///
/// TODO: don't panic if db connection fails, just return an error
pub fn create_user_session(
    db: &mut Connection,
    device_type: Option<String>,
    ttl: Option<i64>,
    user_id: i32,
) -> Result<(AccessToken, RefreshToken), (StatusCode, Message)> {
    // verify device
    let device = match device_type {
        Some(device) if device.len() > 256 => {
            return Err((400, "'device' cannot be longer than 256 characters."));
        }
        Some(device) => Some(device),
        None => None,
    };

    let Ok(permissions) = Permission::fetch_all(db, user_id) else {
        return Err((500, "An internal server error occurred."));
    };

    let Ok(roles) = Role::fetch_all(db, user_id) else {
        return Err((500, "An internal server error occurred."));
    };

    let access_token_duration = chrono::Duration::seconds(
        ttl.map_or_else(|| /* 15 minutes */ 15 * 60, |tt| std::cmp::max(tt, 1)),
    );

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let access_token_claims = AccessTokenClaims {
        exp: (chrono::Utc::now() + access_token_duration).timestamp() as usize,
        sub: user_id,
        token_type: "access_token".to_string(),
        roles,
        permissions,
    };

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let refresh_token_claims = RefreshTokenClaims {
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
        sub: user_id,
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

    UserSession::create(
        db,
        &UserSessionChangeset {
            user_id,
            refresh_token: refresh_token.clone(),
            device,
        },
    )
    .map_err(|_| (500, "Could not create session."))?;

    Ok((access_token, refresh_token))
}

/// /logout
/// If this is successful, delete the cookie storing the refresh token
///
/// # Errors
/// - 401: Invalid session
/// - 401: Invalid token
/// - 500: Could not delete session
///
/// # Panics
/// - could not connect to database
///
/// TODO: don't panic if db connection fails, just return an error
pub fn logout(db: &Database, refresh_token: Option<&'_ str>) -> Result<(), (StatusCode, Message)> {
    let mut db = db.get_connection().unwrap();

    let Some(refresh_token) = refresh_token else {
        return Err((401, "Invalid session."));
    };

    let Ok(session) = UserSession::find_by_refresh_token(&mut db, refresh_token) else {
        return Err((401, "Invalid session."));
    };

    UserSession::delete(&mut db, session.id).map_err(|_| (500, "Could not delete session."))?;

    Ok(())
}

/// /refresh
///
/// refreshes the user session associated with the clients `refresh_token` cookie
///
/// # Returns [`Result`]
/// - Ok([`AccessToken`], [`RefreshToken`])
///     - an access token that should be sent to the user in the response body,
///     - a reset token that should be sent as a secure, http-only, and `same_site=strict` cookie.
/// - Err([`StatusCode`], [`Message`])
///
/// # Errors
/// - 401: Invalid session
/// - 401: Invalid token
/// - 500: Could not update session
/// - 500: An internal server error occurred
///
/// # Panics
/// - could not connect to database
/// - could not get `SECRET_KEY` from environment
///
/// TODO: don't panic if db connection fails, just return an error
pub fn refresh(
    db: &Database,
    refresh_token_str: Option<&'_ str>,
) -> Result<(AccessToken, RefreshToken), (StatusCode, Message)> {
    let mut db = db.get_connection().unwrap();

    let Some(refresh_token_str) = refresh_token_str else {
        return Err((401, "Invalid session."));
    };

    let _refresh_token = match decode::<RefreshTokenClaims>(
        refresh_token_str,
        &DecodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref()),
        &Validation::default(),
    ) {
        Ok(token)
            if token
                .claims
                .token_type
                .eq_ignore_ascii_case("refresh_token") =>
        {
            token
        }
        _ => return Err((401, "Invalid token.")),
    };

    let Ok(session) = UserSession::find_by_refresh_token(&mut db, refresh_token_str) else {
        return Err((401, "Invalid session."));
    };

    let Ok(permissions) = Permission::fetch_all(&mut db, session.user_id) else {
        return Err((500, "An internal server error occurred."));
    };

    let Ok(roles) = Role::fetch_all(&mut db, session.user_id) else {
        return Err((500, "An internal server error occurred."));
    };

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let access_token_claims = AccessTokenClaims {
        exp: (chrono::Utc::now() + chrono::Duration::minutes(15)).timestamp() as usize,
        sub: session.user_id,
        token_type: "access_token".to_string(),
        roles,
        permissions,
    };

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
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
    UserSession::update(
        &mut db,
        session.id,
        &UserSessionChangeset {
            user_id: session.user_id,
            refresh_token: refresh_token_str.clone(),
            device: session.device,
        },
    )
    .map_err(|_| (500, "Could not update session."))?;

    Ok((access_token, refresh_token_str))
}

/// /register
///
/// creates a new User with the information in [`item`](`RegisterInput`)
///
/// sends an email, using [`mailer`](`Mailer`), to the email address in [`item`](`RegisterInput`)
/// that contains a unique link that allows the recipient to activate the account associated with
/// that email address
///
/// # Errors
/// - 400: Already registered
///
/// # Panics
/// - could not connect to database
/// - could not get `SECRET_KEY` from environment
/// - any of the database operations fail
///
/// TODO: don't panic if db connection fails, just return an error
pub fn register(
    db: &Database,
    item: &RegisterInput,
    mailer: &Mailer,
) -> Result<(), (StatusCode, Message)> {
    let mut db = db.get_connection().unwrap();

    match User::find_by_email(&mut db, item.email.to_string()) {
        Ok(user) if user.activated => return Err((400, "Already registered.")),
        Ok(user) => {
            User::delete(&mut db, user.id).unwrap();
        }
        Err(_) => (),
    }

    let salt = generate_salt();
    let hash = argon2::hash_encoded(item.password.as_bytes(), &salt, &ARGON_CONFIG).unwrap();

    let user = User::create(
        &mut db,
        &UserChangeset {
            activated: false,
            email: item.email.clone(),
            hash_password: hash,
        },
    )
    .unwrap();

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
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

    mailer
        .templates
        .send_register(mailer, &user.email, &format!("activate?token={token}"));

    Ok(())
}

/// /activate
///
/// activates the account associated with the token in [`item`](`ActivationInput`)
///
/// # Errors
/// - 401: Invalid token
/// - 401: Invalid token
/// - 400: Invalid token
/// - 200: Already activated!
/// - 500: Could not activate user
///
/// # Panics
/// - could not connect to database
/// - could not get `SECRET_KEY` from environment
///
/// TODO: don't panic if db connection fails, just return an error
pub fn activate(
    db: &Database,
    item: &ActivationInput,
    mailer: &Mailer,
) -> Result<(), (StatusCode, Message)> {
    let mut db = db.get_connection().unwrap();

    let token = match decode::<RegistrationClaims>(
        &item.activation_token,
        &DecodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref()),
        &Validation::default(),
    ) {
        Ok(token)
            if token
                .claims
                .token_type
                .eq_ignore_ascii_case("activation_token") =>
        {
            token
        }
        _ => return Err((401, "Invalid token.")),
    };

    let user = match User::read(&mut db, token.claims.sub) {
        Ok(user) if !user.activated => user,
        Ok(_) => return Err((200, "Already activated!")),
        Err(_) => return Err((400, "Invalid token.")),
    };

    User::update(
        &mut db,
        user.id,
        &UserChangeset {
            activated: true,
            email: user.email.clone(),
            hash_password: user.hash_password,
        },
    )
    .map_err(|_| (500, "Could not activate user."))?;

    mailer.templates.send_activated(mailer, &user.email);

    Ok(())
}

/// /forgot
/// sends an email to the email in the [`ForgotInput`] Json in the request body
/// that will allow the user associated with that email to change their password
///
/// sends an email, using [`mailer`](`Mailer`), to the email address in [`item`](`RegisterInput`)
/// that contains a unique link that allows the recipient to reset the password
/// of the account associated with that email address (or create a new account if there is
/// no accound accosiated with the email address)
///
/// # Errors
/// - None
///
/// # Panics
/// - could not connect to database
/// - current timestamp could not be converted from `i64` to `usize`
/// - could not get `SECRET_KEY` from environment
///
/// TODO: don't panic if db connection fails, just return an error
pub fn forgot_password(
    db: &Database,
    item: &ForgotInput,
    mailer: &Mailer,
) -> Result<(), (StatusCode, Message)> {
    let mut db = db.get_connection().unwrap();

    let user_result = User::find_by_email(&mut db, item.email.clone());

    if let Ok(user) = user_result {
        // if !user.activated {
        //   return Ok(HttpResponse::build(400).body(" has not been activate"))
        // }

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
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

        let link = &format!("reset?token={reset_token}");
        mailer
            .templates
            .send_recover_existent_account(mailer, &user.email, link);
    } else {
        let link = &"register".to_string();
        mailer
            .templates
            .send_recover_nonexistent_account(mailer, &item.email, link);
    }

    Ok(())
}

/// /change
///
/// change the password of the User associated with [`auth`](`Auth`)
/// from [`item.old_password`](`ChangeInput`) to [`item.new_password`](`ChangeInput`)
///
/// # Errors
/// - 400: Missing password
/// - 400: The new password must be different
/// - 400: Account has not been activated
/// - 401: Invalid credentials
/// - 500: Could not update password
/// - 500: Could not find user
///
/// # Panics
/// - could not connect to database
/// - could not get `SECRET_KEY` from environment
///
/// TODO: don't panic if db connection fails, just return an error
pub fn change_password(
    db: &Database,
    item: &ChangeInput,
    auth: &Auth,
    mailer: &Mailer,
) -> Result<(), (StatusCode, Message)> {
    if item.old_password.is_empty() || item.new_password.is_empty() {
        return Err((400, "Missing password"));
    }

    if item.old_password.eq(&item.new_password) {
        return Err((400, "The new password must be different"));
    }

    let mut db = db.get_connection().unwrap();

    let user = match User::read(&mut db, auth.user_id) {
        Ok(user) if user.activated => user,
        Ok(_) => return Err((400, "Account has not been activated")),
        Err(_) => return Err((500, "Could not find user")),
    };

    let is_old_password_valid = argon2::verify_encoded_ext(
        &user.hash_password,
        item.old_password.as_bytes(),
        ARGON_CONFIG.secret,
        ARGON_CONFIG.ad,
    )
    .unwrap();

    if !is_old_password_valid {
        return Err((401, "Invalid credentials"));
    }

    let salt = generate_salt();
    let new_hash =
        argon2::hash_encoded(item.new_password.as_bytes(), &salt, &ARGON_CONFIG).unwrap();

    User::update(
        &mut db,
        auth.user_id,
        &UserChangeset {
            email: user.email.clone(),
            hash_password: new_hash,
            activated: user.activated,
        },
    )
    .map_err(|_| (500, "Could not update password"))?;

    mailer.templates.send_password_changed(mailer, &user.email);

    Ok(())
}

/// /check
///
/// just a lifeline function, clients can post to this endpoint to check
/// if the auth service is running
pub const fn check(_: &Auth) {}

/// reset
///
/// changes the password of the user associated with [`item.reset_token`](`ResetInput`)
/// to [`item.new_password`](`ResetInput`)
///
/// # Errors
/// - 400: Missing password
/// - 401: Invalid token
/// - 400: Invalid token
/// - 400: Account has not been activated
/// - 500: Could not update password
///
/// # Panics
/// - could not connect to database
/// - could not get `SECRET_KEY` from environment
///
/// TODO: don't panic if db connection fails, just return an error
pub fn reset_password(
    db: &Database,
    item: &ResetInput,
    mailer: &Mailer,
) -> Result<(), (StatusCode, Message)> {
    let mut db = db.get_connection().unwrap();

    if item.new_password.is_empty() {
        return Err((400, "Missing password"));
    }

    let token = match decode::<ResetTokenClaims>(
        &item.reset_token,
        &DecodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref()),
        &Validation::default(),
    ) {
        Ok(token) if token.claims.token_type.eq_ignore_ascii_case("reset_token") => token,
        _ => return Err((401, "Invalid token.")),
    };

    let user = match User::read(&mut db, token.claims.sub) {
        Ok(user) if user.activated => user,
        Ok(_) => return Err((400, "Account has not been activated")),
        Err(_) => return Err((400, "Invalid token.")),
    };

    let salt = generate_salt();
    let new_hash =
        argon2::hash_encoded(item.new_password.as_bytes(), &salt, &ARGON_CONFIG).unwrap();

    User::update(
        &mut db,
        user.id,
        &UserChangeset {
            email: user.email.clone(),
            hash_password: new_hash,
            activated: user.activated,
        },
    )
    .map_err(|_| (500, "Could not update password"))?;

    mailer.templates.send_password_reset(mailer, &user.email);

    Ok(())
}

#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn generate_salt() -> [u8; 16] {
    use rand::Fill;
    let mut salt = [0; 16];
    // this does not fail
    salt.try_fill(&mut rand::thread_rng()).unwrap();
    salt
}
