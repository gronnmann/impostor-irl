mod feedback;
mod presence;
mod sessions;
#[cfg(test)]
mod tests;
use axum::{
    Json, Router,
    extract::{
        Query, State, WebSocketUpgrade,
        rejection::JsonRejection,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
    routing::{get, post},
};
use game::Game;
use protocol::{Command, CommandRequest, CommandResponse, Snapshot};
use rand::seq::SliceRandom;
use serde::Deserialize;
use serde_json::json;
use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::{Mutex, watch};
use tower_http::services::{ServeDir, ServeFile};

struct Backend {
    game: Game,
    sessions: sessions::Sessions,
    presence: presence::PresenceTable,
    feedback: feedback::FeedbackRouter,
}
impl Backend {
    fn new() -> Self {
        Self {
            game: Game::new(
                format!("game-{:016x}", rand::random::<u64>()),
                format!("{:04}", rand::random::<u16>() % 10000),
            ),
            sessions: Default::default(),
            presence: Default::default(),
            feedback: Default::default(),
        }
    }
    fn advance(&mut self, now: u64) {
        self.game.tick(now);
        self.presence.expire(now);
        self.feedback.route(&self.game);
    }
    fn snapshot(&mut self, token: Option<&str>, host: bool, now: u64) -> Snapshot {
        self.advance(now);
        let actor = self
            .sessions
            .resolve(token, now)
            .filter(|id| self.game.player(id).is_ok());
        let mut state = self.game.snapshot(actor.as_deref(), host, now, |id| {
            self.sessions.connected(id, now)
        });
        state["presence"] = json!(
            self.presence
                .0
                .iter()
                .filter(|p| host
                    || actor.as_deref() == Some(&p.first)
                    || actor.as_deref() == Some(&p.second))
                .collect::<Vec<_>>()
        );
        state["feedback"] = json!(
            self.feedback
                .recent
                .iter()
                .filter(|f| host || actor.as_deref() == Some(&f.target))
                .collect::<Vec<_>>()
        );
        state
    }
    fn command(&mut self, request: CommandRequest, now: u64) -> CommandResponse {
        self.advance(now);
        let actor = self.sessions.resolve(request.token.as_deref(), now);
        let mut new_token = None;
        let mut player_id = None;
        let result = (|| -> Result<(), String> {
            if request.token.is_some()
                && actor.is_none()
                && !matches!(request.command, Command::Join { .. })
            {
                return Err("Session expired; join again".into());
            }
            match &request.command {
                Command::Join { name, room_code } => {
                    if let Some(id) = &actor {
                        player_id = Some(id.clone());
                        new_token = request.token.clone();
                        return Ok(());
                    }
                    let name = name.trim();
                    if name.is_empty()
                        || name.chars().count() > 24
                        || name.chars().any(char::is_control)
                    {
                        return Err("Use a name of 1–24 characters".into());
                    }
                    if room_code
                        .as_deref()
                        .is_some_and(|code| !code.eq_ignore_ascii_case(&self.game.room_code))
                    {
                        return Err("Room code does not match".into());
                    }
                    let folded_name = name.to_lowercase();
                    if let Some(player) = self
                        .game
                        .players
                        .iter()
                        .find(|p| p.name.to_lowercase() == folded_name)
                    {
                        if room_code.is_none() {
                            return Err(
                                "Name already in use; supply the room code to rejoin".into()
                            );
                        }
                        player_id = Some(player.id.clone());
                        new_token = self.sessions.token_for(&player.id);
                        return Ok(());
                    }
                    let id = self.game.join(name, room_code.as_deref(), now)?;
                    new_token = Some(self.sessions.create(id.clone(), now));
                    player_id = Some(id);
                }
                Command::Start => {
                    let mut order = self
                        .game
                        .players
                        .iter()
                        .map(|p| p.id.clone())
                        .collect::<Vec<_>>();
                    order.shuffle(&mut rand::rng());
                    self.game.start(&order, now)?;
                }
                Command::Demo => {
                    self.sessions.clear();
                    self.presence.0.clear();
                    self.feedback.clear();
                    self.game.demo(now);
                    for p in &self.game.players {
                        self.sessions.create(p.id.clone(), now);
                    }
                }
                Command::Reset => {
                    self.game.reset(now);
                    self.presence.0.clear();
                    self.feedback.clear();
                }
                Command::DemoSession { player_id: id } => {
                    if !self.game.demo {
                        return Err("Session switching is available only in demo mode".into());
                    }
                    self.game.player(id)?;
                    new_token = self.sessions.token_for(id);
                    player_id = Some(id.clone());
                }
                Command::Presence {
                    first,
                    second,
                    near,
                    valid_for_ms,
                } => {
                    let exists = |id: &str| {
                        self.game.player(id).is_ok()
                            || self.game.stations.iter().any(|s| s.id == id)
                    };
                    if first == second || !exists(first) || !exists(second) {
                        return Err("Presence requires two distinct known subjects".into());
                    }
                    if !(1..=60_000).contains(valid_for_ms) {
                        return Err("Presence duration must be 1–60000 ms".into());
                    }
                    self.presence
                        .set(first.clone(), second.clone(), *near, *valid_for_ms, now);
                }
                Command::TestFeedback { target, kind } => {
                    if self.game.player(target).is_err()
                        && !self.game.stations.iter().any(|s| s.id == *target)
                        && target != "horn"
                    {
                        return Err("Unknown feedback target".into());
                    }
                    let kind = serde_json::to_value(kind).unwrap();
                    self.feedback.push(
                        target.clone(),
                        kind.as_str().unwrap(),
                        "Test notification".into(),
                        now,
                    );
                }
                command => {
                    let target = match command {
                        Command::Kill { target_id, .. } => Some(target_id.as_str()),
                        Command::StartTask { station_id, .. } => Some(station_id.as_str()),
                        _ => None,
                    };
                    let near = actor
                        .as_deref()
                        .zip(target)
                        .is_some_and(|(a, b)| self.presence.near(a, b, now));
                    self.game
                        .execute(actor.as_deref(), command, now, rand::random(), near)?;
                }
            }
            Ok(())
        })();
        if result.is_ok() {
            if let Command::RemoveStation { station_id } = &request.command {
                self.presence
                    .0
                    .retain(|p| p.first != *station_id && p.second != *station_id);
            }
        }
        self.feedback.route(&self.game);
        let host_command = matches!(
            request.command,
            Command::Configure { .. }
                | Command::RenamePlayer { .. }
                | Command::SetStationLocation { .. }
                | Command::AddStation { .. }
                | Command::RemoveStation { .. }
                | Command::Start
                | Command::Reset
                | Command::Demo
                | Command::EndMeeting
                | Command::Presence { .. }
                | Command::TestFeedback { .. }
        );
        let response_token = new_token.as_deref().or(request.token.as_deref());
        let state = self.snapshot(
            response_token,
            host_command && response_token.is_none(),
            now,
        );
        CommandResponse {
            ok: result.is_ok(),
            error: result.err(),
            token: new_token,
            player_id,
            state,
        }
    }
}
#[derive(Clone)]
struct App {
    backend: Arc<Mutex<Backend>>,
    changes: watch::Sender<u64>,
}
#[derive(Default, Deserialize)]
struct View {
    token: Option<String>,
    view: Option<String>,
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
fn app() -> (Router, App) {
    let (changes, _) = watch::channel(0);
    let state = App {
        backend: Arc::new(Mutex::new(Backend::new())),
        changes,
    };
    let dist = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web/dist");
    let router = Router::new()
        .route("/api/health", get(|| async { Json(json!({"ok":true})) }))
        .route("/api/state", get(get_state))
        .route("/api/command", post(command))
        .route("/api/ws", get(ws))
        .fallback_service(ServeDir::new(&dist).fallback(ServeFile::new(dist.join("index.html"))))
        .with_state(state.clone());
    (router, state)
}
async fn get_state(State(app): State<App>, Query(view): Query<View>) -> Json<Snapshot> {
    Json(app.backend.lock().await.snapshot(
        view.token.as_deref(),
        view.view.as_deref() == Some("host"),
        now(),
    ))
}
async fn command(
    State(app): State<App>,
    body: Result<Json<CommandRequest>, JsonRejection>,
) -> Json<CommandResponse> {
    let mut backend = app.backend.lock().await;
    let response = match body {
        Ok(Json(request)) => backend.command(request, now()),
        Err(error) => CommandResponse {
            ok: false,
            error: Some(error.body_text()),
            token: None,
            player_id: None,
            state: backend.snapshot(None, false, now()),
        },
    };
    app.changes.send_modify(|v| *v = v.wrapping_add(1));
    Json(response)
}
async fn ws(
    State(app): State<App>,
    Query(view): Query<View>,
    upgrade: WebSocketUpgrade,
) -> impl IntoResponse {
    upgrade.on_upgrade(move |socket| stream(socket, app, view))
}
async fn stream(mut socket: WebSocket, app: App, view: View) {
    let mut changes = app.changes.subscribe();
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    loop {
        let state = app.backend.lock().await.snapshot(
            view.token.as_deref(),
            view.view.as_deref() == Some("host"),
            now(),
        );
        let send = socket.send(Message::Text(
            json!({"type":"state","state":state}).to_string().into(),
        ));
        if !matches!(
            tokio::time::timeout(Duration::from_secs(5), send).await,
            Ok(Ok(()))
        ) {
            break;
        }
        tokio::select! {
            _=interval.tick()=>{},
            result=changes.changed()=>{if result.is_err(){break}},
            message=socket.recv()=>{if matches!(message,None|Some(Err(_))|Some(Ok(Message::Close(_)))){break}},
        }
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3001".into())
        .parse::<u16>()?;
    let (router, state) = app();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(250));
        loop {
            interval.tick().await;
            state.backend.lock().await.advance(now());
        }
    });
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    println!("Impostor server listening on http://0.0.0.0:{port}");
    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
