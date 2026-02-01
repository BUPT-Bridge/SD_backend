use axum::{
    Router,
    body::Body,
    extract::State,
    http::{HeaderMap, Response, StatusCode, header},
    response::IntoResponse,
    routing::get,
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::feedback as feedback_entity;
use interface_types::proto::feedback::FeedbackExportRequest;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use user_auth::db_exchange::token2user;

use crate::AppState;

/// 创建 feedback 导出路由
pub fn router() -> Router<AppState> {
    Router::new().route("/export", get(export_feedback))
}

/// GET /api/feedback/export - 导出反馈（所有权限 0-3 都可以访问）
///
/// 传入 proto（FeedbackExportRequest），包含 start_time/end_time 时间戳
/// 返回 Excel 文件流
async fn export_feedback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Protobuf(payload): Protobuf<FeedbackExportRequest>,
) -> impl IntoResponse {
    // 1) 从 Header 提取 token
    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return StatusCode::UNAUTHORIZED.into_response();
            }
        },
        None => {
            return StatusCode::UNAUTHORIZED.into_response();
        }
    };

    // 2) 解析 token，获取用户信息
    let auth_user = match token2user(&token) {
        Ok(u) => u,
        Err(_err) => {
            return StatusCode::UNAUTHORIZED.into_response();
        }
    };

    // 3) 权限校验：所有权限 0-3 都可以访问
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission == 3 {
        return StatusCode::FORBIDDEN.into_response();
    }

    // 4) 时间戳校验
    let start_ts = payload.start_time;
    let end_ts = payload.end_time;
    if start_ts <= 0 || end_ts <= 0 || end_ts < start_ts {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let start_dt = match chrono::DateTime::from_timestamp(start_ts, 0) {
        Some(dt) => dt.with_timezone(&chrono::FixedOffset::east_opt(0).unwrap()),
        None => return StatusCode::BAD_REQUEST.into_response(),
    };
    let end_dt = match chrono::DateTime::from_timestamp(end_ts, 0) {
        Some(dt) => dt.with_timezone(&chrono::FixedOffset::east_opt(0).unwrap()),
        None => return StatusCode::BAD_REQUEST.into_response(),
    };

    // 5) 查询时间范围内的反馈
    let db = state.database.clone();
    let feedbacks = match feedback_entity::Entity::find()
        .filter(feedback_entity::Column::CreatedTime.between(start_dt, end_dt))
        .all(db.as_ref())
        .await
    {
        Ok(list) => list,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    // 6) 生成 Excel
    // 需要添加依赖：rust_xlsxwriter = "0.70"
    let mut workbook = rust_xlsxwriter::Workbook::new();
    let worksheet = workbook.add_worksheet();

    // 表头
    let _ = worksheet.write_string(0, 0, "序号");
    let _ = worksheet.write_string(0, 1, "反馈类型");
    let _ = worksheet.write_string(0, 2, "反馈内容");
    let _ = worksheet.write_string(0, 3, "联系方式（可选）");
    let _ = worksheet.write_string(0, 4, "提交时间");

    for (idx, item) in feedbacks.iter().enumerate() {
        let row = (idx + 1) as u32;
        let _ = worksheet.write_number(row, 0, (idx + 1) as f64);
        let _ = worksheet.write_string(row, 1, item.r#type.clone().unwrap_or_default());
        let _ = worksheet.write_string(row, 2, item.content.clone().unwrap_or_default());
        let _ = worksheet.write_string(row, 3, item.phone.clone().unwrap_or_default());
        let cst_dt = item
            .created_time
            .with_timezone(&chrono::FixedOffset::east_opt(8 * 3600).unwrap());
        let created_time_str = cst_dt.format("%Y-%m-%d %H:%M:%S").to_string();
        let _ = worksheet.write_string(row, 4, created_time_str);
    }

    let buffer = match workbook.save_to_buffer() {
        Ok(buf) => buf,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    // 7) 返回文件流
    let filename = format!("feedback_{}_{}.xlsx", start_ts, end_ts);
    let mut response = Response::new(Body::from(buffer));
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            .parse()
            .unwrap(),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", filename)
            .parse()
            .unwrap(),
    );
    response
}
