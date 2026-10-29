use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type Snapshot = Value;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Interaction {
    Qr,
    Proximity,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Qr,
    Proximity,
    Either,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub impostors: usize,
    #[serde(default)]
    pub player_count: Option<usize>,
    #[serde(default)]
    pub activities_per_player: Option<usize>,
    pub kill_cooldown_secs: u64,
    pub emergency_cooldown_secs: u64,
    pub meeting_duration_secs: u64,
    pub kill_mode: Mode,
    pub task_mode: Mode,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            impostors: 1,
            player_count: None,
            activities_per_player: None,
            kill_cooldown_secs: 25,
            emergency_cooldown_secs: 30,
            meeting_duration_secs: 90,
            kill_mode: Mode::Either,
            task_mode: Mode::Either,
        }
    }
}
impl Settings {
    pub fn valid(&self) -> bool {
        self.impostors >= 1
            && self.activities_per_player.is_none_or(|count| count >= 1)
            && self.player_count.is_none_or(|count| count >= 3)
            && self.kill_cooldown_secs <= 600
            && self.emergency_cooldown_secs <= 600
            && (5..=600).contains(&self.meeting_duration_secs)
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    Join {
        name: String,
        room_code: Option<String>,
    },
    RenamePlayer {
        player_id: String,
        name: String,
    },
    Configure {
        settings: Settings,
    },
    SetStationLocation {
        station_id: String,
        location: String,
    },
    AddStation {
        kind: String,
        location: String,
    },
    RemoveStation {
        station_id: String,
    },
    Start,
    Reset,
    Demo,
    Kill {
        target_id: String,
        interaction: Interaction,
    },
    Report {
        target_id: String,
    },
    Emergency,
    Vote {
        target_id: Option<String>,
    },
    EndMeeting,
    StartTask {
        station_id: String,
        interaction: Interaction,
    },
    SubmitTask {
        session_id: String,
        result: Value,
    },
    CancelTask,
    Presence {
        first: String,
        second: String,
        near: bool,
        valid_for_ms: u64,
    },
    TestFeedback {
        target: String,
        kind: FeedbackKind,
    },
    DemoSession {
        player_id: String,
    },
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackKind {
    Killed,
    Meeting,
    TaskComplete,
    GameEnd,
}
#[derive(Debug, Deserialize)]
pub struct CommandRequest {
    pub token: Option<String>,
    pub command: Command,
}
#[derive(Debug, Serialize)]
pub struct CommandResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_id: Option<String>,
    pub state: Snapshot,
}
#[derive(Clone, Debug, Serialize)]
pub struct Presence {
    pub first: String,
    pub second: String,
    pub near: bool,
    pub expires_at: u64,
}
#[derive(Clone, Debug, Serialize)]
pub struct Feedback {
    pub id: String,
    pub at: u64,
    pub target: String,
    pub kind: String,
    pub message: String,
}
