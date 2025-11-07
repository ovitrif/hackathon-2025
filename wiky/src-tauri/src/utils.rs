use pubky::PubkySession;
use qrcode::QrCode;
use image::{DynamicImage, ImageFormat};
use base64::{Engine as _, engine::general_purpose};

pub fn generate_qr_image_base64(url: &str) -> Option<String> {
    let qr = QrCode::new(url.as_bytes()).ok()?;
    let qr_image = qr.render::<image::Luma<u8>>().build();

    // Convert to PNG and encode as base64
    let dynamic_image = DynamicImage::ImageLuma8(qr_image);
    let mut buffer = std::io::Cursor::new(Vec::new());
    dynamic_image.write_to(&mut buffer, ImageFormat::Png).ok()?;
    let base64_string = general_purpose::STANDARD.encode(buffer.into_inner());

    Some(format!("data:image/png;base64,{}", base64_string))
}

/// In this context, the title is the readable text on the 1st line
pub fn extract_title(input: &str) -> &str {
    // Get the first line by splitting on newlines and taking the first element
    let first_line = input.lines().next().unwrap_or("");
    first_line.trim_start_matches("# ")
}

pub fn extract_details_wiki_url(url: &str) -> Option<(String, String)> {
    // Split once on '/' and collect the two parts.
    let mut parts = url.splitn(2, '/');

    let first = parts.next()?.trim();
    let second = parts.next()?.trim();

    // Ensure both parts are present and not empty.
    if first.is_empty() || second.is_empty() {
        log::warn!("Invalid Pubky Wiki link: {url}");
        return None;
    }

    Some((first.to_string(), second.to_string()))
}

/// List files from the homeserver
pub async fn get_list(
    session: &PubkySession,
    folder_path: &str,
) -> anyhow::Result<Vec<String>> {
    let session_storage = session.storage();
    let session_storage_list_fut = session_storage.list(folder_path).unwrap().send();

    log::info!("listing {folder_path}");

    let mut result_list = vec![];
    for entry in session_storage_list_fut.await? {
        result_list.push(entry.to_pubky_url());
    }

    Ok(result_list)
}
