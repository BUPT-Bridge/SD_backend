pub mod user {
    include!(concat!(env!("OUT_DIR"), "/sd_backend.user.rs"));
}
pub mod notice {
    include!(concat!(env!("OUT_DIR"), "/sd_backend.notice.rs"));
}
pub mod mutil_media {
    include!(concat!(env!("OUT_DIR"), "/sd_backend.mutil_media.rs"));
}

pub mod slideshow {
    include!(concat!(env!("OUT_DIR"), "/sd_backend.slideshow.rs"));
}
