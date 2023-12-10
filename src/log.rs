use std::collections::VecDeque;

use chrono::{DateTime, Local};

pub struct Log {
    internal: VecDeque<LogEntry>,
}

impl Log {
    pub fn new() -> Self {
        Log {
            internal: VecDeque::new(),
        }
    }

    pub fn log(&mut self, message: impl Into<String>) {
        self.internal.push_front(LogEntry {
            timestamp: Local::now(),
            message: message.into(),
        });
    }

    pub fn take_lines(&self) -> impl Iterator<Item = String> + '_ {
        self.internal.iter().map(LogEntry::render_to_string)
    }

    pub fn len(&self) -> usize {
        self.internal.len()
    }
}

struct LogEntry {
    timestamp: DateTime<Local>,
    message: String,
}

impl LogEntry {
    fn render_to_string(&self) -> String {
        format!("[{}]: {}", self.timestamp.format("%H:%M:%S"), self.message)
    }
}
