use std::collections::HashMap;
use uuid::Uuid;

use std::path::PathBuf;

use crate::pomodoro::{Pomodoro, PomodoroStatus};
use crate::task::{Task, TaskStatus};

use chrono::{DateTime, Utc};

use egui::Color32;

use eframe::{self, egui};

enum NotifyStatus {
    SentBreak,
    SentWork,
    Nothing,
}

pub struct TaskManager {
    tasks: HashMap<Uuid, Task>,

    show_creation_dialog: bool,
    tmp_task: Option<Task>,

    err_msg: Option<String>,

    edit: Option<Uuid>,

    notified: NotifyStatus,
    pomodoro: Option<Pomodoro>,
    pomo_work: u32,
    pomo_break: u32,
    squash_import: bool,
}

impl TaskManager {
    pub const APPNAME: &str = "taskman";
    pub const TASK_LIST: &str = "task_list";
    const CLR_PUSHED: egui::Color32 = egui::Color32::DARK_GREEN;
    const CLR_NORMAL: egui::Color32 = egui::Color32::DARK_GRAY;

    const CLR_CONFIRM: egui::Color32 = egui::Color32::DARK_GREEN;
    const CLR_ABORT: egui::Color32 = egui::Color32::DARK_RED;

    const CLR_DONE: egui::Color32 = egui::Color32::DARK_GREEN;
    const CLR_INPROGRESS: egui::Color32 = egui::Color32::from_rgb_additive(0x89, 0x38, 0x01);
    const CLR_NOTSTARTED: egui::Color32 = egui::Color32::DARK_GRAY;
}

impl Default for TaskManager {
    fn default() -> Self {
        Self {
            tasks: HashMap::new(),
            show_creation_dialog: false,
            tmp_task: None,
            edit: None,
            pomodoro: None,
            pomo_work: 25,
            pomo_break: 5,
            notified: NotifyStatus::Nothing,
            err_msg: None,
            squash_import: false,
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
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
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
                    println!("Got task list of size {}", tm.tasks.len());
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

    fn creation_dialog(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut defer_add = false;
        if let Some(ref mut new_task) = self.tmp_task {
            egui::Window::new("Creating New Task")
                .collapsible(false)
                .title_bar(true)
                .resizable(true)
                .show(ctx, |ui| {
                    for event in ui.input().events.clone() {
                        match event {
                            egui::Event::Key {
                                key,
                                pressed,
                                modifiers,
                            } => {
                                if key == egui::Key::W && pressed && modifiers.ctrl {
                                    self.show_creation_dialog = false;
                                }
                                if key == egui::Key::Enter && pressed && modifiers.ctrl {
                                    defer_add = true;
                                    self.show_creation_dialog = false;
                                }
                            }
                            _ => (),
                        }
                    }

                    ui.horizontal(|ui| {
                        let name_label = ui.label("Task Name");
                        ui.add_sized(
                            [ui.available_width(), 0.0],
                            egui::TextEdit::singleline(&mut new_task.name),
                        )
                        .labelled_by(name_label.id);
                    });

                    ui.horizontal(|ui| {
                        let desc_label = ui.label("Description");
                        ui.add_sized(
                            [ui.available_width(), 0.0],
                            egui::TextEdit::multiline(&mut new_task.description),
                        )
                        .labelled_by(desc_label.id);
                    });

                    ui.separator();
                    egui::ScrollArea::new([false, true]).show(ui, |ui| {
                        ui.vertical(|ui| {
                            let heading = egui::RichText::new("Select subtasks")
                                .text_style(egui::TextStyle::Name("Heading3".into()));
                            ui.label(heading);
                            for existing_task in self.tasks.values() {
                                let mut selected = new_task.has_subtask(existing_task.get_uuid());
                                let before = selected;

                                if ui
                                    .add_sized(
                                        [ui.available_width(), 0.0],
                                        egui::SelectableLabel::new(selected, &existing_task.name),
                                    )
                                    .clicked()
                                {
                                    selected = !selected;
                                };
                                if before != selected {
                                    if selected {
                                        new_task.add_subtask(
                                            existing_task.get_uuid(),
                                            existing_task.name.clone(),
                                        );
                                    } else {
                                        new_task.remove_subtask(existing_task.get_uuid());
                                    }
                                }
                            }
                        });
                    });

                    ui.separator();
                    ui.columns(2, |cols| {
                        if cols[0]
                            .add(egui::Button::new("Add").fill(TaskManager::CLR_CONFIRM))
                            .clicked()
                        {
                            defer_add = true;
                            self.show_creation_dialog = false;
                        }
                        if cols[1]
                            .add(egui::Button::new("Cancel").fill(TaskManager::CLR_ABORT))
                            .clicked()
                        {
                            self.show_creation_dialog = false;
                        }
                    });
                });
        }

        if defer_add {
            let task = self.tmp_task.take().unwrap();
            self.add_task(task);
        }
    }

    fn edit_pane(&mut self, ctx: &egui::Context) {
        let mut defer_delete = false;

        if let Some(uuid) = &self.edit {
            let mut task_names: Vec<(Uuid, String, DateTime<Utc>)> = self
                .tasks
                .values()
                .map(|x| (x.get_uuid(), x.name.clone(), x.get_creation_time()))
                .collect();

            task_names.sort_by_key(|(_id, _name, time)| *time);

            let task_names = task_names
                .drain(..)
                .map(|(id, name, _)| (id, name))
                .collect::<Vec<(Uuid, String)>>();

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

                    ui.separator();
                    egui::ScrollArea::new([false, true]).show(ui, |ui| {
                        ui.vertical(|ui| {
                            let heading = egui::RichText::new("Select subtasks")
                                .text_style(egui::TextStyle::Name("Heading3".into()));
                            ui.label(heading);
                            for (eid, ename) in task_names.iter() {
                                if *eid == edit_task.get_uuid() {
                                    continue;
                                }
                                let mut selected = edit_task.has_subtask(*eid);
                                let before = selected;

                                if ui
                                    .add_sized(
                                        [ui.available_width(), 0.0],
                                        egui::SelectableLabel::new(selected, ename.clone()),
                                    )
                                    .clicked()
                                {
                                    selected = !selected;
                                };
                                if before != selected {
                                    if selected {
                                        edit_task.add_subtask(*eid, ename.clone());
                                    } else {
                                        edit_task.remove_subtask(*eid);
                                    }
                                }
                            }
                        });
                    });

                    if ui.button("Delete").clicked() {
                        defer_delete = true;
                    }
                    // TODO: reset start and finish times
                });
        }
        if defer_delete {
            let to_del = self.edit.take().unwrap();
            for task in self.tasks.values_mut() {
                if task.has_subtask(to_del) {
                    task.remove_subtask(to_del);
                }
            }
            self.tasks.remove(&to_del);
        }
    }

    fn task_list(&mut self, ui: &mut egui::Ui) {
        let mut stati: HashMap<Uuid, TaskStatus> = self
            .tasks
            .values()
            .map(|v| (v.get_uuid(), v.status()))
            .collect();

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
                });

                let sep = egui::Separator::default();
                ui.add(sep);
                ui.vertical(|ui| {
                    if task.display(ui) {
                        self.edit = Some(task.get_uuid())
                    };
                    ui.vertical(|ui| {
                        if let Some(subtasks) = task.get_subtasks() {
                            for (id, name) in subtasks.iter() {
                                let label =
                                    egui::RichText::new(name).color(match stati.get(id).unwrap() {
                                        TaskStatus::NotYet => TaskManager::CLR_NOTSTARTED,
                                        TaskStatus::Started => TaskManager::CLR_INPROGRESS,
                                        TaskStatus::Finished => TaskManager::CLR_DONE,
                                    });
                                ui.label(label);
                            }
                        }
                    });
                });
            });
            ui.separator();
        }
    }

    fn pomodoro_display(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
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
                    NotifyStatus::SentBreak | NotifyStatus::SentWork => {
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
                    work_time_elapsed.num_seconds() as f32 / pomo.work_time.num_seconds() as f32,
                )
                .text(format!(
                    "Work Time: {}",
                    display_duration_min_s(work_time_elapsed)
                )),
                PomodoroStatus::Break(break_time_elapsed) => egui::ProgressBar::new(
                    break_time_elapsed.num_seconds() as f32 / pomo.break_time.num_seconds() as f32,
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
    }

    fn err_win(&mut self, ctx: &egui::Context) {
        let mut close = false;
        if let Some(msg) = &self.err_msg {
            egui::Window::new("Error").show(ctx, |ui| {
                ui.label(msg);
                if ui.button("Ok").clicked() {
                    close = true;
                }
            });
        }
        if close {
            self.err_msg = None;
        }
    }

    fn import(&mut self) {
        let maybe_path = rfd::FileDialog::new()
            .set_directory(home::home_dir().unwrap_or(".".into()))
            .add_filter("json", &["json"])
            .pick_file();

        if let Some(path) = maybe_path {
            match std::fs::File::open(&path) {
                Ok(infile) => {
                    println!("Importing from {}.", path.to_str().unwrap());
                    match serde_json::from_reader::<_, Vec<Task>>(infile) {
                        Ok(mut tasks) => {
                            for task in &mut tasks.drain(..) {
                                if self.squash_import {
                                    self.add_task(task);
                                } else {
                                    if !self.tasks.contains_key(&task.get_uuid()) {
                                        self.add_task(task);
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            self.err_msg = Some(format!(
                                "Error during parsing of file '{}': {}",
                                path.to_str().unwrap(),
                                err.to_string()
                            ))
                        }
                    }
                }
                Err(err) => self.err_msg = Some(err.to_string()),
            }
        }
    }

    fn export(&mut self) {
        let maybe_path = rfd::FileDialog::new()
            .set_directory(home::home_dir().unwrap_or(".".into()))
            .add_filter("json", &["json"])
            .save_file();

        if let Some(path) = maybe_path {
            match std::fs::File::create(&path) {
                Ok(outfile) => {
                    println!("Saving to {}.", path.to_str().unwrap());
                    serde_json::to_writer(
                        outfile,
                        &self.tasks.iter().map(|(_, x)| x).collect::<Vec<&Task>>(),
                    )
                    .unwrap();
                }
                Err(err) => self.err_msg = Some(err.to_string()),
            }
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
        for event in ctx.input().events.clone() {
            match event {
                egui::Event::Key {
                    key,
                    pressed,
                    modifiers,
                } => {
                    if key == egui::Key::N && pressed && modifiers.ctrl {
                        self.show_creation_dialog = true;
                    }
                }
                _ => (),
            }
        }

        if self.show_creation_dialog {
            match self.tmp_task {
                Some(_) => {
                    self.creation_dialog(ctx, frame);
                }
                None => self.tmp_task = Some(Task::default()),
            }
        }

        egui::SidePanel::left("Left Side").show(ctx, |ui| {
            ui.heading("Tasks");
            ui.columns(2, |cols| {
                if cols[0].button("New Task").clicked() {
                    self.show_creation_dialog = true;
                }
                if cols[1].button("Export").clicked() {
                    self.export();
                }
            });

            ui.separator();
            ui.columns(2, |cols| {
                if cols[0].button("Import").clicked() {
                    self.import();
                }
                if cols[1]
                    .selectable_label(self.squash_import, "Squash Existing on Import")
                    .clicked()
                {
                    self.squash_import = !self.squash_import;
                }
            });

            ui.separator();
            self.pomodoro_display(ctx, ui);
        });

        self.edit_pane(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("All Tasks");
                self.task_list(ui);
            });
        });

        self.err_win(ctx);
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
