// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use serde::{Deserialize, Serialize};
use tauri::State;

#[tauri::command]
fn get_manifest(manifest: State<Manifest>) -> Manifest {
    manifest.inner().to_owned()
}

#[tauri::command]
fn read_audio_file(file: String) -> Result<Vec<u8>, String> {
    let data = std::fs::read(file).map_err(|e| e.to_string())?;
    Ok(data)
}

#[derive(Parser, Debug)]
pub struct Cli {
    file: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AudioFile {
    #[serde(default)]
    name: String,
    #[serde(default)]
    directions: String,
    path: String,
    #[serde(default)]
    length: u32,
    volume: Option<f32>,
    start_at: Option<f32>,
    end_at: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Manifest {
    items: Vec<AudioFile>,
}

fn main() {
    let args = Cli::parse();
    let manifest_data = std::fs::read_to_string(args.file).expect("opening manifest file");
    let manifest: Manifest = toml::from_str(&manifest_data).expect("decoding manifest");

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_manifest, read_audio_file])
        .manage(manifest)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
