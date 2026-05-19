use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Manager};

pub struct SearchState {
    engine: RwLock<Option<Arc<crate::search::SearchEngine>>>,
    frequency_cache: RwLock<std::collections::HashMap<String, u32>>,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            engine: RwLock::new(None),
            frequency_cache: RwLock::new(std::collections::HashMap::new()),
        }
    }

    pub fn init(&self, engine: Arc<crate::search::SearchEngine>, frequency_cache: std::collections::HashMap<String, u32>) {
        *self.engine.write().unwrap() = Some(engine);
        *self.frequency_cache.write().unwrap() = frequency_cache;
    }
}

#[tauri::command]
pub async fn query(app: AppHandle, q: String) -> Result<Vec<serde_json::Value>, String> {
    let state = app.state::<SearchState>();
    let engine = state.engine.read().unwrap();
    let freq_cache = state.frequency_cache.read().unwrap();

    if let Some(ref eng) = *engine {
        let hits = eng.search(&q, 8);
        let reranked = crate::search::rerank(&hits, &freq_cache);

        Ok(reranked.into_iter().map(|app| serde_json::json!({
            "name": app.name,
            "path": app.path.to_string_lossy()
        })).collect())
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
pub async fn hide_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn register(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    app.manage(SearchState::new());

    // 初始化搜索引擎
    let storage = Arc::new(crate::storage::Storage::open().map_err(|e| e.to_string())?);
    let apps = storage.index_cache().load().map_err(|e| e.to_string())?;
    let engine = Arc::new(crate::search::SearchEngine::new(apps));
    let frequency_cache = std::collections::HashMap::new();

    let state = app.state::<SearchState>();
    state.init(engine, frequency_cache);

    Ok(())
}