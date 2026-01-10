use std::io::Result;
fn main() -> Result<()> {
    prost_build::compile_protos(
        &[
            "src/proto/user.proto",
            "src/proto/notice.proto",
            "src/proto/slideshow.proto",
            "src/proto/policy.proto",
            "src/proto/community_service.proto",
        ],
        &["src"],
    )?;
    Ok(())
}
