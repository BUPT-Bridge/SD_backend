use axum::{
    Router,
    extract::{Query, State},
    routing::get,
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::user as user_entity;
use interface_types::proto::user::User as ProtoUser;
use interface_types::proto::user::UserResponse;
use sea_orm::{ActiveModelTrait, Set};
use serde::Deserialize;
use user_auth::db_exchange::User as AuthUser;
use user_auth::db_exchange::user2token;
use user_auth::wx_auth::*;

use crate::AppState;

#[derive(Deserialize)]
struct RegisterQuery {
    /// For demo we accept `openid` query param; typically this would be `js_code`.
    js_code: String,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/register", get(register))
}

async fn register(
    State(state): State<AppState>,
    Query(query): Query<RegisterQuery>,
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
    let created_user = match add_user_to_db(&state, &openid).await {
        Ok(u) => Some(u),
        Err(err) => {
            return Protobuf(UserResponse {
                user: None,
                code: 500,
                message: err,
            });
        }
    };
    Protobuf(UserResponse {
        user: created_user,
        code: 200,
        message: "register success".to_string(),
    })
}

async fn add_user_to_db(state: &AppState, openid: &str) -> Result<ProtoUser, String> {
    let db = state.database.clone();

    let active = user_entity::ActiveModel {
        open_id: Set(openid.to_string()),
        ..Default::default()
    };

    let model = active
        .insert(db.as_ref())
        .await
        .map_err(|e| e.to_string())?;

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
