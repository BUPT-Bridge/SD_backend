use image::{DynamicImage, ImageBuffer, ImageReader, imageops};

/// 处理头像：压缩为 webp 并裁剪为 120x120
///
/// 参数：
/// - file_data: 图片二进制数据
/// - filename: 原始文件名
///
/// 返回：
/// - Ok((Vec<u8>, String)): (处理后的 webp 数据, 新的文件名)
/// - Err(String): 错误信息
pub fn process_avatar(file_data: &[u8], filename: &str) -> Result<(Vec<u8>, String), String> {
    // 1) 尝试加载图片
    let img: DynamicImage = ImageReader::new(std::io::Cursor::new(file_data))
        .with_guessed_format()
        .map_err(|e| format!("Failed to read image: {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    // 2) 转换为 RGB 格式（webp 需要）
    let rgb_img: ImageBuffer<image::Rgb<u8>, Vec<u8>> = img.to_rgb8();

    // 3) 使用智能裁剪策略，保持中心区域
    let (width, height): (u32, u32) = (rgb_img.width(), rgb_img.height());
    let min_side: u32 = width.min(height);
    let x: u32 = (width - min_side) / 2;
    let y: u32 = (height - min_side) / 2;
    let cropped: image::SubImage<&ImageBuffer<image::Rgb<u8>, Vec<u8>>> =
        imageops::crop_imm(&rgb_img, x, y, min_side, min_side);
    let cropped: ImageBuffer<image::Rgb<u8>, Vec<u8>> = cropped.to_image();

    // 4) 调整大小为 120x120（使用高质量缩放）
    let resized: ImageBuffer<image::Rgb<u8>, Vec<u8>> =
        imageops::resize(&cropped, 120, 120, imageops::FilterType::Lanczos3);

    // 5) 转换为 DynamicImage
    let dynamic_image: DynamicImage = DynamicImage::ImageRgb8(resized);

    // 6) 编码为 webp 格式（使用默认质量 75）
    let encoder = webp::Encoder::from_image(&dynamic_image)
        .map_err(|e| format!("Failed to create webp encoder: {}", e))?;

    let webp_data: webp::WebPMemory = encoder.encode(75.0);
    let webp_bytes: Vec<u8> = webp_data.to_vec();

    // 7) 修改文件名为 .webp
    let webp_filename: String = format!(
        "{}.webp",
        filename.trim_end_matches(|c: char| { c == '.' || !c.is_alphanumeric() })
    );

    Ok((webp_bytes, webp_filename))
}

/// 压缩图片为 webp 格式
///
/// 参数：
/// - file_data: 图片二进制数据
/// - filename: 原始文件名
///
/// 返回：
/// - Ok((Vec<u8>, String)): (webp 数据, 新的文件名)
/// - Err(String): 错误信息
pub fn compress_to_webp(file_data: &[u8], filename: &str) -> Result<(Vec<u8>, String), String> {
    // 1) 尝试加载图片
    let img: DynamicImage = ImageReader::new(std::io::Cursor::new(file_data))
        .with_guessed_format()
        .map_err(|e| format!("Failed to read image: {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    // 2) 转换为 RGB 格式并包装为 DynamicImage（webp 需要）
    let rgb_img: ImageBuffer<image::Rgb<u8>, Vec<u8>> = img.to_rgb8();
    let dynamic_image: DynamicImage = DynamicImage::ImageRgb8(rgb_img);

    // 3) 编码为 webp 格式（使用默认质量 75）
    let encoder = webp::Encoder::from_image(&dynamic_image)
        .map_err(|e| format!("Failed to create webp encoder: {}", e))?;

    let webp_data: webp::WebPMemory = encoder.encode(75.0);
    let webp_bytes: Vec<u8> = webp_data.to_vec();

    // 4) 修改文件名为 .webp
    let webp_filename: String = format!(
        "{}.webp",
        filename.trim_end_matches(|c: char| { c == '.' || !c.is_alphanumeric() })
    );

    Ok((webp_bytes, webp_filename))
}

/// 从文件名中提取文件类型（后缀）
///
/// 参数：
/// - filename: 原始文件名，如 "image.jpg" 或 "document.pdf"
///
/// 返回：
/// - 文件后缀（小写），如 "jpg" 或 "pdf"
/// - 如果没有后缀，返回空字符串
///
/// 示例：
/// - "image.JPG" -> "jpg"
/// - "archive.tar.gz" -> "gz"
/// - "no_extension" -> ""
pub fn extract_file_type(filename: &str) -> String {
    // 查找最后一个点号的位置
    if let Some(pos) = filename.rfind('.') {
        // 提取点号之后的部分（后缀）
        let extension: &str = &filename[pos + 1..];
        // 转换为小写并去除空格
        extension.trim().to_lowercase()
    } else {
        // 没有找到点号，返回空字符串
        String::new()
    }
}
