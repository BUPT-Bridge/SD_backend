use axum::{
    Router,
    body::Body,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::mutil_media as mutil_media_entity;
use interface_types::proto::mutil_media::{Media as ProtoMedia, MediaResponse};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use std::path::PathBuf;
use tokio::io::AsyncSeekExt;
use user_auth::db_exchange::{ExchangeError, token2user};
use uuid::Uuid;

use crate::AppState;

/// 获取多媒体文件的查询参数
#[derive(Debug, Deserialize)]
struct MediaQuery {
    /// 多媒体文件的 UUID
    uuid: String,
    /// 是否为大文件下载（从本地文件系统读取）
    #[serde(default)]
    bigfile: bool,
}

/// 解析 Range 请求头
/// 支持格式: bytes=start-end, bytes=start-, bytes=-end
fn parse_range_header(range_header: &str, file_size: u64) -> Option<(u64, u64)> {
    if range_header.contains(',') {
        // 暂不支持多段 Range 请求
        return None;
    }

    // 期望格式: bytes=start-end
    let range_part = range_header.strip_prefix("bytes=")?;

    if let Some((start_str, end_str)) = range_part.split_once('-') {
        let start: u64 = if start_str.is_empty() {
            // 格式: bytes=-500 (最后500字节)
            let suffix_len: u64 = end_str.parse().ok()?;
            file_size.saturating_sub(suffix_len)
        } else {
            start_str.parse().ok()?
        };

        let end: u64 = if end_str.is_empty() {
            // 格式: bytes=0- (从开始到结束)
            file_size.saturating_sub(1)
        } else {
            let end: u64 = end_str.parse().ok()?;
            end.min(file_size.saturating_sub(1))
        };

        if start <= end && start < file_size {
            Some((start, end))
        } else {
            None
        }
    } else {
        None
    }
}

/// 检查是否为视频文件
fn is_video_file(media_type: &str) -> bool {
    let media_type_lower = media_type.to_lowercase();
    matches!(
        media_type_lower.as_str(),
        "mp4" | "webm" | "mov" | "avi" | "mkv" | "flv" | "m4v" | "3gp"
    )
}

/// 验证 token，返回错误响应或 Ok(())
fn verify_token(headers: &HeaderMap) -> Result<(), Response> {
    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Err((StatusCode::UNAUTHORIZED, "Invalid token format").into_response());
            }
        },
        None => {
            return Err((StatusCode::UNAUTHORIZED, "Missing token").into_response());
        }
    };

    if let Err(err) = token2user(&token) {
        let msg = match err {
            ExchangeError::InvalidToken => "Invalid token".to_string(),
            ExchangeError::TokenExpired => "Token expired".to_string(),
            ExchangeError::TokenGenerationError(e) | ExchangeError::OtherError(e) => e,
        };
        return Err((StatusCode::UNAUTHORIZED, msg).into_response());
    }

    Ok(())
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
/// - Authorization: Bearer token（可选，携带时会验证）
///
/// 查询参数：
/// - uuid: 必需，多媒体文件的 UUID
///
/// 示例：GET /api/mutil_media/metadata?uuid=xxx
async fn get_media_metadata(
    State(state): State<AppState>,
    _headers: HeaderMap,
    Query(params): Query<MediaQuery>,
) -> Protobuf<MediaResponse> {
    let db = state.database.clone();

    // 1. 解析 UUID
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

    // 2. 查询数据库（通过 UUID 查找，不是通过主键 ID）
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

    // 3. 返回元数据（MediaResponse，protobuf 格式）
    Protobuf(MediaResponse {
        media: Some(ProtoMedia {
            uuid: media.uuid.map(|u| u.to_string()).unwrap_or_default(),
            r#type: media.r#type.unwrap_or_default(),
        }),
        code: 200,
        message: "Get media metadata success".to_string(),
    })
}

/// GET /api/mutil_media/download?uuid=xxx&bigfile=false
///
/// 获取多媒体文件的二进制数据，直接返回文件内容和正确的 Content-Type
///
/// Headers:
/// - Authorization: Bearer token（bigfile=true 时必需，bigfile=false 时可选）
/// - Range: 可选，支持分块下载视频文件，格式: bytes=start-end
///
/// 查询参数：
/// - uuid: 必需，多媒体文件的 UUID
/// - bigfile: 是否为大文件下载（可选，默认 false）
///   - false: 从数据库读取文件（兼容原有逻辑，不需要 token）
///   - true: 从本地 media 文件夹读取文件（需要 token），视频文件支持分块下载
///
/// 示例：
/// - GET /api/mutil_media/download?uuid=xxx&bigfile=false（无需 token）
/// - GET /api/mutil_media/download?uuid=xxx&bigfile=true（需要 token）
/// - GET /api/mutil_media/download?uuid=xxx&bigfile=true (with Range: bytes=0-1023)
async fn get_media_download(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<MediaQuery>,
) -> Response {
    // 1) bigfile=true 时需要验证 token
    if params.bigfile {
        if let Err(resp) = verify_token(&headers) {
            return resp;
        }
    }

    let db = state.database.clone();

    // 2. 解析 UUID
    let uuid = match Uuid::parse_str(&params.uuid) {
        Ok(u) => u,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "Invalid UUID format").into_response();
        }
    };

    // 3. 查询数据库（通过 UUID 查找，不是通过主键 ID）
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

    // 4. 提取文件类型
    let media_type = media
        .r#type
        .unwrap_or("application/octet-stream".to_string());

    // 5. 根据 bigfile 参数决定文件来源
    if params.bigfile {
        // 大文件模式：从本地文件系统读取
        let filename = format!("{}.{}", uuid, media_type);
        let file_path = PathBuf::from("media").join(&filename);

        // 检查文件是否存在
        if !file_path.exists() {
            return (
                StatusCode::NOT_FOUND,
                format!("File not found: {}", file_path.display()),
            )
                .into_response();
        }

        // 获取文件元数据
        let metadata = match tokio::fs::metadata(&file_path).await {
            Ok(m) => m,
            Err(err) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to read file metadata: {}", err),
                )
                    .into_response();
            }
        };
        let file_size = metadata.len();

        // 6. 根据 type 构建正确的 MIME 类型
        let content_type = determine_mime_type(&media_type);

        // 7. 处理 Range 请求（标准 HTTP Partial Content）
        let range_header = headers.get(header::RANGE).and_then(|h| h.to_str().ok());

        if let Some(range_str) = range_header {
            match parse_range_header(range_str, file_size) {
                Some((start, end)) => {
                    return stream_file_chunk(
                        file_path,
                        start,
                        end,
                        file_size,
                        content_type,
                        is_video_file(&media_type),
                    )
                    .await;
                }
                None => {
                    // Range 格式错误，返回 416 Range Not Satisfiable
                    let mut resp_headers = HeaderMap::new();
                    resp_headers.insert(
                        header::CONTENT_RANGE,
                        format!("bytes */{}", file_size).parse().unwrap(),
                    );
                    return (StatusCode::RANGE_NOT_SATISFIABLE, resp_headers).into_response();
                }
            }
        }

        // 普通下载模式（整文件，流式返回，避免大文件一次性读入内存）
        stream_full_file(
            file_path,
            file_size,
            content_type,
            is_video_file(&media_type),
            &uuid,
            &media_type,
        )
        .await
    } else {
        // 普通模式：从数据库读取（兼容原有逻辑，不需要 token）
        let file_data = media.file.unwrap_or_default();

        // 6. 根据 type 构建正确的 MIME 类型
        let content_type = determine_mime_type(&media_type);

        // 7. 构建响应
        let mut resp_headers = HeaderMap::new();
        resp_headers.insert(header::CONTENT_TYPE, content_type.parse().unwrap());
        resp_headers.insert(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", uuid)
                .parse()
                .unwrap(),
        );

        // 8. 构建并返回响应
        (resp_headers, file_data).into_response()
    }
}

/// 流式传输文件分块（用于视频分块下载）
async fn stream_file_chunk(
    file_path: PathBuf,
    start: u64,
    end: u64,
    file_size: u64,
    content_type: &str,
    is_video: bool,
) -> Response {
    use tokio::io::AsyncReadExt;
    use tokio_util::io::ReaderStream;

    // 打开文件
    let mut file = match tokio::fs::File::open(&file_path).await {
        Ok(f) => f,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to open file: {}", err),
            )
                .into_response();
        }
    };

    // 移动到起始位置
    if let Err(err) = file.seek(tokio::io::SeekFrom::Start(start)).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to seek file: {}", err),
        )
            .into_response();
    }

    // 计算需要读取的字节数
    let content_length = end - start + 1;

    // 创建有限读取的流
    let limited_reader = file.take(content_length);
    let stream = ReaderStream::new(limited_reader);
    let body = Body::from_stream(stream);

    // 构建响应头
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, content_type.parse().unwrap());
    headers.insert(
        header::CONTENT_RANGE,
        format!("bytes {}-{}/{}", start, end, file_size)
            .parse()
            .unwrap(),
    );
    headers.insert(
        header::CONTENT_LENGTH,
        content_length.to_string().parse().unwrap(),
    );
    headers.insert(header::ACCEPT_RANGES, "bytes".parse().unwrap());
    headers.insert(
        header::CACHE_CONTROL,
        "public, max-age=31536000".parse().unwrap(),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        if is_video {
            "inline".parse().unwrap()
        } else {
            "attachment".parse().unwrap()
        },
    );

    // 返回 206 Partial Content
    (StatusCode::PARTIAL_CONTENT, headers, body).into_response()
}

/// 流式传输完整文件，避免大文件占用过多内存
async fn stream_full_file(
    file_path: PathBuf,
    file_size: u64,
    content_type: &str,
    is_video: bool,
    uuid: &Uuid,
    media_type: &str,
) -> Response {
    use tokio_util::io::ReaderStream;

    let file = match tokio::fs::File::open(&file_path).await {
        Ok(f) => f,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to open file: {}", err),
            )
                .into_response();
        }
    };

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, content_type.parse().unwrap());
    headers.insert(
        header::CONTENT_LENGTH,
        file_size.to_string().parse().unwrap(),
    );
    headers.insert(header::ACCEPT_RANGES, "bytes".parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        if is_video {
            "inline".parse().unwrap()
        } else {
            format!("attachment; filename=\"{}.{}\"", uuid, media_type)
                .parse()
                .unwrap()
        },
    );

    (headers, body).into_response()
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
