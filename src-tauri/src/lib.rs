use serde::{Deserialize, Serialize};
use scraper::{Html, Selector};
use std::io::{copy, Write};
use std::process::Command;
use once_cell::sync::Lazy;
use std::path::Path;
use reqwest::Client;
use std::fs::File;


// Constants
static CONFIG: Lazy<Result<AppConfig, String>> = Lazy::new(|| {
    let config_path = "config.json";
    if Path::new(config_path).exists() {
        let file_content = std::fs::read_to_string(config_path)
            .map_err(|_| "Failed to read configuration file".to_string())?;
        let config = serde_json::from_str::<AppConfig>(&file_content)
            .map_err(|_|"Failed to deserialize configuration".to_string())?;
        Ok(config)
    } else {
        let config = AppConfig {
            base_url: "https://rutracker.org".to_string(),
            proxy_url: "https://ps1.blockme.site:443".to_string(),
            cookie: "bb_session=0-52335687-cqygg3U3HlXLVNkKPD6R".to_string(),
        };
        let json_data = serde_json::to_string_pretty(&config)
            .map_err(|_|"Failed to serialize configuration".to_string())?;
        let mut file = File::create(config_path).map_err(|_|"Failed to create configuration file".to_string())?;
        file.write_all(json_data.as_bytes())
            .map_err(|_|"Failed to write to configuration file".to_string())?;
        Ok(config)
    }
});

#[derive(Debug, Default, Serialize, Deserialize)]
struct AppConfig {
    base_url: String,
    proxy_url: String,
    cookie: String
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

// Error type for custom errors
#[derive(Debug)]
enum AppError {
    Reqwest(reqwest::Error),
    Io(std::io::Error),
    Json(serde_json::Error),
    Other(String),
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Reqwest(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Json(err)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Reqwest(e) => write!(f, "Request error: {}", e),
            AppError::Io(e) => write!(f, "I/O error: {}", e),
            AppError::Json(e) => write!(f, "JSON error: {}", e),
            AppError::Other(e) => write!(f, "Other error: {}", e),
        }
    }
}

// Helper function to create a client with proxy
fn create_client() -> Result<Client, AppError> {
    let proxy = reqwest::Proxy::https(CONFIG.as_ref().unwrap().proxy_url.as_str())?;
    Ok(Client::builder()
        .proxy(proxy)
        .danger_accept_invalid_certs(true)
        .build()?)
}

async fn make_search_query(query: &str, page: u64) -> Result<String, AppError> {
    let url = format!("{}/forum/tracker.php", CONFIG.as_ref().unwrap().base_url.as_str());
    let client = create_client()?;
    let response = client
        .post(url)
        .query(&[("start", (page * 50).to_string().as_str()), ("nm", query), ("o", "4"), ("s", "2")])
        .header("Cookie", CONFIG.as_ref().unwrap().cookie.as_str())
        .send()
        .await?;
    response.error_for_status_ref()?;
    Ok(response.text().await?)
}

fn parse_search_query(content: String) -> Result<String, AppError> {
    let document = Html::parse_document(&content);
    let selector = Selector::parse("#tor-tbl tbody tr").map_err(|_| AppError::Other("Failed to parse selector".to_string()))?;
    let items = document.select(&selector);
    let mut search_items = Vec::new();

    for item in items {
        let selector = Selector::parse("td").map_err(|_| AppError::Other("Failed to parse td selector".to_string()))?;
        let item_cols: Vec<_> = item.select(&selector).collect();
        let mut search_item = SearchItem::default();

        if let Some(el) = item.select(&Selector::parse(".t-title a").map_err(|_| AppError::Other("Failed to parse a selector".to_string()))?).next() {
            search_item.id = el.attr("data-topic_id").and_then(|v| v.parse::<u64>().ok());
            search_item.title = Some(el.text().collect::<String>().trim().to_string());
        }

        search_item.topic = item_cols.get(2).map(|v| v.text().collect::<String>().trim().into());
        search_item.author = item_cols.get(4).map(|v| v.text().collect::<String>().trim().into());
        search_item.size = item_cols.get(5).map(|v| v.text().collect::<String>().trim().replace(" ↓", "").replace(" ", ""));
        search_item.downloads = item_cols.get(8).map(|v| v.text().collect::<String>().trim().into());
        search_item.date = item_cols.get(9).map(|v| v.text().collect::<String>().trim().into());

        search_items.push(search_item);
    }

    Ok(serde_json::to_string(&search_items)?)
}

#[tauri::command]
async fn download_item(item_id: &str) -> Result<String, String> {
    let url = format!("{}/forum/dl.php", CONFIG.as_ref().unwrap().base_url.as_str());
    let client = create_client().map_err(|e| e.to_string())?;
    let response = client
        .post(url)
        .query(&[("t", item_id)])
        .header("Cookie", CONFIG.as_ref().unwrap().cookie.as_str())
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

async fn request_item_files_list(item_id: &str) -> Result<String, AppError> {
    let url = format!("{}/forum/viewtorrent.php", CONFIG.as_ref().unwrap().base_url.as_str());
    let client = create_client()?;
    let response = client
        .post(url)
        .form(&[("t", item_id)])
        .header("Cookie", CONFIG.as_ref().unwrap().cookie.as_str())
        .send()
        .await?;
    
    response.error_for_status_ref()?;
    Ok(response.text().await?)
}

#[tauri::command]
async fn get_item_files_list(item_id: &str) -> Result<String, String> {
    request_item_files_list(item_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn search_query(query: &str, page: u64) -> Result<String, String> {
    let result = make_search_query(query, page).await.map_err(|e| e.to_string())?;
    parse_search_query(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn init_config() -> Result<String, String> {
    match CONFIG.as_ref() {
        Ok(config) => Ok(serde_json::to_string(config).unwrap()),
        Err(err) => Err(err.clone())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            init_config,
            search_query, 
            download_item, 
            get_item_files_list
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
