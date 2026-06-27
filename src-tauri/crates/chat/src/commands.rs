use tauri::{AppHandle, Manager};

use crate::db;
use crate::models::{
    Conversation, ConversationDetail, ConversationListItem, Message, NewConversation, NewMessage,
    UpdateMessageContent,
};

fn get_conn(app: &AppHandle) -> Result<rusqlite::Connection, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    db::init_db(&data_dir).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_conversations(app: AppHandle) -> Result<Vec<ConversationListItem>, String> {
    let conn = get_conn(&app)?;
    db::list_conversations(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_conversation(app: AppHandle, id: String) -> Result<ConversationDetail, String> {
    let conn = get_conn(&app)?;
    db::get_conversation(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_conversation(app: AppHandle, conversation: NewConversation) -> Result<Conversation, String> {
    let conn = get_conn(&app)?;
    db::create_conversation(&conn, &conversation).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_message(app: AppHandle, message: NewMessage) -> Result<Message, String> {
    let conn = get_conn(&app)?;
    db::save_message(&conn, &message).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_messages(app: AppHandle, messages: Vec<NewMessage>) -> Result<Vec<Message>, String> {
    let conn = get_conn(&app)?;
    db::save_messages(&conn, &messages).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_conversation_title(app: AppHandle, id: String, title: String) -> Result<(), String> {
    let conn = get_conn(&app)?;
    db::update_conversation_title(&conn, &id, &title).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_conversation(app: AppHandle, id: String) -> Result<(), String> {
    let conn = get_conn(&app)?;
    db::delete_conversation(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_message_content(app: AppHandle, message: UpdateMessageContent) -> Result<(), String> {
    let conn = get_conn(&app)?;
    db::update_message_content(&conn, &message).map_err(|e| e.to_string())
}
