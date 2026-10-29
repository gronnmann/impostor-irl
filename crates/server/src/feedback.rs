use game::Game;
use protocol::Feedback;

#[derive(Default)]
pub struct FeedbackRouter {
    pub recent: Vec<Feedback>,
    last_event: Option<String>,
    serial: u64,
}
impl FeedbackRouter {
    pub fn clear(&mut self) {
        self.recent.clear();
        self.last_event = None;
    }
    pub fn push(&mut self, target: String, kind: &str, message: String, now: u64) {
        self.serial += 1;
        self.recent.push(Feedback {
            id: format!("feedback-{}", self.serial),
            at: now,
            target,
            kind: kind.into(),
            message,
        });
        if self.recent.len() > 100 {
            self.recent.remove(0);
        }
    }
    pub fn route(&mut self, game: &Game) {
        let start = self
            .last_event
            .as_ref()
            .and_then(|id| game.events.iter().position(|e| &e.id == id))
            .map_or(0, |i| i + 1);
        for event in &game.events[start..] {
            match event.kind.as_str() {
                "killed" | "task_complete" => {
                    if let Some(target) = &event.target {
                        self.push(target.clone(), &event.kind, event.message.clone(), event.at);
                    }
                }
                "meeting" | "game_end" => {
                    for player in &game.players {
                        self.push(
                            player.id.clone(),
                            &event.kind,
                            event.message.clone(),
                            event.at,
                        );
                    }
                    if event.kind == "meeting" {
                        self.push(
                            "horn".into(),
                            "meeting",
                            "Sound meeting horn".into(),
                            event.at,
                        );
                    }
                }
                _ => {}
            }
        }
        self.last_event = game.events.last().map(|e| e.id.clone());
    }
}
