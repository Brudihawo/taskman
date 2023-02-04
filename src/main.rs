use std::collections::HashMap;
use uuid::Uuid;

use eframe::egui;
use eframe::egui::Color32;

mod manager;
mod pomodoro;
mod task;

use manager::TaskManager;
use pomodoro::{Pomodoro, PomodoroStatus};

// TODO: Tags
// TODO: Task Groups
// TODO: Styling

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1080.0, 1920.0)),
        ..Default::default()
    };
    eframe::run_native(
        TaskManager::APPNAME,
        options,
        Box::new(|cc| Box::new(TaskManager::new(cc))),
    )
}
