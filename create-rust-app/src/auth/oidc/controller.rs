use crate::{
    auth::{
        controller::{create_user_session, generate_salt, ARGON_CONFIG},
        AuthConfig, User, UserChangeset,
    },
    AppConfig, Database,
};
use anyhow::Result;
use diesel::OptionalExtension;
use rand::{distributions::Alphanumeric, Rng};

use super::{
    model::{CreateUserOauth2Link, UpdateUserOauth2Link, UserOauth2Link},
    OIDCProvider,
};

use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
    reqwest::async_http_client,
    AccessTokenHash, AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce,
    OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse,
};

async fn create_oidc_client(provider: &OIDCProvider, app_url: String) -> Result<CoreClient> {
    let provider_metadata = CoreProviderMetadata::discover_async(
        IssuerUrl::new(provider.clone().issuer_url)?,
        async_http_client,
    )
        .await?;

    Ok(CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(provider.clone().client_id),
        Some(ClientSecret::new(provider.clone().client_secret)),
    )
        .set_redirect_uri(RedirectUrl::new(provider.redirect_uri(app_url))?))
}

pub async fn oidc_login_url(
    db: &Database,
    app_config: &AppConfig,
    auth_config: &AuthConfig,
    provider_name: String,
) -> Result<Option<String>> {
    let mut db = db.get_connection();

    let provider_config: Option<_> = auth_config
        .clone()
        .oidc_providers
        .into_iter()
        .find(|provider_config| provider_config.name.eq(&provider_name));

    if provider_config.is_none() {
        return Ok(None);
    }

    let provider = provider_config.unwrap();

    let client = create_oidc_client(&provider, app_config.clone().app_url).await?;

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // TODO: factor in nonce
    let (auth_url, csrf_token, nonce) = client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_extra_param("access_type", "offline") // TODO: Cleanup: this line is Google-OIDC-specific for retrieving a refresh token
        .add_scopes(provider.scope.into_iter().map(Scope::new))
        .set_pkce_challenge(pkce_challenge)
        .url();

    UserOauth2Link::create(
        &mut db,
        &CreateUserOauth2Link {
            provider: provider_name,
            access_token: None,
            refresh_token: None,
            subject_id: None,
            user_id: None,
            csrf_token: csrf_token.secret().to_owned(),
            nonce: nonce.secret().to_owned(),
            pkce_secret: pkce_verifier.secret().to_owned(),
        },
    )?;

    Ok(Some(auth_url.to_string()))
}

type RefreshToken = String;
type AccessToken = String;
type StatusCode = i32;
type Message = String;
pub async fn oauth_login(
    db: &Database,
    app_config: &AppConfig,
    auth_config: &AuthConfig,
    provider_name: String,
    query_param_code: Option<String>,
    query_param_error: Option<String>,
    query_param_state: Option<String>,
) -> Result<(AccessToken, RefreshToken), (StatusCode, Message)> {
    let db = &mut db.get_connection();

    // 1. Make sure this provider is setup
    let provider_config = auth_config
        .clone()
        .oidc_providers
        .clone()
        .into_iter()
        .find(|provider_config| provider_config.name.eq(&provider_name));

    if provider_config.is_none() {
        return Err((501, "This oauth provider is not supported".into()));
    }

    let provider = provider_config.unwrap();

    // 2. make sure we haven't encountered an error
    if query_param_error.is_some() {
        /*
        =================================================================
        Valid values for this error param:
        https://datatracker.ietf.org/doc/html/rfc6749#section-5.2
        =================================================================

        invalid_request
               The request is missing a required parameter, includes an
               unsupported parameter value (other than grant type),
               repeats a parameter, includes multiple credentials,
               utilizes more than one mechanism for authenticating the
               client, or is otherwise malformed.

         invalid_client
               Client authentication failed (e.g., unknown client, no
               client authentication included, or unsupported
               authentication method).  The authorization server MAY
               return an HTTP 401 (Unauthorized) status code to indicate
               which HTTP authentication schemes are supported.  If the
               client attempted to authenticate via the "Authorization"
               request header field, the authorization server MUST
               respond with an HTTP 401 (Unauthorized) status code and
               include the "WWW-Authenticate" response header field
               matching the authentication scheme used by the client.

         invalid_grant
               The provided authorization grant (e.g., authorization
               code, resource owner credentials) or refresh token is
               invalid, expired, revoked, does not match the redirection
               URI used in the authorization request, or was issued to
               another client.

         unauthorized_client
               The authenticated client is not authorized to use this
               authorization grant type.

         unsupported_grant_type
               The authorization grant type is not supported by the
               authorization server.

         invalid_scope
               The requested scope is invalid, unknown, malformed, or
               exceeds the scope granted by the resource owner.
        */
        return Err((401, query_param_error.unwrap()));
    }

    // 3. make sure the CSRF/state variable is what we expect (i.e. exists in our db)
    // later on, we'll use the pkce verifier associated with this csrf token
    if query_param_state.is_none() {
        return Err((400, "Invalid CSRF token".into()));
    }
    let state = query_param_state.unwrap();
    let oauth_request = UserOauth2Link::read_by_csrf_token(db, provider_name.clone(), state)
        .expect("Invalid oauth2 redirection");

    let pkce_verifier = PkceCodeVerifier::new(oauth_request.pkce_secret);

    // 4. exchange code for a token!
    if query_param_code.is_none() {
        return Err((400, "Invalid code".into()));
    }

    let code = query_param_code.unwrap();

    let client = create_oidc_client(&provider, app_config.clone().app_url).await;

    if client.is_err() {
        return Err((500, "Internal server error".into()));
    }

    let client = client.unwrap();

    let token_result = client
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await;

    if token_result.is_err() {
        return Err((400, "Invalid code".into()));
    }

    let token_response = token_result.unwrap();

    let id_token = token_response.id_token();

    if id_token.is_none() {
        return Err((500, "Server did not return an ID token".into()));
    }

    let id_token = id_token.unwrap();

    let claims = id_token.claims(
        &client.id_token_verifier(),
        &Nonce::new(oauth_request.nonce),
    );

    if claims.is_err() {
        return Err((500, "Invalid ID token claims".into()));
    }

    let claims = claims.unwrap();

    if let Some(expected_access_token_hash) = claims.access_token_hash() {
        let signing_alg = id_token.signing_alg();
        if signing_alg.is_err() {
            return Err((500, "Invalid signing algorithm".into()));
        }
        let signing_alg = signing_alg.unwrap();

        let actual_access_token_hash =
            AccessTokenHash::from_token(token_response.access_token(), &signing_alg);

        if actual_access_token_hash.is_err() {
            return Err((500, "Invalid access token".into()));
        }
        let actual_access_token_hash = actual_access_token_hash.unwrap();
        if actual_access_token_hash != *expected_access_token_hash {
            return Err((401, "Invalid access token".into()));
        }
    }

    let subject = claims.subject().to_string();

    // OAuth login can happen in 1 of two ways
    // 1. Check if the subject is already present and linked to an existing user
    // 2. Link the subject to a new user (unless the email is already claimed by a local account)

    let oauth2_link = UserOauth2Link::read_by_subject(db, subject).optional();
    if oauth2_link.is_err() {
        return Err((500, "Internal server error".into()));
    }
    let oauth2_link = oauth2_link.unwrap();
    if oauth2_link.is_some() {
        let oauth2_link = oauth2_link.unwrap();
        if oauth2_link.user_id.is_none() {
            return Err((500, "Internal server error".into()));
        }
        let user = User::read(db, oauth2_link.user_id.unwrap());
        if user.is_err() {
            return Err((500, "Internal server error".into()));
        }
        let user = user.unwrap();

        // TODO: put this in a transaction because we'll create a session and if that fails, we need to rollback!

        UserOauth2Link::update(
            db,
            oauth_request.id,
            &UpdateUserOauth2Link {
                provider: None,
                access_token: Some(Some(token_response.access_token().secret().to_string())),
                refresh_token: token_response
                    .refresh_token()
                    .map(|token| Some(token.secret().to_string())),
                csrf_token: None,
                nonce: None,
                pkce_secret: None,
                user_id: None,
                subject_id: None,
                created_at: None,
                updated_at: None,
            },
        )
            .unwrap();

        let (access_token, refresh_token) = create_user_session(
            db,
            Some(format!("Oauth2 - {}", &provider_name)),
            None,
            user.id,
        )
            .map_err(|error| (error.0, error.1.to_string()))?;

        return Ok((access_token, refresh_token));
    }

    // subject is not already present, let's create a new user!

    let email = claims.email();

    if email.is_none() {
        return Err((500, "No email returned".into()));
    }
    if claims.email_verified().is_none() || claims.email_verified().unwrap() == false {
        return Err((500, "Email not verified".into()));
    }
    let email = email.unwrap().to_string();

    let existing_user = User::find_by_email(db, email.clone()).optional();
    if existing_user.is_err() {
        return Err((500, "Internal server error".into()));
    }
    if existing_user.unwrap().is_some() {
        return Err((500, "Email already registered".into()));
    }

    // create a random password
    let salt = generate_salt();
    let random_password = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect::<String>();
    let hash = argon2::hash_encoded(random_password.as_bytes(), &salt, &ARGON_CONFIG).unwrap();
    let new_user = User::create(
        db,
        &UserChangeset {
            email: email,
            activated: false, // do not activate the account because it should not be allowed to login locally
            hash_password: hash,
        },
    );
    if new_user.is_err() {
        return Err((500, "Internal server error".into()));
    }
    let new_user = new_user.unwrap();

    // TODO: put this in a a transaction because we've created a user at this point and if this
    // next step doesn't work, we need to rollback!
    UserOauth2Link::update(
        db,
        oauth_request.id,
        &UpdateUserOauth2Link {
            provider: None,
            access_token: Some(Some(token_response.access_token().secret().to_string())),
            refresh_token: Some(
                token_response
                    .refresh_token()
                    .map(|token| token.secret().into()),
            ),
            csrf_token: Some("".into()),
            nonce: Some("".into()),
            pkce_secret: Some("".into()),
            user_id: Some(Some(new_user.id)),
            subject_id: Some(Some(claims.subject().to_string())),
            created_at: None,
            updated_at: None,
        },
    )
        .unwrap();

    Ok(create_user_session(
        db,
        Some(format!("Oauth2 - {}", &provider_name)),
        None,
        new_user.id,
    )
        .map_err(|error| (error.0, error.1.to_string())))?
}
