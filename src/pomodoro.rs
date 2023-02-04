use chrono::{DateTime, Duration, Utc};

pub struct Pomodoro {
    pub start: DateTime<Utc>,
    pub work_time: Duration,
    pub break_time: Duration,
}

pub enum PomodoroStatus {
    Work(Duration),
    Break(Duration),
    Done,
}

impl Pomodoro {
    pub fn new(work_time: Duration, break_time: Duration) -> Self {
        Self {
            start: Utc::now(),
            work_time,
            break_time,
        }
    }

    pub fn status(&self) -> PomodoroStatus {
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
        Self::new(Duration::minutes(25), Duration::minutes(5))
    }
}
