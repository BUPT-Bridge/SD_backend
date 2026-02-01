pub mod apply_permission;
pub mod login;
pub mod modify;
pub mod register;

pub use apply_permission::router as apply_permission_router;
pub use login::router as login_router;
pub use modify::router as modify_router;
pub use register::router as register_router;
