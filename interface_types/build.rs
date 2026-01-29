use std::io::Result;
fn main() -> Result<()> {
    prost_build::compile_protos(
        &[
            "src/proto/user.proto",
            "src/proto/notice.proto",
        ],
        &["src"],
    )?;
    Ok(())
}
