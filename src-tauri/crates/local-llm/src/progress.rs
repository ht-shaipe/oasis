use hf_hub::api::Progress;
use serde::{Deserialize, Serialize};
use tauri::Emitter;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgressPayload {
    pub model_id: String,
    pub file_name: String,
    pub current: usize,
    pub total: usize,
    pub percentage: u32,
    pub status: String,
}

pub struct TauriProgress {
    pub app: tauri::AppHandle,
    pub model_id: String,
    pub file_name: String,
    pub total: usize,
    pub current: usize,
}

impl Progress for TauriProgress {
    fn init(&mut self, size: usize, filename: &str) {
        self.total = size;
        self.file_name = filename.to_string();
        self.current = 0;
        let _ = self.app.emit(
            "local-llm-download-progress",
            DownloadProgressPayload {
                model_id: self.model_id.clone(),
                file_name: filename.to_string(),
                current: 0,
                total: size,
                percentage: 0,
                status: "downloading".to_string(),
            },
        );
    }

    fn update(&mut self, size: usize) {
        self.current = size;
        let pct = if self.total > 0 {
            (size as u64 * 100 / self.total as u64) as u32
        } else {
            0
        };
        let _ = self.app.emit(
            "local-llm-download-progress",
            DownloadProgressPayload {
                model_id: self.model_id.clone(),
                file_name: self.file_name.clone(),
                current: size,
                total: self.total,
                percentage: pct,
                status: "downloading".to_string(),
            },
        );
    }

    fn finish(&mut self) {
        let _ = self.app.emit(
            "local-llm-download-progress",
            DownloadProgressPayload {
                model_id: self.model_id.clone(),
                file_name: self.file_name.clone(),
                current: self.total,
                total: self.total,
                percentage: 100,
                status: "file_done".to_string(),
            },
        );
    }
}
