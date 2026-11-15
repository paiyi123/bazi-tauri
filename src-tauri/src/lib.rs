mod bazi;
mod birth_time;
mod lesson_analysis;
mod lu_strength;
mod print_template;
mod quant_model;
mod shen_sha;
mod share;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default().plugin(tauri_plugin_opener::init());
    // 手機版才有相機掃碼（命例 QR Code 匯入）。
    #[cfg(mobile)]
    let builder = builder.plugin(tauri_plugin_barcode_scanner::init());

    builder
        .invoke_handler(tauri::generate_handler![
            bazi::calculate_bazi,
            bazi::analyze_pillars,
            bazi::get_lunar_year_options,
            bazi::get_lunar_month_detail,
            print_template::render_print_html,
            share::record_share_qr_svg
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
