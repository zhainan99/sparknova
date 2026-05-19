use tauri::{AppHandle, Manager};
use crate::domain::NoteKind;
use crate::storage::Storage;

#[tauri::command]
pub async fn list_notes(app: AppHandle) -> Result<Vec<serde_json::Value>, String> {
    let storage = app.state::<std::sync::Arc<Storage>>();
    let notes = storage.note_store().list_all().map_err(|e| e.to_string())?;
    Ok(notes.into_iter().map(|n| serde_json::json!({
        "id": n.id,
        "content": n.content,
        "kind": format!("{:?}", n.kind),
        "done": n.done,
        "created_at": n.created_at,
    })).collect())
}

#[tauri::command]
pub async fn create_note(app: AppHandle, content: String, kind: String) -> Result<serde_json::Value, String> {
    let storage = app.state::<std::sync::Arc<Storage>>();
    let note_kind = match kind.as_str() {
        "Todo" => NoteKind::Todo,
        _ => NoteKind::Memo,
    };
    let note = storage.note_store().insert(&content, note_kind).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "id": note.id,
        "content": note.content,
        "kind": format!("{:?}", note.kind),
        "done": note.done,
        "created_at": note.created_at,
    }))
}

#[tauri::command]
pub async fn delete_note(app: AppHandle, id: u64) -> Result<(), String> {
    let storage = app.state::<std::sync::Arc<Storage>>();
    storage.note_store().delete(id).map_err(|e| e.to_string())
}