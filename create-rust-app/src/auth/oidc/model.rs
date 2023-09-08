/* This file is generated and managed by dsync */

use crate::auth::oidc::schema::*;
use crate::auth::User;
use crate::diesel::*;
use diesel::QueryResult;
use serde::{Deserialize, Serialize};

type Connection = crate::Connection;

#[tsync::tsync]
#[derive(
Debug,
Serialize,
Deserialize,
Clone,
Queryable,
Insertable,
AsChangeset,
Identifiable,
Associations,
Selectable,
)]
#[diesel(table_name=user_oauth2_links, primary_key(id), belongs_to(User, foreign_key=user_id))]
pub struct UserOauth2Link {
    pub id: i32,
    pub provider: String,
    pub csrf_token: String,
    pub nonce: String,
    pub pkce_secret: String,
    pub refresh_token: Option<String>,
    pub access_token: Option<String>,
    pub subject_id: Option<String>,
    pub user_id: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[tsync::tsync]
#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=user_oauth2_links)]
pub struct CreateUserOauth2Link {
    pub provider: String,
    pub csrf_token: String,
    pub nonce: String,
    pub pkce_secret: String,
    pub refresh_token: Option<String>,
    pub access_token: Option<String>,
    pub subject_id: Option<String>,
    pub user_id: Option<i32>,
}

#[tsync::tsync]
#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=user_oauth2_links)]
pub struct UpdateUserOauth2Link {
    pub provider: Option<String>,
    pub csrf_token: Option<String>,
    pub nonce: Option<String>,
    pub pkce_secret: Option<String>,
    pub refresh_token: Option<Option<String>>,
    pub access_token: Option<Option<String>>,
    pub subject_id: Option<Option<String>>,
    pub user_id: Option<Option<i32>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl UserOauth2Link {
    pub fn create(db: &mut Connection, item: &CreateUserOauth2Link) -> QueryResult<Self> {
        use crate::auth::oidc::schema::user_oauth2_links::dsl::*;

        insert_into(user_oauth2_links)
            .values(item)
            .get_result::<Self>(db)
    }

    pub fn read(db: &mut Connection, param_id: i32) -> QueryResult<Self> {
        use crate::auth::oidc::schema::user_oauth2_links::dsl::*;

        user_oauth2_links.filter(id.eq(param_id)).first::<Self>(db)
    }

    pub fn read_by_csrf_token(
        db: &mut Connection,
        param_provider: String,
        param_token: String,
    ) -> QueryResult<Self> {
        use crate::auth::oidc::schema::user_oauth2_links::dsl::*;

        user_oauth2_links
            .filter(provider.eq(param_provider))
            .filter(csrf_token.eq(param_token))
            .first::<Self>(db)
    }

    pub fn read_by_subject(db: &mut Connection, param_subject_id: String) -> QueryResult<Self> {
        use crate::auth::oidc::schema::user_oauth2_links::dsl::*;

        user_oauth2_links
            .filter(subject_id.eq(param_subject_id))
            .first::<Self>(db)
    }

    pub fn update(
        db: &mut Connection,
        param_id: i32,
        item: &UpdateUserOauth2Link,
    ) -> QueryResult<Self> {
        use crate::auth::oidc::schema::user_oauth2_links::dsl::*;

        diesel::update(user_oauth2_links.filter(id.eq(param_id)))
            .set(item)
            .get_result(db)
    }

    pub fn delete(db: &mut Connection, param_id: i32) -> QueryResult<usize> {
        use crate::auth::oidc::schema::user_oauth2_links::dsl::*;

        diesel::delete(user_oauth2_links.filter(id.eq(param_id))).execute(db)
    }
}
