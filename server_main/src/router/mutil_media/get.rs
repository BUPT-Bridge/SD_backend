use axum::{
    Router,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
    routing::get,
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::mutil_media as mutil_media_entity;
use interface_types::proto::mutil_media::{Media as ProtoMedia, MediaResponse};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use user_auth::db_exchange::{ExchangeError, token2user};
use uuid::Uuid;

use crate::AppState;

/// 获取多媒体文件的查询参数
#[derive(Debug, Deserialize)]
struct MediaQuery {
    /// 多媒体文件的 UUID
    uuid: String,
}

/// 创建 mutil_media 的 GET 路由
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/metadata", get(get_media_metadata))
        .route("/download", get(get_media_download))
}

/// GET /api/mutil_media/metadata?uuid=xxx
///
/// 获取多媒体文件的元数据，返回 MediaResponse（protobuf 格式）
/// 用于关联表查询，获取 UUID 和类型
///
/// Headers:
/// - Authorization: Bearer token（必需，权限 0-3 均可）
///
/// 查询参数：
/// - uuid: 必需，多媒体文件的 UUID
///
/// 示例：GET /api/mutil_media/metadata?uuid=xxx
async fn get_media_metadata(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<MediaQuery>,
) -> Protobuf<MediaResponse> {
    // 1) 从 Header 提取并验证 token
    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Protobuf(MediaResponse {
                    media: None,
                    code: 401,
                    message: "Invalid token format".to_string(),
                });
            }
        },
        None => {
            return Protobuf(MediaResponse {
                media: None,
                code: 401,
                message: "Missing token".to_string(),
            });
        }
    };

    // 2) 解析 token，获取用户信息
    match token2user(&token) {
        Ok(_) => {
            // Token 验证成功，权限 0-3 均可访问，继续执行查询逻辑
        }
        Err(err) => {
            let msg = match err {
                ExchangeError::InvalidToken => "Invalid token".to_string(),
                ExchangeError::TokenExpired => "Token expired".to_string(),
                ExchangeError::TokenGenerationError(e) | ExchangeError::OtherError(e) => e,
            };
            return Protobuf(MediaResponse {
                media: None,
                code: 401,
                message: msg,
            });
        }
    }

    let db = state.database.clone();

    // 3. 解析 UUID
    let uuid = match Uuid::parse_str(&params.uuid) {
        Ok(u) => u,
        Err(_) => {
            return Protobuf(MediaResponse {
                media: None,
                code: 400,
                message: "Invalid UUID format".to_string(),
            });
        }
    };

    // 4. 查询数据库（通过 UUID 查找，不是通过主键 ID）
    let media = match mutil_media_entity::Entity::find()
        .filter(mutil_media_entity::Column::Uuid.eq(uuid))
        .one(db.as_ref())
        .await
    {
        Ok(Some(m)) => m,
        Ok(None) => {
            return Protobuf(MediaResponse {
                media: None,
                code: 404,
                message: "Media not found".to_string(),
            });
        }
        Err(err) => {
            return Protobuf(MediaResponse {
                media: None,
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    // 5. 返回元数据（MediaResponse，protobuf 格式）
    Protobuf(MediaResponse {
        media: Some(ProtoMedia {
            uuid: media.uuid.map(|u| u.to_string()).unwrap_or_default(),
            r#type: media.r#type.unwrap_or_default(),
        }),
        code: 200,
        message: "Get media metadata success".to_string(),
    })
}

/// GET /api/mutil_media/download?uuid=xxx
///
/// 获取多媒体文件的二进制数据，直接返回文件内容和正确的 Content-Type
///
/// Headers:
/// - Authorization: Bearer token（必需，权限 0-3 均可）
///
/// 查询参数：
/// - uuid: 必需，多媒体文件的 UUID
///
/// 示例：GET /api/mutil_media/download?uuid=xxx
async fn get_media_download(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<MediaQuery>,
) -> impl IntoResponse {
    // 1) 从 Header 提取并验证 token
    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return (StatusCode::UNAUTHORIZED, "Invalid token format").into_response();
            }
        },
        None => {
            return (StatusCode::UNAUTHORIZED, "Missing token").into_response();
        }
    };

    // 2) 解析 token，获取用户信息
    if let Err(err) = token2user(&token) {
        let msg = match err {
            ExchangeError::InvalidToken => "Invalid token".to_string(),
            ExchangeError::TokenExpired => "Token expired".to_string(),
            ExchangeError::TokenGenerationError(e) | ExchangeError::OtherError(e) => e,
        };
        return (StatusCode::UNAUTHORIZED, msg).into_response();
    }

    let db = state.database.clone();

    // 3. 解析 UUID
    let uuid = match Uuid::parse_str(&params.uuid) {
        Ok(u) => u,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "Invalid UUID format").into_response();
        }
    };

    // 4. 查询数据库（通过 UUID 查找，不是通过主键 ID）
    let media = match mutil_media_entity::Entity::find()
        .filter(mutil_media_entity::Column::Uuid.eq(uuid))
        .one(db.as_ref())
        .await
    {
        Ok(Some(m)) => m,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, "Media not found").into_response();
        }
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", err),
            )
                .into_response();
        }
    };

    // 5. 提取文件类型和文件数据
    let media_type = media
        .r#type
        .unwrap_or("application/octet-stream".to_string());
    let file_data = media.file.unwrap_or_default();

    // 6. 根据 type 构建正确的 MIME 类型
    let content_type = determine_mime_type(&media_type);

    // 5. 构建响应
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, content_type.parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", uuid)
            .parse()
            .unwrap(),
    );

    // 7. 构建并返回响应
    (headers, file_data).into_response()
}

/// 根据文件扩展名确定 MIME 类型
fn determine_mime_type(file_type: &str) -> &'static str {
    let file_type_lower = file_type.to_lowercase();
    match file_type_lower.as_str() {
        // 图片类型
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",

        // 视频类型
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mov" => "video/quicktime",
        "avi" => "video/x-msvideo",
        "mkv" => "video/x-matroska",
        "flv" => "video/x-flv",

        // 音频类型
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "flac" => "audio/flac",
        "aac" => "audio/aac",
        "m4a" => "audio/mp4",

        // 文档类型
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "txt" => "text/plain",
        "json" => "application/json",
        "xml" => "application/xml",

        // 压缩文件
        "zip" => "application/zip",
        "rar" => "application/vnd.rar",
        "7z" => "application/x-7z-compressed",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",

        // 其他
        _ => "application/octet-stream",
    }
}
