pub mod user {
    include!(concat!(env!("OUT_DIR"), "/sd_backend.user.rs"));
}
pub mod notice {
    include!(concat!(env!("OUT_DIR"), "/sd_backend.notice.rs"));
}

pub mod slideshow {
    include!(concat!(env!("OUT_DIR"), "/sd_backend.slideshow.rs"));
}
