use axum::{
    Router,
    extract::{Query, State},
    routing::get,
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::user as user_entity;
use interface_types::proto::user::User as ProtoUser;
use interface_types::proto::user::UserResponse;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use user_auth::db_exchange::User as AuthUser;
use user_auth::db_exchange::user2token;
use user_auth::wx_auth::*;

use crate::AppState;

#[derive(Deserialize)]
struct LoginQuery {
    /// For demo we accept `openid` query param; typically this would be `js_code`.
    js_code: String,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/login", get(login))
}

async fn login(
    State(state): State<AppState>,
    Query(query): Query<LoginQuery>,
) -> Protobuf<UserResponse> {
    // Use wx_auth to resolve the provided token/code into an openid.
    let wx_result = wx_auth_session_to_json(&query.js_code).await;

    let openid = match wx_result {
        Ok(resp) => match resp.openid {
            Some(oid) => oid,
            None => {
                return Protobuf(UserResponse {
                    user: None,
                    code: 400,
                    message: "WeiXin auth did not return an openid".to_string(),
                });
            }
        },
        Err(err) => {
            return Protobuf(UserResponse {
                user: None,
                code: 500,
                message: format!("failed to resolve openid: {:?}", err),
            });
        }
    };

    // Insert or update the user in the database (currently only insert is implemented).
    let queryed_user = match query_user_in_db(&state, &openid).await {
        Ok(u) => Some(u),
        Err(err) => {
            return Protobuf(UserResponse {
                user: None,
                code: 500,
                message: err,
            });
        }
    };

    let user = match queryed_user {
        Some(u) => Some(u),
        None => {
            return Protobuf(UserResponse {
                user: None,
                code: 500,
                message: "User not found".to_string(),
            });
        }
    };

    Protobuf(UserResponse {
        user: user,
        code: 200,
        message: "login success".to_string(),
    })
}

async fn query_user_in_db(state: &AppState, openid: &str) -> Result<ProtoUser, String> {
    let db = state.database.clone();

    let user_queryed_result = user_entity::Entity::find()
        .filter(user_entity::Column::OpenId.eq(openid))
        .one(db.as_ref())
        .await
        .unwrap();
    if user_queryed_result == None {
        return Err("User not found".to_string());
    }
    let model = user_queryed_result.unwrap();
    let user = AuthUser {
        open_id: model.open_id,
        nickname: model.nickname.clone(),
        name: model.name.clone(),
        phone_number: model.phone_number.clone(),
        address: model.address.clone(),
        is_important: model.is_important,
        avatar: model.avatar.clone(),
        permission: model.permission,
    };
    let jwt_token = user2token(&user).unwrap();

    Ok(ProtoUser {
        token: Some(jwt_token),
        nickname: model.nickname,
        name: model.name,
        phone_number: model.phone_number,
        address: model.address,
        is_important: model.is_important.map(|b| b.to_string()),
        avatar: model.avatar,
        permission: model.permission.map(|p| p.to_string()),
    })
}
