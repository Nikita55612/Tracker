use serde::{Deserialize, Serialize};
use tauri::path::BaseDirectory;
use scraper::{Html, Selector};
use std::io::{copy, Write};
use std::process::Command;
use reqwest::Client;
use tauri::{Manager, State};
use std::fs::File;


struct AppData {
    config: AppConfig,
    client: Client
}

fn read_config(path: std::path::PathBuf) -> Result<AppConfig, String> {
    let config;
    if path.exists() {
        let file_content = std::fs::read_to_string(&path)
            .map_err(|_| "Failed to read configuration file".to_string())?;
        config = serde_json::from_str::<AppConfig>(&file_content)
            .map_err(|_|"Failed to deserialize configuration".to_string())?;
    } else {
        config = AppConfig::default();
        let json_data = serde_json::to_string_pretty(&config)
            .map_err(|_|"Failed to serialize configuration".to_string())?;
        let mut file = File::create(&path).map_err(|_|"Failed to create configuration file".to_string())?;
        file.write_all(json_data.as_bytes())
            .map_err(|_|"Failed to write to configuration file".to_string())?; 
    }
    Ok(config)
}

#[derive(Debug, Serialize, Deserialize)]
struct AppConfig {
    base_url: String,
    proxy_url: String,
    cookie: String
}

impl Default for AppConfig {
    fn default() -> AppConfig {
        AppConfig {
            base_url: "https://rutracker.org".to_string(),
            proxy_url: "https://ps1.blockme.site:443".to_string(),
            cookie: "bb_session=0-52335687-cqygg3U3HlXLVNkKPD6R".to_string(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct SearchItem {
    id: Option<u64>,
    title: Option<String>,
    topic: Option<String>,
    author: Option<String>,
    size: Option<String>,
    downloads: Option<String>,
    date: Option<String>,
}

fn parse_search_query(content: String) -> Result<String, String> {
    let document = Html::parse_document(&content);
    let selector = Selector::parse("#tor-tbl tbody tr")
        .map_err(|_| "Failed to parse selector".to_string())?;
    let items = document.select(&selector);
    let mut search_items = Vec::new();

    for item in items {
        let selector = Selector::parse("td")
            .map_err(|_| "Failed to parse td selector".to_string())?;
        let item_cols: Vec<_> = item.select(&selector).collect();
        let mut search_item = SearchItem::default();

        if let Some(el) = item.select(&Selector::parse(".t-title a")
            .map_err(|_| "Failed to parse a selector".to_string())?).next() {
                search_item.id = el.attr("data-topic_id").and_then(|v| v.parse::<u64>().ok());
                search_item.title = Some(el.text().collect::<String>().trim().to_string());
        }

        search_item.topic = item_cols.get(2).map(|v| v.text().collect::<String>().trim().into());
        search_item.author = item_cols.get(4).map(|v| v.text().collect::<String>().trim().into());
        search_item.size = item_cols.get(5).map(|v| v.text().collect::<String>().trim()
            .replace(" ↓", "").replace(" ", ""));
        search_item.downloads = item_cols.get(8).map(|v| v.text().collect::<String>().trim().into());
        search_item.date = item_cols.get(9).map(|v| v.text().collect::<String>().trim().into());

        search_items.push(search_item);
    }
    Ok(serde_json::to_string(&search_items).map_err(|e| format!("JSON error: {}", e))?)
}

#[tauri::command]
async fn download_item(state: State<'_, AppData>, item_id: &str) -> Result<String, String> {
    let url = format!("{}/forum/dl.php", state.config.base_url.as_str());
    let response = state.client.clone()
        .post(url)
        .query(&[("t", item_id)])
        .header("Cookie", state.config.cookie.as_str())
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        let download_dir = dirs_2::download_dir().ok_or("Failed to get download directory")?;
        let file_path = download_dir.join(format!("{}.torrent", item_id));
        let mut file = File::create(&file_path).map_err(|e| e.to_string())?;
        let content = response.bytes().await.map_err(|e| e.to_string())?;
        copy(&mut content.as_ref(), &mut file).map_err(|e| e.to_string())?;

        #[cfg(target_os = "windows")]
        Command::new("cmd")
            .args(&["/C", "start", "", &file_path.to_string_lossy()])
            .spawn()
            .map_err(|e| e.to_string())?;

        #[cfg(target_os = "macos")]
        Command::new("open")
            .arg(&file_path)
            .spawn()
            .map_err(|e| e.to_string())?;

        #[cfg(target_os = "linux")]
        Command::new("xdg-open")
            .arg(&file_path)
            .spawn()
            .map_err(|e| e.to_string())?;

        Ok("Download successful".to_string())
    } else {
        Err("Download failed".to_string())
    }
}

#[tauri::command]
async fn get_item_files_list(state: State<'_, AppData>, item_id: &str) -> Result<String, String> {
    let url = format!("{}/forum/viewtorrent.php", state.config.base_url.as_str());
    let response = state.client.clone()
        .post(url)
        .form(&[("t", item_id)])
        .header("Cookie", state.config.cookie.as_str())
        .send()
        .await
        .map_err(|e| e.to_string())?;
    response.error_for_status_ref().map_err(|e| e.to_string())?;
    response.text().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn search_query(state: State<'_, AppData>, query: &str, page: u64) -> Result<String, String> {
    let url = format!("{}/forum/tracker.php", state.config.base_url.as_str());
    let response = state.client.clone()
        .post(url)
        .query(&[("start", (page * 50).to_string().as_str()), ("nm", query), ("o", "4"), ("s", "2")])
        .header("Cookie", state.config.cookie.as_str())
        .send()
        .await
        .map_err(|e| e.to_string())?;
    response.error_for_status_ref().map_err(|e| e.to_string())?;
    let res = response.text().await.map_err(|e| e.to_string())?;
    parse_search_query(res)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let config_path = app.path().resolve("config.json", BaseDirectory::Resource)?;
            let config = read_config(config_path).unwrap_or_default();
            let proxy = reqwest::Proxy::https(config.proxy_url.as_str())?;
            let client = Client::builder()
                .proxy(proxy)
                .danger_accept_invalid_certs(true)
                .build()
                .map_err(|e| e.to_string())?;
            app.manage(AppData { 
                config: config,
                client: client
            });
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            search_query, 
            download_item, 
            get_item_files_list
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
