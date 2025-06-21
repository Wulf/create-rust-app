mod query;
mod mutation;
mod subscription;

use std::collections::HashSet;
pub use query::{QueryRoot};
pub use mutation::MutationRoot;
pub use subscription::SubscriptionRoot;

use actix_web::{HttpRequest, HttpResponse, web};
use async_graphql::{Data, Schema};
// use async_graphql::http::{GraphQLPlaygroundConfig, playground_source};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use create_rust_app::auth::{Auth, Permission};
use jsonwebtoken::{DecodingKey, decode, Validation};
use std::iter::FromIterator;

pub type GraphQLSchema = Schema<query::QueryRoot, mutation::MutationRoot, subscription::SubscriptionRoot>;

pub async fn index(auth: Auth, schema: web::Data<GraphQLSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner().data(auth)).await.into()
}

pub async fn index_playground() -> actix_web::Result<HttpResponse> {
    let content = std::fs::read_to_string("./.cargo/graphql-playground.html").unwrap();

    Ok(
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            // GraphQL Playground original source:
            // .body(playground_source(
            //     GraphQLPlaygroundConfig::new("/api/graphql")
            //         .with_header("Authorization", "token")
            //         .subscription_endpoint("/api/graphql/ws"),
            // ))

            // GraphQL Playground modified source to include authentication:
            .body(content)
    )
}

pub async fn index_ws(
    schema: web::Data<GraphQLSchema>,
    req: HttpRequest,
    payload: web::Payload,
) -> actix_web::Result<HttpResponse> {
    GraphQLSubscription::new(Schema::clone(&*schema))
        .on_connection_init(on_connection_init)
        .start(&req, payload)
}

#[derive(serde::Deserialize)]
struct WSConnectPayload {
    token: String,
}

pub async fn on_connection_init(value: serde_json::Value) -> async_graphql::Result<Data> {
    if let Ok(payload) = serde_json::from_value::<WSConnectPayload>(value) {
        let access_token = decode::<create_rust_app::auth::AccessTokenClaims>(
            payload.token.as_str(),
            &DecodingKey::from_secret(std::env::var("SECRET_KEY").unwrap().as_ref()),
            &Validation::default(),
        ).unwrap();

        let mut data = Data::default();
        let permissions: HashSet<Permission> = HashSet::from_iter(access_token.claims.permissions.iter().cloned());
        let roles: HashSet<String> = HashSet::from_iter(access_token.claims.roles.iter().cloned());

        data.insert(Auth {
            user_id: access_token.claims.sub,
            permissions,
            roles,
        });
        Ok(data)
    } else {
        Err("Token is required".into())
    }
}
