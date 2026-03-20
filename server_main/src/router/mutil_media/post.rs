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
use std::path::PathBuf;
use tokio::{fs, io::AsyncWriteExt};
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
    /// 是否为大文件上传（保存到本地文件系统）
    #[serde(default)]
    bigfile: bool,
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
        Ok(_) => {}
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

    // 3) 生成 UUID
    let uuid = Uuid::new_v4();

    // 4) 从 multipart 中提取文件数据和文件名
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

                // 判断是否需要流式上传
                if params.bigfile && !params.compress && !params.avatar {
                    // 大文件流式上传：直接写入磁盘
                    let media_type =
                        extract_file_type(filename.as_ref().unwrap_or(&"unknown".to_string()));

                    match save_stream_to_file(&uuid, field, &media_type).await {
                        Ok(_) => {
                            // 文件保存成功，插入数据库记录
                            let db = state.database.clone();
                            let new_media = mutil_media_entity::ActiveModel {
                                uuid: sea_orm::Set(Some(uuid)),
                                file: sea_orm::Set(None),
                                r#type: sea_orm::Set(Some(media_type.clone())),
                                ..Default::default()
                            };

                            match new_media.insert(db.as_ref()).await {
                                Ok(inserted_media) => {
                                    return Json(JsonMediaResponse {
                                        media: Some(JsonMedia {
                                            uuid: inserted_media
                                                .uuid
                                                .map(|u| u.to_string())
                                                .unwrap_or_default(),
                                            r#type: inserted_media.r#type.unwrap_or_default(),
                                        }),
                                        code: 200,
                                        message: "Upload media success (bigfile streaming)"
                                            .to_string(),
                                    });
                                }
                                Err(err) => {
                                    return Json(JsonMediaResponse {
                                        media: None,
                                        code: 500,
                                        message: format!("Failed to insert media record: {}", err),
                                    });
                                }
                            }
                        }
                        Err(err) => {
                            return Json(JsonMediaResponse {
                                media: None,
                                code: 500,
                                message: format!("Failed to save file to filesystem: {}", err),
                            });
                        }
                    }
                } else {
                    // 小文件或需要压缩：读取到内存
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

    // 5) 使用文件名
    let filename = filename.unwrap_or_else(|| "unknown".to_string());

    // 6) 验证文件数据存在（小文件或需要压缩的情况）
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

    // 7) 处理图片（如果启用了 compress 或 avatar 参数）
    let (processed_data, processed_filename) = if params.avatar {
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
        (file_data, filename)
    };

    // 8) 更新媒体类型
    let final_media_type = extract_file_type(&processed_filename);

    // 9) 根据 bigfile 参数决定存储方式
    if params.bigfile {
        // 大文件模式：保存到本地文件系统
        match save_to_local_filesystem(&uuid, &processed_data, &final_media_type).await {
            Ok(_) => {
                let db = state.database.clone();
                let new_media = mutil_media_entity::ActiveModel {
                    uuid: sea_orm::Set(Some(uuid)),
                    file: sea_orm::Set(None),
                    r#type: sea_orm::Set(Some(final_media_type.clone())),
                    ..Default::default()
                };

                match new_media.insert(db.as_ref()).await {
                    Ok(inserted_media) => Json(JsonMediaResponse {
                        media: Some(JsonMedia {
                            uuid: inserted_media
                                .uuid
                                .map(|u| u.to_string())
                                .unwrap_or_default(),
                            r#type: inserted_media.r#type.unwrap_or_default(),
                        }),
                        code: 200,
                        message: "Upload media success (bigfile)".to_string(),
                    }),
                    Err(err) => Json(JsonMediaResponse {
                        media: None,
                        code: 500,
                        message: format!("Failed to insert media record: {}", err),
                    }),
                }
            }
            Err(err) => Json(JsonMediaResponse {
                media: None,
                code: 500,
                message: format!("Failed to save file to filesystem: {}", err),
            }),
        }
    } else {
        // 普通模式：保存到数据库
        let db = state.database.clone();
        let new_media = mutil_media_entity::ActiveModel {
            uuid: sea_orm::Set(Some(uuid)),
            file: sea_orm::Set(Some(processed_data)),
            r#type: sea_orm::Set(Some(final_media_type.clone())),
            ..Default::default()
        };

        match new_media.insert(db.as_ref()).await {
            Ok(inserted_media) => Json(JsonMediaResponse {
                media: Some(JsonMedia {
                    uuid: inserted_media
                        .uuid
                        .map(|u| u.to_string())
                        .unwrap_or_default(),
                    r#type: inserted_media.r#type.unwrap_or_default(),
                }),
                code: 200,
                message: "Upload media success".to_string(),
            }),
            Err(err) => Json(JsonMediaResponse {
                media: None,
                code: 500,
                message: format!("Failed to upload media: {}", err),
            }),
        }
    }
}

/// 流式保存文件到本地文件系统
async fn save_stream_to_file(
    uuid: &Uuid,
    mut field: axum::extract::multipart::Field<'_>,
    media_type: &str,
) -> Result<(), String> {
    let media_dir = PathBuf::from("media");

    fs::create_dir_all(&media_dir)
        .await
        .map_err(|err| format!("Failed to create media directory: {}", err))?;

    let filename = format!("{}.{}", uuid, media_type);
    let file_path = media_dir.join(&filename);

    let mut file = fs::File::create(&file_path)
        .await
        .map_err(|err| format!("Failed to create file: {}", err))?;

    while let Some(chunk) = field
        .chunk()
        .await
        .map_err(|err| format!("Failed to read chunk: {}", err))?
    {
        file.write_all(&chunk)
            .await
            .map_err(|err| format!("Failed to write chunk: {}", err))?;
    }

    file.flush()
        .await
        .map_err(|err| format!("Failed to flush file: {}", err))?;

    Ok(())
}

/// 将文件保存到本地文件系统
async fn save_to_local_filesystem(
    uuid: &Uuid,
    data: &[u8],
    media_type: &str,
) -> Result<(), String> {
    let media_dir = PathBuf::from("media");

    fs::create_dir_all(&media_dir)
        .await
        .map_err(|err| format!("Failed to create media directory: {}", err))?;

    let filename = format!("{}.{}", uuid, media_type);
    let file_path = media_dir.join(&filename);

    fs::write(&file_path, data)
        .await
        .map_err(|err| format!("Failed to write file: {}", err))?;

    Ok(())
}
