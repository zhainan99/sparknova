use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::thread;
use tauri::{AppHandle, Manager};
use tracing::info;

pub struct SearchState {
    engine: Arc<RwLock<Option<Arc<crate::search::SearchEngine>>>>,
    pub frequency_cache: Arc<RwLock<HashMap<String, u32>>>,
    storage: Arc<crate::storage::Storage>,
}

impl Clone for SearchState {
    fn clone(&self) -> Self {
        Self {
            engine: self.engine.clone(),
            frequency_cache: self.frequency_cache.clone(),
            storage: self.storage.clone(),
        }
    }
}

impl SearchState {
    pub fn new(storage: Arc<crate::storage::Storage>) -> Self {
        // 从 redb 加载已有频次数据（轻量操作，可以同步）
        let frequency_cache = storage.frequency()
            .get_all()
            .unwrap_or_default()
            .into_iter()
            .collect::<HashMap<String, u32>>();
        
        Self {
            engine: Arc::new(RwLock::new(None)),
            frequency_cache: Arc::new(RwLock::new(frequency_cache)),
            storage,
        }
    }
    
    /// 异步初始化搜索引擎（在后台线程加载索引）
    pub fn init_async(&self, storage: Arc<crate::storage::Storage>) {
        let state = self.clone();
        thread::spawn(move || {
            info!("开始后台加载搜索索引...");
            match storage.index_cache().load() {
                Ok(apps) => {
                    let engine = Arc::new(crate::search::SearchEngine::new(apps));
                    *state.engine.write().unwrap() = Some(engine);
                    info!("搜索索引加载完成");
                }
                Err(e) => {
                    tracing::warn!("加载搜索索引失败: {:?}", e);
                }
            }
        });
    }

    /// 同步初始化（保留用于测试）
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
        let _ = thread::spawn(move || {
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
        // 引擎还在加载中，返回空结果
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

    // 后台异步初始化搜索引擎
    let state = app.state::<SearchState>();
    state.init_async(storage);

    Ok(())
}