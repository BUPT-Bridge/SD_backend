use axum::{
    Json, Router,
    extract::{Multipart, Query, State},
    http::HeaderMap,
    response::IntoResponse,
    routing::post,
};
use db_manager::entity::mutil_media as mutil_media_entity;
use sea_orm::ActiveModelTrait;
use serde::{Deserialize, Serialize};
use user_auth::db_exchange::{ExchangeError, token2user};
use uuid::Uuid;

use crate::AppState;

use super::utils::{compress_to_webp, extract_file_type, process_avatar};

/// 创建 mutil_media 的 POST 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", post(upload_media))
}

/// 上传参数
#[derive(Debug, Deserialize)]
struct UploadParams {
    /// 是否压缩为 webp 格式
    #[serde(default)]
    compress: bool,
    /// 是否作为头像上传（自动压缩为 webp 并裁剪为 120x120）
    #[serde(default)]
    avatar: bool,
}

/// JSON 响应结构
#[derive(Serialize)]
struct JsonMediaResponse {
    /// 媒体信息
    media: Option<JsonMedia>,
    /// 状态码
    code: i32,
    /// 响应消息
    message: String,
}

/// JSON 媒体信息
#[derive(Serialize)]
struct JsonMedia {
    /// UUID
    uuid: String,
    /// 文件类型
    r#type: String,
}

/// POST /api/mutil_media
///
/// 上传多媒体文件（multipart/form-data 格式）
///
/// Headers:
/// - Authorization: Bearer token（必需，权限 0-3 均可）
///
/// Query 参数：
/// - compress: 是否压缩为 webp 格式（可选，默认 false）
/// - avatar: 是否作为头像上传（可选，默认 false，自动压缩为 webp 并裁剪为 120x120）
///
/// 请求体（multipart/form-data）：
/// - file: 文件数据（必需）
/// - filename: 文件名（可选，默认使用原始文件名）
///
/// 返回（JSON 格式）：
/// ```json
/// {
///   "media": {
///     "uuid": "xxx",
///     "type": "webp"
///   },
///   "code": 200,
///   "message": "Upload media success"
/// }
/// ```
///
/// 注意：
/// - 需要有效的 token（权限 0-3 均可）
/// - 使用 multipart/form-data 格式上传文件
/// - compress 参数：压缩图片为 webp 格式，后缀名改为 .webp
/// - avatar 参数：压缩为 webp 后裁剪为 120x120 像素
/// - 支持 jpg、png、jpeg、gif 等常见图片格式
/// - 非图片文件不受 compress/avatar 参数影响
async fn upload_media(
    State(state): State<AppState>,
    Query(params): Query<UploadParams>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> impl IntoResponse {
    // 1) 从 Header 提取并验证 token
    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Json(JsonMediaResponse {
                    media: None,
                    code: 401,
                    message: "Invalid token format".to_string(),
                });
            }
        },
        None => {
            return Json(JsonMediaResponse {
                media: None,
                code: 401,
                message: "Missing token".to_string(),
            });
        }
    };

    // 2) 解析 token，获取用户信息
    match token2user(&token) {
        Ok(_) => {
            // Token 验证成功，权限 0-3 均可访问，继续执行上传逻辑
        }
        Err(err) => {
            let msg = match err {
                ExchangeError::InvalidToken => "Invalid token".to_string(),
                ExchangeError::TokenExpired => "Token expired".to_string(),
                ExchangeError::TokenGenerationError(e) | ExchangeError::OtherError(e) => e,
            };
            return Json(JsonMediaResponse {
                media: None,
                code: 401,
                message: msg,
            });
        }
    };

    // 3) 从 multipart 中提取文件数据和文件名
    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "file" => {
                // 获取原始文件名（如果存在）
                if filename.is_none() {
                    if let Some(original_name) = field.file_name() {
                        filename = Some(original_name.to_string());
                    }
                }
                // 读取文件数据
                match field.bytes().await {
                    Ok(bytes) => {
                        file_data = Some(bytes.to_vec());
                    }
                    Err(err) => {
                        return Json(JsonMediaResponse {
                            media: None,
                            code: 400,
                            message: format!("Failed to read file data: {}", err),
                        });
                    }
                }
            }
            "filename" => {
                // 读取文件名字段（可选）
                match field.text().await {
                    Ok(name) => {
                        filename = Some(name);
                    }
                    Err(err) => {
                        return Json(JsonMediaResponse {
                            media: None,
                            code: 400,
                            message: format!("Failed to read filename: {}", err),
                        });
                    }
                }
            }
            _ => {
                // 忽略其他字段
            }
        }
    }

    // 4) 验证文件数据存在
    let file_data = match file_data {
        Some(data) => data,
        None => {
            return Json(JsonMediaResponse {
                media: None,
                code: 400,
                message: "No file data provided".to_string(),
            });
        }
    };

    // 5) 使用文件名（优先使用 filename 字段，否则使用文件的原始文件名）
    let filename = filename.unwrap_or_else(|| {
        // 从 multipart 字段中获取原始文件名
        "unknown".to_string()
    });

    // 6) 处理图片（如果启用了 compress 或 avatar 参数）
    let (processed_data, processed_filename) = if params.avatar {
        // 头像模式：压缩为 webp 并裁剪为 120x120
        match process_avatar(&file_data, &filename) {
            Ok((data, name)) => (data, name),
            Err(err) => {
                return Json(JsonMediaResponse {
                    media: None,
                    code: 400,
                    message: format!("Failed to process avatar: {}", err),
                });
            }
        }
    } else if params.compress {
        // 压缩模式：转换为 webp 格式
        match compress_to_webp(&file_data, &filename) {
            Ok((data, name)) => (data, name),
            Err(err) => {
                return Json(JsonMediaResponse {
                    media: None,
                    code: 400,
                    message: format!("Failed to compress image: {}", err),
                });
            }
        }
    } else {
        // 不处理，使用原始数据
        (file_data, filename)
    };

    // 7) 从文件名提取文件类型（后缀）
    let media_type = extract_file_type(&processed_filename);

    // 8) 生成 UUID
    let uuid = Uuid::new_v4();

    // 9) 创建 ActiveModel 并插入数据库
    let db = state.database.clone();
    let new_media = mutil_media_entity::ActiveModel {
        uuid: sea_orm::Set(Some(uuid)),
        file: sea_orm::Set(Some(processed_data)),
        r#type: sea_orm::Set(Some(media_type.clone())),
        ..Default::default()
    };

    // 10) 执行插入操作
    match new_media.insert(db.as_ref()).await {
        Ok(inserted_media) => {
            // 插入成功，返回 JSON 响应
            Json(JsonMediaResponse {
                media: Some(JsonMedia {
                    uuid: inserted_media
                        .uuid
                        .map(|u| u.to_string())
                        .unwrap_or_default(),
                    r#type: inserted_media.r#type.unwrap_or_default(),
                }),
                code: 200,
                message: "Upload media success".to_string(),
            })
        }
        Err(err) => {
            // 插入失败，返回错误响应
            Json(JsonMediaResponse {
                media: None,
                code: 500,
                message: format!("Failed to upload media: {}", err),
            })
        }
    }
}
