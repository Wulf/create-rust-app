pub mod controller;
mod model;

mod schema;

#[derive(Clone)]
pub struct OIDCProvider {
    pub name: String,
    pub client_id: String,
    pub client_secret: String,
    pub scope: Vec<String>,
    // used to discover the rest of the endpoints (.well-known)
    pub issuer_url: String,
    // URI to redirect to upon successful OAuth
    pub success_uri: String,
    // URI to redirect to when OAuth fails
    pub error_uri: String,
}

type ClientId = String;
type ClientSecret = String;
type SuccessURI = String;
type ErrorURI = String;
type ProviderFactory = fn(ClientId, ClientSecret, SuccessURI, ErrorURI) -> OIDCProvider;

impl OIDCProvider {
    pub const GOOGLE: ProviderFactory =
        |client_id: ClientId, client_secret: ClientSecret, success_uri: SuccessURI, error_uri: ErrorURI| OIDCProvider {
            name: "google".to_string(),
            scope: vec!["email".to_string()],
            issuer_url: "https://accounts.google.com".to_string(),
            client_id,
            client_secret,
            success_uri,
            error_uri,
        };

    pub fn redirect_uri(&self, api_url: String) -> String {
        format!(
            "{api_url}/api/auth/oidc/{provider_name}/login",
            api_url = api_url,
            provider_name = self.name
        )
    }
}
