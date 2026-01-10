use user_auth::user_auth::{authorize::*, r#struct::*};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_greater_than() {
        let admin = UserPermissionLevel::Admin;
        let provider = UserPermissionLevel::Provider;
        let user = UserPermissionLevel::User;
        let guest = UserPermissionLevel::Guest;

        // 测试 Admin 是最高权限
        assert!(admin > provider);
        assert!(admin > user);
        assert!(admin > guest);

        // 测试 Provider 权限
        assert!(provider > user);
        assert!(provider > guest);
        assert!(provider < admin);

        // 测试 User 权限
        assert!(user > guest);
        assert!(user < provider);
        assert!(user < admin);

        // 测试 Guest 是最低权限
        assert!(guest < user);
        assert!(guest < provider);
        assert!(guest < admin);
    }

    #[test]
    fn test_permission_less_than() {
        let admin = UserPermissionLevel::Admin;
        let provider = UserPermissionLevel::Provider;
        let user = UserPermissionLevel::User;
        let guest = UserPermissionLevel::Guest;

        assert!(guest < user);
        assert!(user < provider);
        assert!(provider < admin);
    }

    #[test]
    fn test_permission_greater_equal() {
        let admin = UserPermissionLevel::Admin;
        let admin2 = UserPermissionLevel::Admin;
        let user = UserPermissionLevel::User;

        assert!(admin >= admin2);
        assert!(admin >= user);
        assert!(user >= user.clone());
    }

    #[test]
    fn test_permission_less_equal() {
        let guest = UserPermissionLevel::Guest;
        let guest2 = UserPermissionLevel::Guest;
        let admin = UserPermissionLevel::Admin;

        assert!(guest <= guest2);
        assert!(guest <= admin);
        assert!(admin <= admin.clone());
    }

    #[test]
    fn test_permission_equality() {
        let admin1 = UserPermissionLevel::Admin;
        let admin2 = UserPermissionLevel::Admin;
        let user1 = UserPermissionLevel::User;
        let user2 = UserPermissionLevel::User;

        assert_eq!(admin1, admin2);
        assert_eq!(user1, user2);
        assert_ne!(admin1, user1);
    }

    #[test]
    fn test_permission_sorting() {
        let admin = UserPermissionLevel::Admin;
        let provider = UserPermissionLevel::Provider;
        let user = UserPermissionLevel::User;
        let guest = UserPermissionLevel::Guest;

        let mut permissions = vec![admin.clone(), guest.clone(), provider.clone(), user.clone()];
        permissions.sort();

        // 排序后应该是从低到高：Guest, User, Provider, Admin
        assert_eq!(permissions[0], guest);
        assert_eq!(permissions[1], user);
        assert_eq!(permissions[2], provider);
        assert_eq!(permissions[3], admin);
    }

    #[tokio::test]
    async fn test_permission_authorization() {
        let admin = UserPermissionLevel::Admin; //3
        let provider = UserPermissionLevel::Provider; //2
        let user = UserPermissionLevel::User; //1
        let guest = UserPermissionLevel::Guest; //0

        // 测试权限代码足够时授权成功
        assert_eq!(
            authorize_user(3, admin.clone()).await,
            UserPermissionAuthorizeResult::Authorized
        );
        assert_eq!(
            authorize_user(2, provider.clone()).await,
            UserPermissionAuthorizeResult::Authorized
        );
        assert_eq!(
            authorize_user(1, user.clone()).await,
            UserPermissionAuthorizeResult::Authorized
        );
        assert_eq!(
            authorize_user(0, guest.clone()).await,
            UserPermissionAuthorizeResult::Authorized
        );

        // 测试权限代码不足时授权失败
        assert_eq!(
            authorize_user(0, user.clone()).await,
            UserPermissionAuthorizeResult::Unauthorized
        );
        assert_eq!(
            authorize_user(0, admin.clone()).await,
            UserPermissionAuthorizeResult::Unauthorized
        );
        assert_eq!(
            authorize_user(1, provider.clone()).await,
            UserPermissionAuthorizeResult::Unauthorized
        );

        // 测试边界情况
        assert_eq!(
            authorize_user(2, user.clone()).await,
            UserPermissionAuthorizeResult::Authorized
        );
        assert_eq!(
            authorize_user(10, admin.clone()).await,
            UserPermissionAuthorizeResult::Authorized
        );
    }

    #[tokio::test]
    async fn test_authorize_user_strict() {
        let admin = UserPermissionLevel::Admin; //3
        let provider = UserPermissionLevel::Provider; //2
        let user = UserPermissionLevel::User; //1
        let guest = UserPermissionLevel::Guest; //0

        // 测试权限代码相等时授权成功
        assert_eq!(
            authorize_user_strict(3, admin.clone()).await,
            UserPermissionAuthorizeResult::Authorized
        );
        assert_eq!(
            authorize_user_strict(2, provider.clone()).await,
            UserPermissionAuthorizeResult::Authorized
        );
        assert_eq!(
            authorize_user_strict(1, user.clone()).await,
            UserPermissionAuthorizeResult::Authorized
        );
        assert_eq!(
            authorize_user_strict(0, guest.clone()).await,
            UserPermissionAuthorizeResult::Authorized
        );

        // 测试权限代码不相等时授权失败
        assert_eq!(
            authorize_user_strict(0, user.clone()).await,
            UserPermissionAuthorizeResult::Unauthorized
        );
        assert_eq!(
            authorize_user_strict(0, admin.clone()).await,
            UserPermissionAuthorizeResult::Unauthorized
        );
        assert_eq!(
            authorize_user_strict(1, provider.clone()).await,
            UserPermissionAuthorizeResult::Unauthorized
        );

        // 测试边界情况
        assert_eq!(
            authorize_user_strict(2, user.clone()).await,
            UserPermissionAuthorizeResult::Unauthorized
        );
        assert_eq!(
            authorize_user_strict(10, admin.clone()).await,
            UserPermissionAuthorizeResult::Unauthorized
        );
    }
}
