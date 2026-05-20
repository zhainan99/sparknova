use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::thread;
use tauri::{AppHandle, Manager};
use tracing::{info, warn};

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
                    warn!("加载搜索索引失败: {:?}", e);
                }
            }
        });
    }
    
    /// 触发增量扫描（如果在 show_main_window 中调用）
    /// 检查是否需要更新索引，如果需要则执行扫描并更新搜索引擎
    pub fn trigger_incremental_scan(&self) {
        let cache = self.storage.index_cache();
        
        // 检查是否需要扫描
        let needs = match cache.needs_scan() {
            Ok(v) => v,
            Err(e) => {
                warn!("检查扫描状态失败: {:?}", e);
                return;
            }
        };
        
        info!(needs_scan = needs, "增量扫描检查结果");
        
        if !needs {
            return;
        }
        
        info!("触发增量索引更新...");
        let state = self.clone();
        let storage = self.storage.clone();
            
        thread::spawn(move || {
            let cache = storage.index_cache();
            
            // 执行扫描
            match crate::indexer::scan_start_menu() {
                Ok(report) => {
                    info!("扫描到 {} 个应用", report.entries.len());
                    
                    // 合并到缓存
                    match cache.merge_new_entries(&report.entries) {
                        Ok(merged) => {
                            // 更新搜索引擎
                            let new_engine = Arc::new(crate::search::SearchEngine::new(merged));
                            *state.engine.write().unwrap() = Some(new_engine);
                            
                            // 更新扫描时间
                            let _ = cache.set_last_scan_time(
                                crate::storage::IndexCache::current_timestamp()
                            );
                            
                            info!("增量索引更新完成");
                        }
                        Err(e) => {
                            warn!("合并索引失败: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("扫描应用失败: {:?}", e);
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