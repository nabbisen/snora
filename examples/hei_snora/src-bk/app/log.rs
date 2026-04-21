use snora::ToastIntent;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub intent: ToastIntent,
    pub timestamp: String,
    pub message: String,
}
