use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

pub struct SearchState {
    engine: RwLock<Option<Arc<crate::search::SearchEngine>>>,
    pub frequency_cache: RwLock<HashMap<String, u32>>,
    storage: Arc<crate::storage::Storage>,
}

impl SearchState {
    pub fn new(storage: Arc<crate::storage::Storage>) -> Self {
        // 从 redb 加载已有频次数据
        let frequency_cache = storage.frequency()
            .get_all()
            .unwrap_or_default()
            .into_iter()
            .collect::<HashMap<String, u32>>();
        
        Self {
            engine: RwLock::new(None),
            frequency_cache: RwLock::new(frequency_cache),
            storage,
        }
    }

    pub fn init(&self, engine: Arc<crate::search::SearchEngine>) {
        *self.engine.write().unwrap() = Some(engine);
    }
    
    /// 更新频次并持久化
    pub fn record_launch(&self, path: &str) {
        let mut cache = self.frequency_cache.write().unwrap();
        let count = cache.get(path).copied().unwrap_or(0);
        cache.insert(path.to_string(), count + 1);
        
        // 异步保存到 redb
        let storage = self.storage.clone();
        let path_owned = path.to_string();
        let _ = std::thread::spawn(move || {
            let _ = storage.frequency().record_launch(&path_owned);
        });
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
    let storage = Arc::new(crate::storage::Storage::open().map_err(|e| e.to_string())?);
    let state = SearchState::new(storage.clone());
    app.manage(state);

    // 初始化搜索引擎
    let apps = storage.index_cache().load().map_err(|e| e.to_string())?;
    let engine = Arc::new(crate::search::SearchEngine::new(apps));

    let state = app.state::<SearchState>();
    state.init(engine);

    Ok(())
}