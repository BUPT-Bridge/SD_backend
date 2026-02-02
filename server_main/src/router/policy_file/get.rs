use axum::{
    Router,
    extract::{Query, State},
    routing::get,
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::policy_file as policy_file_entity;
use interface_types::proto::policy_file::{
    PolicyFile as ProtoPolicyFile, PolicyFileResponse,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;

use crate::AppState;

/// 创建 policy_file 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_policy_file))
}

/// 查询参数
#[derive(Debug, Deserialize)]
struct PolicyFileParams {
    /// 政策文件类型
    r#type: Option<String>,
}

/// GET /api/policy_file?type=xxx - 获取指定类型的政策文件（所有权限均可访问）
/// 必须提供 type 参数
async fn get_policy_file(
    State(state): State<AppState>,
    Query(params): Query<PolicyFileParams>,
) -> Protobuf<PolicyFileResponse> {
    // 1) 检查必填参数
    let file_type = match params.r#type {
        Some(v) if !v.is_empty() => v,
        _ => {
            return Protobuf(PolicyFileResponse {
                policy_files: vec![],
                code: 400,
                message: "Missing required parameter: type".to_string(),
            });
        }
    };

    // 2) 查询符合条件的政策文件
    let db = state.database.clone();
    let policy_files = match policy_file_entity::Entity::find()
        .filter(policy_file_entity::Column::Type.eq(file_type))
        .all(db.as_ref())
        .await
    {
        Ok(files) => files,
        Err(err) => {
            return Protobuf(PolicyFileResponse {
                policy_files: vec![],
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    // 3) 转换为 proto 格式
    let proto_files: Vec<ProtoPolicyFile> = policy_files
        .into_iter()
        .map(|f| ProtoPolicyFile {
            id: f.id,
            title: f.title.unwrap_or_default(),
            r#type: f.r#type.unwrap_or_default(),
            index: f.index.unwrap_or_default(),
            create_time: f.create_time.and_utc().timestamp(),
        })
        .collect();

    Protobuf(PolicyFileResponse {
        policy_files: proto_files,
        code: 200,
        message: "Get policy files success".to_string(),
    })
}
