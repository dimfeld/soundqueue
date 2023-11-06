// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use clap::Parser;
use serde::{Deserialize, Serialize};
use tauri::State;

#[tauri::command]
fn get_manifest(args: State<Cli>) -> Result<Manifest, String> {
    let manifest_data =
        std::fs::read_to_string(&args.file).map_err(|e| format!("opening manifest file: {e}"))?;
    let mut manifest: Manifest =
        toml::from_str(&manifest_data).map_err(|e| format!("decoding manifest: {e}"))?;

    let full_path = args
        .file
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

#[derive(Parser, Debug)]
pub struct Cli {
    file: PathBuf,
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Manifest {
    items: Vec<AudioFile>,
}

fn main() {
    let args = Cli::parse();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_manifest,])
        .manage(args)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
