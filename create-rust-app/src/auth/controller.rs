use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

use crate::auth::{
    mail, AccessTokenClaims, Auth, PaginationParams, Permission, Role, User, UserChangeset,
    UserSession, UserSessionChangeset, UserSessionJson, UserSessionResponse, ID,
};
use crate::{Database, Mailer};

use serde::{Deserialize, Serialize};

pub const COOKIE_NAME: &'static str = "refresh_token";

type Seconds = i64;
type StatusCode = i32;
type Message = &'static str;

#[derive(Deserialize, Serialize)]
pub struct LoginInput {
    email: String,
    password: String,
    device: Option<String>,
    ttl: Option<Seconds>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    exp: usize,
    sub: ID,
    token_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterInput {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationClaims {
    exp: usize,
    sub: ID,
    token_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct ActivationInput {
    activation_token: String,
}

#[derive(Serialize, Deserialize)]
pub struct ForgotInput {
    email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetTokenClaims {
    exp: usize,
    sub: ID,
    token_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChangeInput {
    old_password: String,
    new_password: String,
}

#[derive(Serialize, Deserialize)]
pub struct ResetInput {
    reset_token: String,
    new_password: String,
}

/// /sessions
pub fn get_sessions(
    db: &Database,
    auth: &Auth,
    info: &PaginationParams,
) -> Result<UserSessionResponse, (StatusCode, Message)> {
    let mut db = db.pool.get().unwrap();

    let sessions = UserSession::read_all(&mut db, &info, auth.user_id);

    if sessions.is_err() {
        return Err((500, "Could not fetch sessions."));
    }

    let sessions: Vec<UserSession> = sessions.unwrap();
    let mut sessions_json: Vec<UserSessionJson> = vec![];

    for session in sessions {
        let session_json = UserSessionJson {
            id: session.id,
            device: session.device,
            created_at: session.created_at,
            #[cfg(not(feature = "database_sqlite"))]
            updated_at: session.updated_at,
        };

        sessions_json.push(session_json);
    }

    let num_sessions = UserSession::count_all(&mut db, auth.user_id);
    if num_sessions.is_err() {
        return Err((500, "Could not fetch sessions."));
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

    Ok(resp)
}

/// /sessions/{id}
pub fn destroy_session(
    db: &Database,
    auth: &Auth,
    item_id: ID,
) -> Result<(), (StatusCode, Message)> {
    let mut db = db.pool.get().unwrap();

    let user_session = UserSession::read(&mut db, item_id);

    if user_session.is_err() {
        return Err((500, "Internal error."));
    }

    let user_session = user_session.unwrap();

    if user_session.user_id != auth.user_id {
        return Err((404, "Session not found."));
    }

    if UserSession::delete(&mut db, user_session.id).is_err() {
        return Err((500, "Could not delete session."));
    }

    Ok(())
}

/// /sessions
pub fn destroy_sessions(db: &Database, auth: &Auth) -> Result<(), (StatusCode, Message)> {
    let mut db = db.pool.get().unwrap();

    if UserSession::delete_all_for_user(&mut db, auth.user_id).is_err() {
        return Err((500, "Could not delete sessions."));
    }

    Ok(())
}

type AccessToken = String;
type RefreshToken = String;

/// /login
/// Returns a tuple;
/// - the access token should be sent to the user in the body, and,
/// - the reset token should be sent as a secure, http-only, and same_site=strict cookie.
pub fn login(
    db: &Database,
    item: &LoginInput,
) -> Result<(AccessToken, RefreshToken), (StatusCode, Message)> {
    let mut db = db.pool.get().unwrap();

    // verify device
    let mut device = None;
    if item.device.is_some() {
        let device_string = item.device.as_ref().unwrap();
        if device_string.len() > 256 {
            return Err((400, "'device' cannot be longer than 256 characters."));
        }
    }

    let user = User::find_by_email(&mut db, item.email.clone());

    if user.is_err() {
        return Err((401, "Invalid credentials."));
    }

    let user = user.unwrap();

    if !user.activated {
        return Err((400, "Account has not been activated."));
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
        return Err((401, "Invalid credentials."));
    }

    let permissions = Permission::fetch_all(&mut db, user.id);
    if permissions.is_err() {
        println!("{:#?}", permissions.err());
        return Err((500, "An internal server error occurred."));
    }
    let permissions = permissions.unwrap();

    let roles = Role::fetch_all(&mut db, user.id);
    if roles.is_err() {
        println!("{:#?}", roles.err());
        return Err((500, "An internal server error occurred."));
    }
    let roles = roles.unwrap();

    let access_token_duration = chrono::Duration::seconds(if item.ttl.is_some() {
        std::cmp::max(item.ttl.unwrap(), 1)
    } else {
        /* 15 minutes */
        15 * 60
    });

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
        &mut db,
        &UserSessionChangeset {
            user_id: user.id,
            refresh_token: refresh_token.clone(),
            device,
        },
    );

    if user_session.is_err() {
        return Err((500, "Could not create a session."));
    }

    Ok((access_token, refresh_token))
}

/// /logout
/// If this is successful, delete the cookie storing the refresh token
pub fn logout(db: &Database, refresh_token: Option<&'_ str>) -> Result<(), (StatusCode, Message)> {
    let mut db = db.pool.get().unwrap();

    if refresh_token.is_none() {
        return Err((401, "Invalid session."));
    }

    let refresh_token = refresh_token.unwrap();

    let session = UserSession::find_by_refresh_token(&mut db, refresh_token);

    if session.is_err() {
        return Err((401, "Invalid session."));
    }

    let session = session.unwrap();

    let is_deleted = UserSession::delete(&mut db, session.id);

    if is_deleted.is_err() {
        return Err((401, "Could not delete session."));
    }

    Ok(())
}

/// /refresh
/// Returns a tuple;
/// - the access token should be sent to the user in the body, and,
/// - the reset token should be sent as a secure, http-only, and same_site=strict cookie.
pub fn refresh(
    db: &Database,
    refresh_token_str: Option<&'_ str>,
) -> Result<(AccessToken, RefreshToken), (StatusCode, Message)> {
    let mut db = db.pool.get().unwrap();

    if refresh_token_str.is_none() {
        return Err((401, "Invalid session."));
    }

    let refresh_token_str = refresh_token_str.unwrap();

    let refresh_token = decode::<RefreshTokenClaims>(
        &refresh_token_str,
        &DecodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref()),
        &Validation::default(),
    );

    if refresh_token.is_err() {
        return Err((401, "Invalid token."));
    }

    let refresh_token = refresh_token.unwrap();

    if !refresh_token
        .claims
        .token_type
        .eq_ignore_ascii_case("refresh_token")
    {
        return Err((401, "Invalid token."));
    }

    let session = UserSession::find_by_refresh_token(&mut db, refresh_token_str);

    if session.is_err() {
        return Err((401, "Invalid session."));
    }

    let session = session.unwrap();

    let permissions = Permission::fetch_all(&mut db, session.user_id);
    if permissions.is_err() {
        return Err((500, "An internal server error occurred."));
    }
    let permissions = permissions.unwrap();

    let roles = Role::fetch_all(&mut db, session.user_id);
    if roles.is_err() {
        return Err((500, "An internal server error occurred."));
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
        &mut db,
        session.id,
        &UserSessionChangeset {
            user_id: session.user_id,
            refresh_token: refresh_token_str.clone(),
            device: session.device,
        },
    );

    if session_update.is_err() {
        return Err((500, "Could not update the session."));
    }

    Ok((access_token, refresh_token_str))
}

/// /register
pub fn register(
    db: &Database,
    item: &RegisterInput,
    mailer: &Mailer,
) -> Result<(), (StatusCode, Message)> {
    let mut db = db.pool.get().unwrap();

    let user = User::find_by_email(&mut db, (&item.email).to_string());

    if user.is_ok() {
        let user = user.unwrap();
        if !user.activated {
            User::delete(&mut db, user.id).unwrap();
        } else {
            return Err((400, "Already registered."));
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
        &mut db,
        &UserChangeset {
            activated: false,
            email: item.email.clone(),
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
            "http://localhost:3000/activate?token={token}",
            token = token
        ),
    );

    Ok(())
}

/// /activate
pub fn activate(
    db: &Database,
    item: &ActivationInput,
    mailer: &Mailer,
) -> Result<(), (StatusCode, Message)> {
    let mut db = db.pool.get().unwrap();

    let token = decode::<RegistrationClaims>(
        &item.activation_token,
        &DecodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref()),
        &Validation::default(),
    );

    if token.is_err() {
        return Err((401, "Invalid token."));
    }

    let token = token.unwrap();

    if !token
        .claims
        .token_type
        .eq_ignore_ascii_case("activation_token")
    {
        return Err((401, "Invalid token."));
    }

    let user = User::read(&mut db, token.claims.sub);

    if user.is_err() {
        return Err((400, "Invalid token."));
    }

    let user = user.unwrap();

    if user.activated {
        return Err((200, "Already activated!"));
    }

    let activated_user = User::update(
        &mut db,
        user.id,
        &UserChangeset {
            activated: true,
            email: user.email.clone(),
            hash_password: user.hash_password,
        },
    );

    if activated_user.is_err() {
        return Err((500, "Could not activate user."));
    }

    mail::auth_activated::send(&mailer, &user.email);

    Ok(())
}

/// /forgot
pub fn forgot_password(
    db: &Database,
    item: &ForgotInput,
    mailer: &Mailer,
) -> Result<(), (StatusCode, Message)> {
    let mut db = db.pool.get().unwrap();

    let user_result = User::find_by_email(&mut db, item.email.clone());

    if user_result.is_ok() {
        let user = user_result.unwrap();

        // if !user.activated {
        //   return Ok(HttpResponse::build(400).body(" has not been activate"))
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
            "http://localhost:3000/reset?token={reset_token}",
            reset_token = reset_token
        );
        mail::auth_recover_existent_account::send(&mailer, &user.email, link);
    } else {
        let link = &format!("http://localhost:300/register");
        mail::auth_recover_nonexistent_account::send(&mailer, &item.email, link);
    }

    Ok(())
}

/// /change
pub fn change_password(
    db: &Database,
    item: &ChangeInput,
    auth: &Auth,
    mailer: &Mailer,
) -> Result<(), (StatusCode, Message)> {
    if item.old_password.len() == 0 || item.new_password.len() == 0 {
        return Err((400, "Missing password"));
    }

    if item.old_password.eq(&item.new_password) {
        return Err((400, "The new password must be different"));
    }

    let mut db = db.pool.get().unwrap();

    let user = User::read(&mut db, auth.user_id);

    if user.is_err() {
        return Err((500, "Could not find user"));
    }

    let user = user.unwrap();

    if !user.activated {
        return Err((400, "Account has not been activated"));
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
        return Err((400, "Invalid credentials"));
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
        &mut db,
        auth.user_id,
        &UserChangeset {
            email: user.email.clone(),
            hash_password: new_hash,
            activated: user.activated,
        },
    );

    if updated_user.is_err() {
        return Err((500, "Could not update password"));
    }

    mail::auth_password_changed::send(&mailer, &user.email);

    Ok(())
}

/// /check
pub fn check(_: &Auth) {}

/// reset
pub fn reset_password(
    db: &Database,
    item: &ResetInput,
    mailer: &Mailer,
) -> Result<(), (StatusCode, Message)> {
    let mut db = db.pool.get().unwrap();

    if item.new_password.len() == 0 {
        return Err((400, "Missing password"));
    }

    let token = decode::<ResetTokenClaims>(
        &item.reset_token,
        &DecodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref()),
        &Validation::default(),
    );

    if token.is_err() {
        return Err((401, "Invalid token."));
    }

    let token = token.unwrap();

    if !token.claims.token_type.eq_ignore_ascii_case("reset_token") {
        return Err((401, "Invalid token."));
    }

    let user = User::read(&mut db, token.claims.sub);

    if user.is_err() {
        return Err((400, "Invalid token."));
    }

    let user = user.unwrap();

    if !user.activated {
        return Err((400, "Account has not been activated"));
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
        &mut db,
        user.id,
        &UserChangeset {
            email: user.email.clone(),
            hash_password: new_hash,
            activated: user.activated,
        },
    );

    if update.is_err() {
        return Err((500, "Could not update password"));
    }

    mail::auth_password_reset::send(&mailer, &user.email);

    Ok(())
}
