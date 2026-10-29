pub mod activities;
use protocol::{Command, Interaction, Mode, Settings, Snapshot};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Crew,
    Impostor,
}
#[derive(Clone, Debug)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub color: String,
    pub role: Option<Role>,
    pub alive: bool,
    pub body: bool,
    pub kill_ready_at: u64,
}
#[derive(Clone, Debug, Serialize)]
pub struct Task {
    pub id: String,
    pub player_id: String,
    pub station_id: String,
    pub kind: String,
    pub completed: bool,
}
#[derive(Clone, Debug, Serialize)]
pub struct Attempt {
    pub id: String,
    pub task_id: String,
    pub station_id: String,
    pub kind: String,
    pub challenge: Value,
}
#[derive(Clone, Debug)]
pub struct Event {
    pub id: String,
    pub at: u64,
    pub kind: String,
    pub message: String,
    pub target: Option<String>,
}
#[derive(Clone, Debug)]
struct Meeting {
    reason: String,
    victim_id: Option<String>,
    ends_at: u64,
    votes: BTreeMap<String, Option<String>>,
}
pub struct Game {
    pub game_id: String,
    pub room_code: String,
    pub phase: String,
    pub settings: Settings,
    pub players: Vec<Player>,
    pub tasks: Vec<Task>,
    pub attempts: BTreeMap<String, Attempt>,
    pub stations: Vec<activities::Station>,
    meeting: Option<Meeting>,
    pub last_meeting: Option<Value>,
    pub winner: Option<Role>,
    pub win_reason: Option<String>,
    pub emergency_ready_at: u64,
    pub demo: bool,
    pub events: Vec<Event>,
    serial: u64,
}
impl Default for Game {
    fn default() -> Self {
        Self::new("game-1".into(), "SHIP".into())
    }
}
impl Game {
    pub fn new(game_id: String, room_code: String) -> Self {
        Self {
            game_id,
            room_code,
            phase: "lobby".into(),
            settings: Settings::default(),
            players: vec![],
            tasks: vec![],
            attempts: BTreeMap::new(),
            stations: activities::stations(),
            meeting: None,
            last_meeting: None,
            winner: None,
            win_reason: None,
            emergency_ready_at: 0,
            demo: false,
            events: vec![],
            serial: 0,
        }
    }
    fn id(&mut self, prefix: &str) -> String {
        self.serial += 1;
        format!("{prefix}-{}", self.serial)
    }
    fn event(&mut self, now: u64, kind: &str, message: String, target: Option<String>) {
        let id = self.id("event");
        self.events.push(Event {
            id,
            at: now,
            kind: kind.into(),
            message,
            target,
        });
        if self.events.len() > 100 {
            self.events.remove(0);
        }
    }
    pub fn player(&self, id: &str) -> Result<&Player, String> {
        self.players
            .iter()
            .find(|p| p.id == id)
            .ok_or("Unknown player".into())
    }
    pub fn join(&mut self, name: &str, room: Option<&str>, now: u64) -> Result<String, String> {
        if self.phase != "lobby" {
            return Err("The round has already started".into());
        }
        if room.is_some_and(|r| !r.eq_ignore_ascii_case(&self.room_code)) {
            return Err("Room code does not match".into());
        }
        let name = Self::validated_name(name)?;
        if self
            .settings
            .player_count
            .is_some_and(|count| self.players.len() >= count)
        {
            return Err("Lobby is full".into());
        }
        let id = self.id("player");
        let palette = [
            "#34d5e7", "#f56575", "#f5bf4f", "#ab83ff", "#65d99b", "#f299ce",
        ];
        let color = if self.players.len() < palette.len() {
            palette[self.players.len()].into()
        } else {
            // Distinct additional suits for larger lobbies, rather than repeating six colors.
            format!("hsl({} 72% 58%)", (self.players.len() * 137) % 360)
        };
        self.players.push(Player {
            id: id.clone(),
            name: name.into(),
            color,
            role: None,
            alive: true,
            body: false,
            kill_ready_at: 0,
        });
        self.event(
            now,
            "player_joined",
            format!("{name} joined the lobby"),
            None,
        );
        Ok(id)
    }
    fn validated_name(name: &str) -> Result<&str, String> {
        let name = name.trim();
        if name.is_empty() || name.chars().count() > 24 || name.chars().any(char::is_control) {
            return Err("Use a name of 1–24 characters".into());
        }
        Ok(name)
    }
    pub fn reset(&mut self, now: u64) {
        self.phase = "lobby".into();
        self.tasks.clear();
        self.attempts.clear();
        self.meeting = None;
        self.last_meeting = None;
        self.winner = None;
        self.win_reason = None;
        self.emergency_ready_at = 0;
        self.demo = false;
        self.events.clear();
        for p in &mut self.players {
            p.role = None;
            p.alive = true;
            p.body = false;
            p.kill_ready_at = 0;
        }
        self.event(now, "game_reset", "Returned to lobby".into(), None);
    }
    pub fn start(&mut self, order: &[String], now: u64) -> Result<(), String> {
        if self.phase != "lobby" {
            return Err("Start requires the lobby".into());
        }
        if self
            .settings
            .player_count
            .is_some_and(|count| count != self.players.len())
        {
            return Err("Join the configured player count before starting".into());
        }
        if !self.settings.valid()
            || self.players.len() < 3
            || self.settings.impostors >= self.players.len().div_ceil(2)
        {
            return Err("Need at least three players and more crew than impostors".into());
        }
        if order.len() != self.players.len()
            || self
                .players
                .iter()
                .any(|p| order.iter().filter(|id| **id == p.id).count() != 1)
        {
            return Err("Invalid role assignment".into());
        }
        if self.stations.is_empty()
            || self
                .settings
                .activities_per_player
                .is_some_and(|n| n > self.stations.len())
        {
            return Err(
                "Add enough activity placements for the activities per player setting".into(),
            );
        }
        for p in &mut self.players {
            p.role = Some(if order[..self.settings.impostors].contains(&p.id) {
                Role::Impostor
            } else {
                Role::Crew
            });
            p.alive = true;
            p.body = false;
            p.kill_ready_at = now + self.settings.kill_cooldown_secs * 1000;
        }
        self.tasks = self
            .players
            .iter()
            .filter(|p| p.role == Some(Role::Crew))
            .enumerate()
            .flat_map(|(index, p)| {
                let count = self
                    .settings
                    .activities_per_player
                    .unwrap_or(self.stations.len());
                self.stations
                    .iter()
                    .cycle()
                    .skip(index * count % self.stations.len())
                    .take(count)
                    .map(move |station| Task {
                        id: format!("{}-{}", p.id, station.id),
                        player_id: p.id.clone(),
                        station_id: station.id.clone(),
                        kind: station.kind.clone(),
                        completed: false,
                    })
            })
            .collect();
        self.phase = "playing".into();
        self.emergency_ready_at = now + self.settings.emergency_cooldown_secs * 1000;
        self.event(now, "game_started", "The round has started".into(), None);
        Ok(())
    }
    pub fn demo(&mut self, now: u64) {
        self.reset(now);
        self.players.clear();
        self.settings = Settings::default();
        for name in ["Nova", "Atlas", "Echo", "Luna", "Orion", "Vega"] {
            self.join(name, None, now).unwrap();
        }
        let order = self
            .players
            .iter()
            .map(|p| p.id.clone())
            .collect::<Vec<_>>();
        self.start(&order, now).unwrap();
        self.demo = true;
        for p in &mut self.players {
            p.kill_ready_at = now;
        }
        self.emergency_ready_at = now;
    }
    fn station_setup_allowed(&self) -> Result<(), String> {
        if self.phase == "lobby" {
            Ok(())
        } else {
            Err("Station setup requires the lobby".into())
        }
    }
    fn validated_location(location: &str) -> Result<&str, String> {
        if location.chars().any(char::is_control) || location.trim().chars().count() > 80 {
            return Err("Use a location of at most 80 characters without controls".into());
        }
        Ok(location.trim())
    }
    fn playing(&self) -> Result<(), String> {
        if self.phase == "playing" {
            Ok(())
        } else {
            Err("Play is not active".into())
        }
    }
    fn living(&self, id: &str) -> Result<&Player, String> {
        let p = self.player(id)?;
        if p.alive {
            Ok(p)
        } else {
            Err("Dead players cannot do that".into())
        }
    }
    fn interaction(mode: Mode, interaction: Interaction, near: bool) -> Result<(), String> {
        if (mode == Mode::Qr && interaction != Interaction::Qr)
            || (mode == Mode::Proximity && interaction != Interaction::Proximity)
        {
            return Err("Interaction is disabled by settings".into());
        }
        if interaction == Interaction::Proximity && !near {
            return Err("Current nearby presence is required".into());
        }
        Ok(())
    }
    pub fn execute(
        &mut self,
        actor: Option<&str>,
        command: &Command,
        now: u64,
        seed: u64,
        near: bool,
    ) -> Result<(), String> {
        self.tick(now);
        match command {
            Command::RenamePlayer { player_id, name } => {
                if self.phase != "lobby" && !self.demo {
                    return Err("Renaming requires the lobby or demo mode".into());
                }
                let name = Self::validated_name(name)?;
                if self
                    .players
                    .iter()
                    .any(|p| p.id != *player_id && p.name.to_lowercase() == name.to_lowercase())
                {
                    return Err("Choose a name that is not already used by another player".into());
                }
                let player = self
                    .players
                    .iter_mut()
                    .find(|p| p.id == *player_id)
                    .ok_or("Unknown player")?;
                player.name = name.into();
            }
            Command::Configure { settings } => {
                if self.phase != "lobby" {
                    return Err("Configure requires the lobby".into());
                }
                if !settings.valid() {
                    return Err(
                        "Invalid settings: timers 0–600s, meeting 5–600s, impostors >=1, player count >=3 or automatic".into(),
                    );
                }
                if settings
                    .player_count
                    .is_some_and(|count| count < self.players.len())
                {
                    return Err("Player count cannot be below the joined roster".into());
                }
                self.settings = settings.clone();
            }
            Command::SetStationLocation {
                station_id,
                location,
            } => {
                self.station_setup_allowed()?;
                let location = Self::validated_location(location)?.to_owned();
                self.stations
                    .iter_mut()
                    .find(|s| s.id == *station_id)
                    .ok_or("Unknown station")?
                    .location = location;
            }
            Command::AddStation { kind, location } => {
                self.station_setup_allowed()?;
                let location = Self::validated_location(location)?.to_owned();
                let mut station = activities::stations()
                    .into_iter()
                    .find(|s| s.kind == *kind)
                    .ok_or("Unknown activity kind")?;
                station.id = self.id("station");
                station.location = location;
                self.stations.push(station);
            }
            Command::RemoveStation { station_id } => {
                self.station_setup_allowed()?;
                let index = self
                    .stations
                    .iter()
                    .position(|s| s.id == *station_id)
                    .ok_or("Unknown station")?;
                self.stations.remove(index);
            }
            Command::Reset => self.reset(now),
            Command::Demo => self.demo(now),
            Command::EndMeeting => {
                if self.meeting.is_none() {
                    return Err("No meeting is active".into());
                }
                self.finish_meeting(now);
            }
            Command::Emergency => {
                self.playing()?;
                if let Some(id) = actor {
                    self.living(id)?;
                }
                if now < self.emergency_ready_at {
                    return Err("Emergency button is cooling down".into());
                }
                self.begin_meeting("emergency", None, now);
            }
            Command::Kill {
                target_id,
                interaction,
            } => {
                self.playing()?;
                let id = actor.ok_or("Player session required")?;
                let p = self.living(id)?;
                if p.role != Some(Role::Impostor) {
                    return Err("Only impostors can kill".into());
                }
                if now < p.kill_ready_at {
                    return Err("Kill is cooling down".into());
                }
                let victim = self.living(target_id)?;
                if victim.role != Some(Role::Crew) {
                    return Err("Target must be living crew".into());
                }
                Self::interaction(self.settings.kill_mode, *interaction, near)?;
                let name = victim.name.clone();
                let p = self
                    .players
                    .iter_mut()
                    .find(|p| p.id == *target_id)
                    .unwrap();
                p.alive = false;
                p.body = true;
                self.attempts.remove(target_id);
                self.players
                    .iter_mut()
                    .find(|p| p.id == id)
                    .unwrap()
                    .kill_ready_at = now + self.settings.kill_cooldown_secs * 1000;
                self.event(
                    now,
                    "killed",
                    format!("{name} was killed"),
                    Some(target_id.clone()),
                );
                self.check_win(now);
            }
            Command::Report { target_id } => {
                self.playing()?;
                self.living(actor.ok_or("Player session required")?)?;
                if !self.player(target_id)?.body {
                    return Err("That player has no reportable body".into());
                }
                self.begin_meeting("body", Some(target_id.clone()), now);
            }
            Command::Vote { target_id } => {
                let id = actor.ok_or("Player session required")?;
                self.living(id)?;
                if let Some(target) = target_id {
                    self.living(target)?;
                }
                let meeting = self.meeting.as_mut().ok_or("No meeting is active")?;
                if meeting.votes.contains_key(id) {
                    return Err("You have already voted".into());
                }
                meeting.votes.insert(id.into(), target_id.clone());
                let all = meeting.votes.len() == self.players.iter().filter(|p| p.alive).count();
                self.event(now, "vote_cast", "A vote was cast".into(), None);
                if all {
                    self.finish_meeting(now);
                }
            }
            Command::StartTask {
                station_id,
                interaction,
            } => {
                self.playing()?;
                let id = actor.ok_or("Player session required")?;
                let p = self.player(id)?;
                if p.body {
                    return Err("Wait for a meeting to clear your body".into());
                }
                let kind = self
                    .stations
                    .iter()
                    .find(|s| s.id == *station_id)
                    .ok_or("Unknown station")?
                    .kind
                    .clone();
                Self::interaction(self.settings.task_mode, *interaction, near)?;
                let task_id = if p.role == Some(Role::Impostor) {
                    format!("fake-{id}-{station_id}")
                } else {
                    let task = self
                        .tasks
                        .iter()
                        .find(|t| t.player_id == id && t.station_id == *station_id)
                        .ok_or("Task not assigned")?;
                    if task.completed {
                        return Err("Task already complete".into());
                    }
                    task.id.clone()
                };
                let attempt = Attempt {
                    id: self.id("session"),
                    task_id,
                    station_id: station_id.clone(),
                    challenge: activities::challenge(&kind, seed),
                    kind,
                };
                self.attempts.insert(id.into(), attempt);
            }
            Command::SubmitTask { session_id, result } => {
                self.playing()?;
                let id = actor.ok_or("Player session required")?;
                let attempt = self.attempts.get(id).ok_or("No active task")?;
                if attempt.id != *session_id {
                    return Err("Task session is stale".into());
                }
                if !activities::valid(&attempt.kind, &attempt.challenge, result) {
                    return Err("Incorrect result; try again".into());
                }
                let task_id = attempt.task_id.clone();
                let kind = attempt.kind.clone();
                if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                    task.completed = true;
                }
                self.attempts.remove(id);
                self.event(
                    now,
                    "task_complete",
                    format!("Completed {kind}"),
                    Some(id.into()),
                );
                self.check_win(now);
            }
            Command::CancelTask => {
                let id = actor.ok_or("Player session required")?;
                self.attempts.remove(id);
            }
            _ => return Err("Command belongs to the server adapter".into()),
        }
        Ok(())
    }
    fn begin_meeting(&mut self, reason: &str, victim_id: Option<String>, now: u64) {
        self.phase = "meeting".into();
        self.attempts.clear();
        for p in &mut self.players {
            p.body = false;
        }
        self.meeting = Some(Meeting {
            reason: reason.into(),
            victim_id,
            ends_at: now + self.settings.meeting_duration_secs * 1000,
            votes: BTreeMap::new(),
        });
        self.event(
            now,
            "meeting",
            "Meeting called — gather at the meeting point".into(),
            None,
        );
    }
    pub fn tick(&mut self, now: u64) {
        if self.meeting.as_ref().is_some_and(|m| now >= m.ends_at) {
            self.finish_meeting(now);
        }
    }
    fn finish_meeting(&mut self, now: u64) {
        let Some(meeting) = self.meeting.take() else {
            return;
        };
        let mut tally: BTreeMap<String, usize> = BTreeMap::new();
        for target in meeting.votes.values() {
            *tally
                .entry(target.clone().unwrap_or("skip".into()))
                .or_default() += 1;
        }
        let max = tally.values().max().copied().unwrap_or(0);
        let leaders: Vec<_> = tally
            .iter()
            .filter(|(_, count)| **count == max)
            .map(|(id, _)| id.clone())
            .collect();
        let ejected = if leaders.len() == 1 && leaders[0] != "skip" {
            Some(leaders[0].clone())
        } else {
            None
        };
        if let Some(id) = &ejected {
            self.players.iter_mut().find(|p| p.id == *id).unwrap().alive = false;
        }
        self.last_meeting =
            Some(json!({"ejected_id":ejected,"skipped":ejected.is_none(),"tally":tally}));
        self.phase = "playing".into();
        self.emergency_ready_at = now + self.settings.emergency_cooldown_secs * 1000;
        self.event(
            now,
            "meeting_ended",
            if let Some(id) = ejected {
                format!("{} was ejected", self.player(&id).unwrap().name)
            } else {
                "No one was ejected".into()
            },
            None,
        );
        self.check_win(now);
    }
    fn check_win(&mut self, now: u64) {
        let impostors = self
            .players
            .iter()
            .filter(|p| p.alive && p.role == Some(Role::Impostor))
            .count();
        let crew = self
            .players
            .iter()
            .filter(|p| p.alive && p.role == Some(Role::Crew))
            .count();
        let result = if impostors == 0 {
            Some((Role::Crew, "All impostors were eliminated"))
        } else if !self.tasks.is_empty() && self.tasks.iter().all(|t| t.completed) {
            Some((Role::Crew, "All crew tasks are complete"))
        } else if impostors >= crew {
            Some((Role::Impostor, "Impostors have reached parity"))
        } else {
            None
        };
        if let Some((winner, reason)) = result {
            self.winner = Some(winner);
            self.win_reason = Some(reason.into());
            self.phase = "ended".into();
            self.attempts.clear();
            self.meeting = None;
            self.event(now, "game_end", reason.into(), None);
        }
    }
    pub fn snapshot(
        &self,
        actor: Option<&str>,
        host: bool,
        now: u64,
        connected: impl Fn(&str) -> bool,
    ) -> Snapshot {
        let ended = self.phase == "ended";
        let players: Vec<_> = self
            .players
            .iter()
            .map(|p| {
                let mut v =
                    json!({"id":p.id,"name":p.name,"color":p.color,"connected":connected(&p.id)});
                if host || ended || self.phase == "meeting" {
                    v["alive"] = json!(p.alive);
                }
                if host || ended {
                    v["role"] = json!(p.role);
                }
                v
            })
            .collect();
        let me=actor.and_then(|id|self.player(id).ok()).map(|p|json!({"id":p.id,"name":p.name,"role":p.role,"alive":p.alive,"kill_ready_at":p.kill_ready_at}));
        let tasks: Vec<_> = self
            .tasks
            .iter()
            .filter(|t| host || actor == Some(t.player_id.as_str()))
            .collect();
        let meeting=self.meeting.as_ref().map(|m|{let mut v=json!({"reason":m.reason,"ends_at":m.ends_at,"voted_player_ids":m.votes.keys().collect::<Vec<_>>()});if let Some(id)=&m.victim_id {v["victim_id"]=json!(id);}if host {v["votes"]=json!(m.votes);}v});
        let events: Vec<_> = self
            .events
            .iter()
            .filter(|e| {
                host || e.target.is_none() || (e.kind != "killed" && e.target.as_deref() == actor)
            })
            .map(|e| json!({"id":e.id,"at":e.at,"kind":e.kind,"message":e.message}))
            .collect();
        json!({"game_id":self.game_id,"room_code":self.room_code,"phase":self.phase,"now":now,"settings":self.settings,"players":players,"me":me,"stations":self.stations,"available_activities":activities::stations(),"tasks":tasks,"active_task":actor.and_then(|id|self.attempts.get(id)),"meeting":meeting,"last_meeting":self.last_meeting,"winner":self.winner,"win_reason":self.win_reason,"task_progress":{"completed":self.tasks.iter().filter(|t|t.completed).count(),"total":self.tasks.len()},"emergency_ready_at":self.emergency_ready_at,"presence":[],"feedback":[],"events":events,"demo":self.demo})
    }
}
