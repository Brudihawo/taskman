use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

use eframe::egui;

#[derive(Debug)]
struct Task {
    id: Uuid,
    name: String,
    description: String,
    started: Option<DateTime<Utc>>,
    finished: Option<DateTime<Utc>>,
    subtasks: Option<Vec<Uuid>>,
}

// TODO: Task Groups
// TODO: Tags
// TODO: Serialization
// TODO: Styling

struct TaskManager {
    tasks: HashMap<Uuid, Task>,
    show_new_dialog: bool,
    tmp_task: Option<Task>,
}

impl Default for Task {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "New Task".to_string(),
            description: "".to_string(),
            started: None,
            finished: None,
            subtasks: None,
        }
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self {
            tasks: HashMap::new(),
            show_new_dialog: false,
            tmp_task: None,
        }
    }
}

impl TaskManager {
    fn add_task(&mut self, task: Task) {
        self.tasks.insert(task.id, task);
    }

    fn new_window(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut defer_add = false;
        if let Some(ref mut task) = self.tmp_task {
            egui::Window::new("Creating New Task")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        let name_label = ui.label("Task Name");
                        ui.text_edit_singleline(&mut task.name)
                            .labelled_by(name_label.id);
                    });

                    ui.horizontal(|ui| {
                        let desc_label = ui.label("Description");
                        ui.text_edit_singleline(&mut task.description)
                            .labelled_by(desc_label.id);
                    });

                    ui.horizontal(|ui| {
                        if ui.button("Add").clicked() {
                            defer_add = true;
                            self.show_new_dialog = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_new_dialog = false;
                        }
                    });
                });
        }
        if defer_add {
            let task = self.tmp_task.take().unwrap();
            self.add_task(task);

            println!("{:?}", self.tasks);
        }
    }
}

impl eframe::App for TaskManager {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.show_new_dialog {
            match self.tmp_task {
                Some(_) => {
                    self.new_window(ctx, frame);
                }
                None => self.tmp_task = Some(Task::default()),
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("TaskMan");

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("All Tasks:");
                for (_, task) in self.tasks.iter() {
                    ui.horizontal(|ui| {
                        ui.label(&task.name);
                    });
                }
            });

            if ui.button("New Task").clicked() {
                self.show_new_dialog = true;
            }
        });
    }
}

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1080.0, 1920.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Test",
        options,
        Box::new(|_cc| Box::new(TaskManager::default())),
    )
}
