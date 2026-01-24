use std::cmp::Ordering;
use std::convert::From;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserPermissionLevel {
    Admin,
    Provider,
    User,
    Guest,
}

impl UserPermissionLevel {
    // 返回权限级别的数值，数值越大权限越高
    pub fn level(&self) -> i32 {
        match self {
            UserPermissionLevel::Admin => 3,
            UserPermissionLevel::Provider => 2,
            UserPermissionLevel::User => 1,
            UserPermissionLevel::Guest => 0,
        }
    }
}

impl From<i32> for UserPermissionLevel {
    fn from(level: i32) -> Self {
        match level {
            3 => UserPermissionLevel::Admin,
            2 => UserPermissionLevel::Provider,
            1 => UserPermissionLevel::User,
            _ => UserPermissionLevel::Guest,
        }
    }
}

impl From<UserPermissionLevel> for i32 {
    fn from(level: UserPermissionLevel) -> Self {
        level.level()
    }
}

impl PartialOrd for UserPermissionLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UserPermissionLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        self.level().cmp(&other.level())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserPermissionAuthorizeResult {
    Authorized,
    Unauthorized,
}
