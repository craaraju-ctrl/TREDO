use std::sync::Arc;
use dashmap::DashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TantraTask {
    pub id: String,
    pub title: String,
    pub priority: String,
    pub status: String,
    pub description: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TantraEvent {
    pub id: String,
    pub title: String,
    pub start: String,
    pub end: String,
    pub is_dnd: bool,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TantraNews {
    pub id: String,
    pub headline: String,
    pub source: String,
    pub impact: String,
    pub timestamp: u64,
}

#[derive(Clone)]
pub struct TantraService {
    pub tasks: Arc<DashMap<String, TantraTask>>,
    pub events: Arc<DashMap<String, TantraEvent>>,
    pub news: Arc<DashMap<String, TantraNews>>,
    pub dnd_active: Arc<std::sync::atomic::AtomicBool>,
}

impl TantraService {
    pub fn new() -> Self {
        let tasks = Arc::new(DashMap::new());
        let events = Arc::new(DashMap::new());
        let news = Arc::new(DashMap::new());
        let dnd_active = Arc::new(std::sync::atomic::AtomicBool::new(true)); // meeting active by default on load

        // Load pre-populated safety events
        events.insert("cal-1".to_string(), TantraEvent {
            id: "cal-1".to_string(),
            title: "Operator Weekly Risk & Alignment Meeting".to_string(),
            start: "09:00".to_string(),
            end: "10:00".to_string(),
            is_dnd: true,
            status: "ACTIVE".to_string(),
        });
        events.insert("cal-2".to_string(), TantraEvent {
            id: "cal-2".to_string(),
            title: "Binance API Maintenance Upgrade Window".to_string(),
            start: "14:00".to_string(),
            end: "15:30".to_string(),
            is_dnd: false,
            status: "PENDING".to_string(),
        });

        // Load pre-populated coworker workflow actions
        tasks.insert("task-1".to_string(), TantraTask {
            id: "task-1".to_string(),
            title: "Approve Exposure Limit Increase (SOL-USD)".to_string(),
            priority: "HIGH".to_string(),
            status: "PENDING".to_string(),
            description: "Autonomous agent requests allocation increase from 10% to 15% collateral.".to_string(),
            category: "Risk Review".to_string(),
        });
        tasks.insert("task-2".to_string(), TantraTask {
            id: "task-2".to_string(),
            title: "Inspect In-flight Collateral Overlap".to_string(),
            priority: "MEDIUM".to_string(),
            status: "PENDING".to_string(),
            description: "Bids on BTC and ETH exceed standard concurrent threshold warning.".to_string(),
            category: "Trade Verify".to_string(),
        });

        Self {
            tasks,
            events,
            news,
            dnd_active,
        }
    }

    pub async fn run_service(&self) {
        println!("[TantraService] Alert, Calendar, and Work Queue coordination active.");
    }

    pub fn set_dnd(&self, active: bool) {
        self.dnd_active.store(active, std::sync::atomic::Ordering::SeqCst);
        println!("[TantraService] Calendar DND guard status updated to: {}", active);
    }

    pub fn is_dnd_active(&self) -> bool {
        self.dnd_active.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn resolve_task(&self, task_id: &str) -> bool {
        if let Some(mut task) = self.tasks.get_mut(task_id) {
            task.status = "RESOLVED".to_string();
            println!("[TantraService] Resolved workflow coworker task: {}", task_id);
            true
        } else {
            false
        }
    }
}

impl Default for TantraService {
    fn default() -> Self {
        Self::new()
    }
}
