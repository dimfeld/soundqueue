// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{path::PathBuf, sync::Mutex};

use serde::{Deserialize, Serialize};
use tauri::{Manager, State};

#[tauri::command]
fn get_manifest(args: State<File>) -> Result<Manifest, String> {
    let path = args.path.lock().unwrap().clone();
    let manifest_data =
        std::fs::read_to_string(&path).map_err(|e| format!("opening manifest file: {e}"))?;
    let mut manifest: Manifest =
        toml::from_str(&manifest_data).map_err(|e| format!("decoding manifest: {e}"))?;

    let full_path = path
        .canonicalize()
        .map_err(|e| format!("converting to absolute path: {e}"))?;
    let base_dir = full_path
        .parent()
        .ok_or_else(|| "no parent dir".to_string())?;
    for item in &mut manifest.items {
        if item.name.is_empty() {
            item.name = item.path.display().to_string();
        }

        item.path = base_dir
            .join(&item.path)
            .canonicalize()
            .map_err(|e| format!("{}: {}", item.path.display(), e))?;
    }

    Ok(manifest)
}

pub struct File {
    pub path: Mutex<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ActionButton {
    name: String,
    volume: f32,
    #[serde(default = "default_duration")]
    duration: f32,
}

fn default_duration() -> f32 {
    0.0
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AudioFile {
    #[serde(default)]
    name: String,
    #[serde(default)]
    directions: String,
    path: PathBuf,
    #[serde(default)]
    length: u32,
    volume: Option<f32>,
    start_at: Option<f32>,
    stop_at: Option<f32>,
    /// Spend this many seconds fading in
    fade_in: Option<f32>,
    /// Spend this many seconds fading out
    fade_out: Option<f32>,
    #[serde(default)]
    actions: Vec<ActionButton>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Manifest {
    items: Vec<AudioFile>,
}

fn main() {
    use tauri_plugin_cli::CliExt;
    tauri::Builder::default()
         .plugin(tauri_plugin_cli::init())
        .setup(|app| {
            let matches = app.cli().matches()?;
            let file = &matches.args["file"];
            *app.state::<File>().path.lock().unwrap() =
                PathBuf::from(file.value.as_str().expect("File should be a string"));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_manifest,])
        .manage(File {
            path: Mutex::new(PathBuf::new()),
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
