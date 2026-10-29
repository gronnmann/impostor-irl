use protocol::Presence;

#[derive(Default)]
pub struct PresenceTable(pub Vec<Presence>);
impl PresenceTable {
    pub fn expire(&mut self, now: u64) {
        self.0.retain(|p| p.expires_at > now);
    }
    pub fn set(
        &mut self,
        mut first: String,
        mut second: String,
        near: bool,
        valid_for_ms: u64,
        now: u64,
    ) {
        if first > second {
            std::mem::swap(&mut first, &mut second);
        }
        self.0.retain(|p| p.first != first || p.second != second);
        self.0.push(Presence {
            first,
            second,
            near,
            expires_at: now.saturating_add(valid_for_ms),
        });
    }
    pub fn near(&self, a: &str, b: &str, now: u64) -> bool {
        self.0.iter().any(|p| {
            p.near
                && p.expires_at > now
                && ((p.first == a && p.second == b) || (p.first == b && p.second == a))
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expiry_and_unordered_updates() {
        let mut p = PresenceTable::default();
        p.set("b".into(), "a".into(), true, 100, 0);
        assert!(p.near("a", "b", 99));
        assert!(!p.near("a", "b", 100));
        p.set("a".into(), "b".into(), false, 100, 50);
        assert_eq!(p.0.len(), 1);
        assert!(!p.near("b", "a", 51));
        p.expire(150);
        assert!(p.0.is_empty());
    }
}
