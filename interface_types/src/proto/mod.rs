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

pub mod community_service {
    include!(concat!(env!("OUT_DIR"), "/sd_backend.community_service.rs"));
}

pub mod resource_service {
    include!(concat!(env!("OUT_DIR"), "/sd_backend.resource_service.rs"));
}

pub mod medical_service {
    include!(concat!(env!("OUT_DIR"), "/sd_backend.medical_service.rs"));
}

pub mod feedback {
    include!(concat!(env!("OUT_DIR"), "/sd_backend.feedback.rs"));
}

pub mod dinner_provider {
    include!(concat!(env!("OUT_DIR"), "/sd_backend.dinner_provider.rs"));
}

pub mod detail_meal {
    include!(concat!(env!("OUT_DIR"), "/sd_backend.detail_meal.rs"));
}

pub mod apply_permission {
    include!(concat!(env!("OUT_DIR"), "/sd_backend.apply_permission.rs"));
}
