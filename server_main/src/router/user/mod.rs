pub mod admin_manager;
pub mod apply_permission;
pub mod info;
pub mod login;
pub mod modify;
pub mod register;

pub use admin_manager::router as admin_manager_router;
pub use apply_permission::router as apply_permission_router;
pub use info::router as info_router;
pub use login::router as login_router;
pub use modify::router as modify_router;
pub use register::router as register_router;
