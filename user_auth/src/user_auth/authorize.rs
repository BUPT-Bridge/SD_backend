use super::r#struct::*;

///基本认证接口，用于判断用户是否不小于某权限。
pub async fn authorize_user(
    user_permission_code: u8,
    permission_level: UserPermissionLevel,
) -> UserPermissionAuthorizeResult {
    if user_permission_code >= permission_level.level() {
        UserPermissionAuthorizeResult::Authorized
    } else {
        UserPermissionAuthorizeResult::Unauthorized
    }
}

/// 严格认证接口，用于判断用户是否等于某权限。
pub async fn authorize_user_strict(
    user_permission_code: u8,
    permission_level: UserPermissionLevel,
) -> UserPermissionAuthorizeResult {
    if user_permission_code == permission_level.level() {
        UserPermissionAuthorizeResult::Authorized
    } else {
        UserPermissionAuthorizeResult::Unauthorized
    }
}
