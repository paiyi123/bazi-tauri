use qrcode::render::svg;
use qrcode::{EcLevel, QrCode};

/// 把命例分享碼畫成 QR Code（SVG 字串），給前端直接顯示。
#[tauri::command]
pub fn record_share_qr_svg(text: String) -> Result<String, String> {
    build_qr_svg(&text)
}

pub fn build_qr_svg(text: &str) -> Result<String, String> {
    if text.trim().is_empty() {
        return Err("分享碼是空的".to_string());
    }
    let code = QrCode::with_error_correction_level(text.as_bytes(), EcLevel::M)
        .map_err(|error| format!("無法產生 QR Code：{error}"))?;
    Ok(code
        .render::<svg::Color>()
        .min_dimensions(240, 240)
        .quiet_zone(true)
        .dark_color(svg::Color("#000000"))
        .light_color(svg::Color("#ffffff"))
        .build())
}

#[cfg(test)]
mod tests {
    use super::build_qr_svg;

    #[test]
    fn builds_svg_for_chinese_share_code() {
        let svg = build_qr_svg(r#"BAZI1:["王小明","F","S","A",1970,11,6,5,0,0,"卯時 05:00",""]"#)
            .expect("qr should build");
        assert!(svg.starts_with("<?xml") || svg.starts_with("<svg"));
        assert!(svg.contains("<path") || svg.contains("<rect"));
    }

    #[test]
    fn rejects_empty_text() {
        assert!(build_qr_svg("  ").is_err());
    }
}
