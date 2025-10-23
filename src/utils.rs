use qrcode::QrCode;

pub fn generate_qr_image(url: &str) -> Option<egui::ColorImage> {
    let qr = QrCode::new(url.as_bytes()).ok()?;
    let qr_image = qr.render::<image::Luma<u8>>().build();

    let (width, height) = qr_image.dimensions();
    let scale = 2; // Scale QR code to fit within window
    let scaled_width = (width * scale) as usize;
    let scaled_height = (height * scale) as usize;

    let mut pixels = Vec::with_capacity(scaled_width * scaled_height);

    for y in 0..scaled_height {
        for x in 0..scaled_width {
            let orig_x = x as u32 / scale;
            let orig_y = y as u32 / scale;
            let pixel = qr_image.get_pixel(orig_x, orig_y);
            let color = if pixel[0] < 128 {
                egui::Color32::BLACK
            } else {
                egui::Color32::WHITE
            };
            pixels.push(color);
        }
    }

    Some(egui::ColorImage::new([scaled_width, scaled_height], pixels))
}

/// In this context, the title is the readable text on the 1st line
pub fn extract_title(input: &str) -> &str {
    // Get the first line by splitting on newlines and taking the first element
    let first_line = input.lines().next().unwrap_or("");
    first_line.trim_start_matches("# ")
}

/// Parse a wiki URL/link and extract user_id and page_id
/// 
/// Supports multiple formats:
/// - `user_id/page_id`
/// - `pubky://user_id/pub/wiki.app/page_id`
/// - Any URL ending with `user_id/page_id`
/// 
/// Returns `Some((user_id, page_id))` if valid, `None` otherwise
/// Logs a warning if the URL format is invalid
pub fn parse_wiki_link(url: &str) -> Option<(String, String)> {
    // Split on '/' and collect non-empty parts
    let parts: Vec<&str> = url.split('/').filter(|s| !s.is_empty()).collect();
    
    if parts.len() >= 2 {
        // Take last two non-empty parts as user_id and page_id
        let user_id = parts[parts.len() - 2].to_string();
        let page_id = parts[parts.len() - 1].to_string();
        
        log::info!("Parsed wiki link: user='{}', page='{}'", user_id, page_id);
        Some((user_id, page_id))
    } else {
        log::warn!("Invalid wiki link format: '{}' (parsed parts: {:?})", url, parts);
        None
    }
}
