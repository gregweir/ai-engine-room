// AI Engine Room native application boundary.
//
// The native entry point registers only narrow application commands: controlled
// acquisition views, pure composition/diagnosis, one explicitly authorized
// inference action, and the report-safe preview. There are no generic fs/shell/
// exec/env/inspection commands. The sole plugin supports the Report workspace's
// explicit write-only clipboard action; its capability grants plain-text writes
// only. aer-core remains the authoritative provider-neutral domain model and
// stays serde-free; provider-aware serialisable DTOs live in this application
// layer.

mod commands;
mod diagnosis;
mod machine;
pub mod platform;
mod report_save;
pub mod runtime;
// `view` is `pub` so the Milestone 1F read-only live verification
// (`tests/resource_context_live.rs`) can reach the pure composition
// (`compose_resource_context`) and the view projections (`snapshot_view`,
// `loaded_models_view`) the same way `runtime` and `platform` are already
// exposed for the 1E/1B live tests. The Tauri command surface in `commands`
// stays private; the live test calls the pure composition directly, not the
// command wrapper, and does not call the private `build_snapshot`.
pub mod view;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    report_save::register_dialog(
        tauri::Builder::default().plugin(tauri_plugin_clipboard_manager::init()),
    )
    .manage(report_save::ReportSaveState::default())
    .invoke_handler(tauri::generate_handler![
        commands::compose_resource_context,
        commands::current_snapshot,
        commands::current_machine_context,
        commands::current_loaded_models,
        commands::current_llama_cpp_snapshot,
        commands::current_lm_studio_snapshot,
        commands::current_model_inventory,
        commands::current_runtime_status,
        commands::diagnose_observation,
        commands::report_preview,
        commands::save_report,
        commands::run_inference_observation,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
