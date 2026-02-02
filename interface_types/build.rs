use std::io::Result;
fn main() -> Result<()> {
    prost_build::compile_protos(
        &[
            "src/proto/user.proto",
            "src/proto/notice.proto",
            "src/proto/mutil_media.proto",
            "src/proto/slideshow.proto",
            "src/proto/community_service.proto",
            "src/proto/resource_service.proto",
            "src/proto/medical_service.proto",
            "src/proto/feedback.proto",
            "src/proto/dinner_provider.proto",
            "src/proto/detail_meal.proto",
            "src/proto/apply_permission.proto",
            "src/proto/service_map_type.proto",
            "src/proto/health_guide_type.proto",
            "src/proto/service_map_content.proto",
            "src/proto/health_guide_content.proto",
            "src/proto/policy_type.proto",
            "src/proto/policy_file.proto",
            "src/proto/admin_manager.proto",
            "src/proto/ai_chat.proto",
        ],
        &["src"],
    )?;
    Ok(())
}
