use std::collections::HashMap;

#[derive(Default)]
pub struct Sessions {
    entries: HashMap<String, (String, u64)>,
}
impl Sessions {
    pub fn create(&mut self, id: String, now: u64) -> String {
        let token = format!("{:032x}", rand::random::<u128>());
        self.entries.insert(token.clone(), (id, now));
        token
    }
    pub fn resolve(&mut self, token: Option<&str>, now: u64) -> Option<String> {
        let (id, seen) = self.entries.get_mut(token?)?;
        *seen = now;
        Some(id.clone())
    }
    pub fn token_for(&self, id: &str) -> Option<String> {
        self.entries
            .iter()
            .find(|(_, (p, _))| p == id)
            .map(|(t, _)| t.clone())
    }
    pub fn connected(&self, id: &str, now: u64) -> bool {
        self.entries
            .values()
            .any(|(p, seen)| p == id && now.saturating_sub(*seen) < 30_000)
    }
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
