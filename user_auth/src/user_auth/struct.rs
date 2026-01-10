use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserPermissionLevel {
    Admin,
    Provider,
    User,
    Guest,
}

impl UserPermissionLevel {
    // 返回权限级别的数值，数值越大权限越高
    pub fn level(&self) -> u8 {
        match self {
            UserPermissionLevel::Admin => 3,
            UserPermissionLevel::Provider => 2,
            UserPermissionLevel::User => 1,
            UserPermissionLevel::Guest => 0,
        }
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
