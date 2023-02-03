use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

use std::error::Error;

use eframe::egui;

mod task;
use task::Task;
// TODO: Task Groups
// TODO: Tags
// TODO: Serialization
// TODO: Styling

struct TaskManager {
    tasks: HashMap<Uuid, Task>,
    show_new_dialog: bool,
    tmp_task: Option<Task>,
}

impl TaskManager {
    const APPNAME: &str = "taskman";
    const TASK_LIST: &str = "task_list";
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

fn configure_text_styles(ctx: &egui::Context) {
    use egui::FontFamily::{Monospace, Proportional};
    use egui::{FontId, TextStyle};

    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (TextStyle::Heading, FontId::new(32.0, Proportional)),
        (
            TextStyle::Name("Heading2".into()),
            FontId::new(22.0, Proportional),
        ),
        (
            TextStyle::Name("Heading3".into()),
            FontId::new(19.0, Proportional),
        ),
        (TextStyle::Body, FontId::new(16.0, Proportional)),
        (TextStyle::Monospace, FontId::new(12.0, Monospace)),
        (TextStyle::Button, FontId::new(12.0, Proportional)),
        (TextStyle::Small, FontId::new(8.0, Proportional)),
    ]
    .into();
    ctx.set_style(style);
}

impl TaskManager {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_text_styles(&&cc.egui_ctx);

        let mut tm = Self::default();
        if let Some(storage) = cc.storage {
            println!("Found Storage");
            if let Some(res) = storage.get_string(&TaskManager::TASK_LIST) {
                println!("Found task list entry");
                if let Ok(mut conv) = serde_json::from_str::<Vec<Task>>(&res) {
                    for task in conv.drain(..) {
                        tm.tasks.insert(task.get_uuid(), task);
                    }
                    println!("got task list of size {}", tm.tasks.len());
                    // TODO: Verify task link integrity
                }
            }
        } else {
            panic!("No storage found")
        }
        tm
    }

    fn add_task(&mut self, task: Task) {
        self.tasks.insert(task.get_uuid(), task);
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
        egui::SidePanel::left("Panel").show(ctx, |ui| {
            if ui.button("New Task").clicked() {
                self.show_new_dialog = true;
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("All Tasks:");
                for (_, task) in self.tasks.iter() {
                    task.display(ui);
                    ui.separator();
                }
            });
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string(
            "task_list",
            serde_json::to_string(&self.tasks.iter().map(|(_, x)| x).collect::<Vec<&Task>>())
                .unwrap(),
        );
        storage.flush();
    }
}


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
