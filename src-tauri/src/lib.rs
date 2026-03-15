mod db;
mod models;
mod commands;
mod quick_capture;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            let app_data_dir = app.path().app_data_dir().expect("failed to get app data dir");
            let conn = db::init_db(app_data_dir)?;
            app.manage(db::DbState(std::sync::Mutex::new(conn)));
            quick_capture::setup_global_shortcut(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::daily_notes::get_or_create_daily_note,
            commands::blocks::create_block,
            commands::blocks::get_blocks_for_date,
            commands::blocks::update_block,
            commands::blocks::delete_block,
            commands::blocks::reorder_block,
            commands::blocks::reparent_block,
            commands::tags::create_tag,
            commands::tags::get_all_tags,
            commands::tags::add_tag_to_block,
            commands::tags::remove_tag_from_block,
            commands::tags::get_tags_for_block,
            commands::tags::get_blocks_by_tag,
            commands::links::create_link,
            commands::links::delete_link,
            commands::links::get_backlinks,
            commands::links::get_forward_links,
            commands::search::search,
            commands::mindmaps::create_mind_map,
            commands::mindmaps::get_mind_maps,
            commands::mindmaps::delete_mind_map,
            commands::mindmaps::add_mind_map_node,
            commands::mindmaps::update_node_position,
            commands::mindmaps::get_mind_map_nodes,
            commands::mindmaps::get_mind_map_nodes_with_blocks,
            commands::mindmaps::send_nodes_to_journal,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
