use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

use std::error::Error;

use eframe::egui;
use eframe::egui::Color32;

mod task;
use task::Task;

// TODO: Tags
// TODO: Task Groups
// TODO: Styling

struct TaskManager {
    tasks: HashMap<Uuid, Task>,
    show_new_dialog: bool,
    tmp_task: Option<Task>,
    edit: Option<Uuid>,
    pomodoro: Option<Pomodoro>,
    notified: NotifyStatus,
    pomo_work: u32,
    pomo_break: u32,
}

struct Pomodoro {
    start: DateTime<Utc>,
    work_time: chrono::Duration,
    break_time: chrono::Duration,
}

enum NotifyStatus {
    SentBreak,
    SentWork,
    Nothing,
}

enum PomodoroStatus {
    Work(chrono::Duration),
    Break(chrono::Duration),
    Done,
}

impl Pomodoro {
    fn new(work_time: chrono::Duration, break_time: chrono::Duration) -> Self {
        Self {
            start: Utc::now(),
            work_time,
            break_time,
        }
    }

    fn status(&self) -> PomodoroStatus {
        let elapsed = Utc::now() - self.start;
        if elapsed < self.work_time {
            PomodoroStatus::Work(elapsed)
        } else if elapsed < self.work_time + self.break_time {
            PomodoroStatus::Break(elapsed - self.work_time)
        } else {
            PomodoroStatus::Done
        }
    }
}

impl Default for Pomodoro {
    fn default() -> Self {
        Self::new(chrono::Duration::minutes(25), chrono::Duration::minutes(5))
    }
}

impl TaskManager {
    const APPNAME: &str = "taskman";
    const TASK_LIST: &str = "task_list";
    const CLR_PUSHED: egui::Color32 = egui::Color32::DARK_GREEN;
    const CLR_NORMAL: egui::Color32 = egui::Color32::DARK_GRAY;
}

impl Default for TaskManager {
    fn default() -> Self {
        Self {
            tasks: HashMap::new(),
            show_new_dialog: false,
            tmp_task: None,
            edit: None,
            pomodoro: None,
            pomo_work: 25,
            pomo_break: 5,
            notified: NotifyStatus::Nothing,
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
        (
            TextStyle::Name("Smaller".into()),
            FontId::new(14.0, Proportional),
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

    fn new_task_dialog(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                        ui.text_edit_multiline(&mut task.description)
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

    fn edit_panel(&mut self, ctx: &egui::Context) {
        let mut defer_delete = false;

        if let Some(uuid) = &self.edit {
            let edit_task = self.tasks.get_mut(&uuid).unwrap();
            egui::SidePanel::right("Edit Task")
                .resizable(true)
                .show_animated(ctx, true, |ui| {
                    egui::Grid::new("Grid").striped(false).show(ui, |ui| {
                        if ui
                            .add(egui::Button::new("x").fill(Color32::DARK_RED))
                            .clicked()
                        {
                            self.edit = None;
                        }
                        ui.add_sized([ui.available_width(), 0.0], egui::Label::new("Task Name"));
                    });

                    ui.add_sized(
                        [ui.available_width(), 0.0],
                        egui::TextEdit::singleline(&mut edit_task.name)
                            .font(egui::TextStyle::Name("Heading2".into())),
                    );
                    ui.separator();

                    let description_label = ui.label("Description");
                    ui.add_sized(
                        [ui.available_width(), 0.0],
                        egui::TextEdit::multiline(&mut edit_task.description)
                            .font(egui::TextStyle::Body),
                    )
                    .labelled_by(description_label.id);

                    if ui.button("Delete").clicked() {
                        defer_delete = true;
                    }
                    // TODO: reset start and finish times
                });
        }
        if defer_delete {
            let to_del = self.edit.take().unwrap();
            self.tasks.remove(&to_del);
        }
    }

    fn task_list(&mut self, ui: &mut egui::Ui) {
        let mut tasks: Vec<&mut Task> = self.tasks.values_mut().collect();
        tasks.sort_by_key(|x| x.get_creation_time());
        for task in tasks.iter_mut().rev() {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    if ui
                        .add(egui::Button::new("start").fill(if task.is_started() {
                            TaskManager::CLR_PUSHED
                        } else {
                            TaskManager::CLR_NORMAL
                        }))
                        .clicked()
                    {
                        task.start();
                    }

                    if ui
                        .add(egui::Button::new("done").fill(if task.is_finished() {
                            TaskManager::CLR_PUSHED
                        } else {
                            TaskManager::CLR_NORMAL
                        }))
                        .clicked()
                    {
                        task.finish();
                    }

                    if ui
                        .add(
                            egui::Button::new("edit")
                                .fill(Color32::from_rgb_additive(0x89, 0x38, 0x01)),
                        )
                        .clicked()
                    {
                        self.edit = Some(task.get_uuid());
                    }
                });

                let sep = egui::Separator::default();
                ui.add(sep);
                task.display(ui);
            });
            ui.separator();
        }
    }
}

fn display_duration_min_s(d: chrono::Duration) -> String {
    format!(
        "{}:{:02}",
        d.num_minutes(),
        d.num_seconds() - d.num_minutes() * 60
    )
}

impl eframe::App for TaskManager {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.show_new_dialog {
            match self.tmp_task {
                Some(_) => {
                    self.new_task_dialog(ctx, frame);
                }
                None => self.tmp_task = Some(Task::default()),
            }
        }

        egui::SidePanel::left("Left Side").show(ctx, |ui| {
            ui.heading("Tasks");
            if ui.button("New Task").clicked() {
                self.show_new_dialog = true;
            }
            ui.separator();
            ui.heading("Pomodoro");
            if ui.button("Start / Stop").clicked() {
                match self.pomodoro {
                    Some(_) => self.pomodoro = None,
                    None => {
                        self.pomodoro = Some(Pomodoro::new(
                            chrono::Duration::minutes(self.pomo_work.into()),
                            chrono::Duration::minutes(self.pomo_break.into()),
                        ))
                    }
                }
            }

            if let Some(pomo) = &self.pomodoro {
                // Request Repaint so that progress bar updates regularly
                ctx.request_repaint();

                // Handle Notification
                match pomo.status() {
                    PomodoroStatus::Work(_) => match self.notified {
                        NotifyStatus::SentBreak | NotifyStatus::Nothing => {
                            notify_rust::Notification::new()
                                .summary("Start Working")
                                .body(&format!(
                                    "Working interval time: {}",
                                    display_duration_min_s(pomo.work_time)
                                ))
                                .show()
                                .unwrap();
                            self.notified = NotifyStatus::SentWork;
                        }
                        _ => (),
                    },
                    PomodoroStatus::Break(_) => match self.notified {
                        NotifyStatus::SentWork | NotifyStatus::Nothing => {
                            notify_rust::Notification::new()
                                .summary("Take a Break")
                                .body(&format!(
                                    "Break interval time: {}",
                                    display_duration_min_s(pomo.break_time)
                                ))
                                .show()
                                .unwrap();
                            self.notified = NotifyStatus::SentBreak;
                        }
                        _ => (),
                    },
                    PomodoroStatus::Done => match self.notified {
                        NotifyStatus::SentBreak | NotifyStatus::Nothing => {
                            notify_rust::Notification::new()
                                .summary("Pomodoro is Done")
                                .show()
                                .unwrap();
                            self.notified = NotifyStatus::Nothing;
                        }
                        _ => (),
                    },
                }

                let pbar = match pomo.status() {
                    PomodoroStatus::Work(work_time_elapsed) => egui::ProgressBar::new(
                        work_time_elapsed.num_seconds() as f32
                            / pomo.work_time.num_seconds() as f32,
                    )
                    .text(format!(
                        "Work Time: {}",
                        display_duration_min_s(work_time_elapsed)
                    )),
                    PomodoroStatus::Break(break_time_elapsed) => egui::ProgressBar::new(
                        break_time_elapsed.num_seconds() as f32
                            / pomo.break_time.num_seconds() as f32,
                    )
                    .text(format!(
                        "Break Time: {}",
                        display_duration_min_s(break_time_elapsed)
                    )),
                    PomodoroStatus::Done => egui::ProgressBar::new(1.0).text("Done"),
                };
                ui.add(pbar);
            } else {
                ui.add(egui::Slider::new(&mut self.pomo_work, 1..=60))
                    .labelled_by(ui.label("Work Interval").id);
                ui.add(egui::Slider::new(&mut self.pomo_break, 1..=60))
                    .labelled_by(ui.label("Break Interval").id);
            }
        });

        self.edit_panel(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("All Tasks");
                self.task_list(ui);
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
