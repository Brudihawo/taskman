use chrono::{DateTime, Local, Utc};
use uuid::Uuid;

use serde::de;
use serde::de::{Deserializer, MapAccess, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde::Deserialize;

use eframe::egui;
use egui::Color32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    id: Uuid,
    creation_time: DateTime<Utc>,
    pub name: String,
    pub description: String,
    started: Option<DateTime<Utc>>,
    finished: Option<DateTime<Utc>>,
    pub subtasks: Option<Vec<Uuid>>,
}

pub enum TaskStatus {
    NotYet,
    Started,
    Finished,
}

impl Task {
    const DATEFMT: &str = "%d.%m.%Y %H:%M:%S";

    pub fn get_uuid(&self) -> Uuid {
        self.id
    }
    pub fn get_creation_time(&self) -> DateTime<Utc> {
        self.creation_time
    }

    pub fn is_started(&self) -> bool {
        self.started.is_some()
    }

    pub fn is_finished(&self) -> bool {
        self.finished.is_some()
    }

    pub fn start(&mut self) {
        if self.is_started() || self.is_finished() {
            return;
        }

        self.started = Some(Utc::now());
    }

    pub fn finish(&mut self) {
        if !self.is_started() || self.is_finished() {
            return;
        }

        self.finished = Some(Utc::now());
    }

    pub fn get_duration(&self) -> Option<chrono::Duration> {
        if self.is_finished() {
            Some(self.finished.unwrap() - self.started.unwrap())
        } else {
            None
        }
    }

    pub fn status(&self) -> TaskStatus {
        match (self.is_started(), self.is_finished()) {
            (true, true) => TaskStatus::Finished,
            (true, false) => TaskStatus::Started,
            (false, true) => unreachable!(),
            (false, false) => TaskStatus::NotYet,
        }
    }

    pub fn display(&self, ui: &mut egui::Ui) -> bool {
        let mut clicked = false;
        ui.vertical(|ui| {
            clicked = clicked
                | ui.add(
                    egui::Label::new(
                        if let Some(dur) = self.get_duration() {
                            egui::RichText::new(format!(
                                "{} | {} -> {} (Took {:02}:{:02}:{:02})",
                                &DateTime::<Local>::from(self.creation_time).format(Task::DATEFMT),
                                &DateTime::<Local>::from(self.started.unwrap())
                                    .format(Task::DATEFMT),
                                &DateTime::<Local>::from(self.finished.unwrap())
                                    .format(Task::DATEFMT),
                                dur.num_hours(),
                                dur.num_minutes() - dur.num_hours() * 60,
                                dur.num_seconds() - dur.num_minutes() * 60 - dur.num_hours() * 3600
                            ))
                            .color(Color32::DARK_GREEN)
                        } else if let Some(begin) = self.started {
                            egui::RichText::new(format!(
                                "{} | {} -> ...",
                                &DateTime::<Local>::from(self.creation_time).format(Task::DATEFMT),
                                &DateTime::<Local>::from(begin).format(Task::DATEFMT),
                            ))
                            .color(Color32::from_rgb_additive(0x89, 0x38, 0x01))
                        } else {
                            egui::RichText::new(format!(
                                "{}",
                                &DateTime::<Local>::from(self.creation_time).format(Task::DATEFMT),
                            ))
                        }
                        .text_style(egui::TextStyle::Name("Smaller".into())),
                    )
                    .sense(egui::Sense::click()),
                )
                .clicked();

            clicked = clicked
                | ui.add(
                    egui::Label::new(
                        egui::RichText::new(format!("{}", &self.name))
                            .text_style(egui::TextStyle::Name("Heading2".into()))
                            .strong(),
                    )
                    .sense(egui::Sense::click()),
                )
                .double_clicked();

            clicked = clicked
                | ui.add(egui::Label::new(&self.description).sense(egui::Sense::click()))
                    .double_clicked();
        });
        clicked
    }
}

impl Default for Task {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            creation_time: Utc::now(),
            name: "New Task".to_string(),
            description: "".to_string(),
            started: None,
            finished: None,
            subtasks: None,
        }
    }
}

impl Serialize for Task {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Task", 6)?;
        s.serialize_field("id", &self.id.as_u128())?;
        s.serialize_field("creationtime", &self.creation_time)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("description", &self.description)?;
        s.serialize_field("started", &self.started)?;
        s.serialize_field("finished", &self.finished)?;
        s.serialize_field(
            "subtasks",
            &self
                .subtasks
                .as_ref()
                .map(|ids| ids.iter().map(|u| u.as_u128()).collect::<Vec<u128>>()),
        )?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for Task {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Id,
            CreationTime,
            Name,
            Description,
            Started,
            Finished,
            Subtasks,
        }

        struct TaskVisitor;

        impl<'de> Visitor<'de> for TaskVisitor {
            type Value = Task;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Task")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let u_id: u128 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let creation_time: DateTime<Utc> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let name: String = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let description: String = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                let started: Option<DateTime<Utc>> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(4, &self))?;
                let finished: Option<DateTime<Utc>> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(5, &self))?;
                let u_subtasks: Option<Vec<u128>> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(6, &self))?;

                let id = Uuid::from_u128(u_id);
                let subtasks = u_subtasks
                    .map(|u| u.iter().map(|&x| Uuid::from_u128(x)).collect::<Vec<Uuid>>());

                Ok(Self::Value {
                    id,
                    creation_time,
                    name,
                    description,
                    started,
                    finished,
                    subtasks,
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Task, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut id = None;
                let mut creation_time = None;
                let mut name = None;
                let mut description = None;
                let mut started = None;
                let mut finished = None;
                let mut subtasks = None;
                while let Some(key) = map.next_key::<Field>()? {
                    match key {
                        Field::Id => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value::<u128>()?);
                        }
                        Field::CreationTime => {
                            if creation_time.is_some() {
                                return Err(de::Error::duplicate_field("creationtime"));
                            }
                            creation_time = Some(map.next_value::<DateTime<Utc>>()?);
                        }
                        Field::Name => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        Field::Description => {
                            if description.is_some() {
                                return Err(de::Error::duplicate_field("description"));
                            }
                            description = Some(map.next_value()?);
                        }
                        Field::Started => {
                            if started.is_some() {
                                return Err(de::Error::duplicate_field("started"));
                            }
                            started = Some(map.next_value()?);
                        }
                        Field::Finished => {
                            if finished.is_some() {
                                return Err(de::Error::duplicate_field("finished"));
                            }
                            finished = Some(map.next_value()?);
                        }
                        Field::Subtasks => {
                            if subtasks.is_some() {
                                return Err(de::Error::duplicate_field("subtasks"));
                            }
                            subtasks = Some(map.next_value::<Option<Vec<u128>>>()?);
                        }
                    }
                }

                Ok(Task {
                    id: id
                        .map(|x| Uuid::from_u128(x))
                        .ok_or_else(|| de::Error::missing_field("id"))?,
                    creation_time: creation_time
                        .ok_or_else(|| de::Error::missing_field("creationtime"))?,
                    name: name.ok_or_else(|| de::Error::missing_field("name"))?,
                    description: description
                        .ok_or_else(|| de::Error::missing_field("description"))?,
                    started: started.ok_or_else(|| de::Error::missing_field("started"))?,
                    finished: finished.ok_or_else(|| de::Error::missing_field("finished"))?,
                    subtasks: subtasks
                        .map(|o| {
                            o.map(|x| x.iter().map(|u| Uuid::from_u128(*u)).collect::<Vec<Uuid>>())
                        })
                        .ok_or_else(|| de::Error::missing_field("subtasks"))?,
                })
            }
        }

        const FIELDS: &'static [&'static str] = &[
            "id",
            "name",
            "description",
            "started",
            "finished",
            "subtasks",
        ];

        deserializer.deserialize_struct("task", FIELDS, TaskVisitor)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ser_de() {
        let task = Task::default();
        let ser_d = serde_json::to_string(&task).unwrap();
        let des_d = serde_json::from_str::<Task>(&ser_d).unwrap();
        assert_eq!(task, des_d)
    }
}
