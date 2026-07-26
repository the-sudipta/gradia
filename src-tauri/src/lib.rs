mod backup;
mod calc;
mod commands;
mod db;
mod excel;
mod models;

use db::DbState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Unable to resolve Gradia data directory: {e}"))?;
            let database_path = data_dir.join("gradia.db");
            db::open(&database_path)?;
            app.manage(DbState {
                path: database_path,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_bootstrap,
            commands::create_semester,
            commands::update_semester,
            commands::set_active_semester,
            commands::create_course,
            commands::update_course,
            commands::create_section,
            commands::update_section,
            commands::add_student,
            commands::update_student,
            commands::preview_roster_import,
            commands::import_roster,
            commands::list_roster,
            commands::search_students,
            commands::create_gradebook_view,
            commands::update_gradebook_view,
            commands::create_assessment_field,
            commands::update_assessment_field,
            commands::save_grade_entry,
            commands::get_gradebook,
            commands::get_grade_bands,
            commands::save_grading_policy,
            commands::get_section_analytics,
            commands::create_attendance_session,
            commands::update_attendance_session,
            commands::set_attendance_status,
            commands::get_attendance,
            commands::toggle_pipeline,
            commands::get_pipeline,
            commands::get_dashboard,
            commands::get_delete_impact,
            commands::delete_academic_entity,
            commands::finalize_results,
            excel::preflight_excel,
            excel::preview_excel_export,
            excel::export_excel,
            backup::export_backup,
            backup::import_backup
        ])
        .run(tauri::generate_context!())
        .expect("error while running Gradia");
}
