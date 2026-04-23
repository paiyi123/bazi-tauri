mod bazi;
mod print_template;
mod quant_model;
mod shen_sha;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            bazi::calculate_bazi,
            bazi::analyze_pillars,
            bazi::get_lunar_year_options,
            bazi::get_lunar_month_detail,
            print_template::render_print_html
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
