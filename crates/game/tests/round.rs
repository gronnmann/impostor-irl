use game::{Game, Role, activities};
use protocol::{Command, Interaction, Mode};
use serde_json::{Value, json};

fn demo() -> Game {
    let mut g = Game::default();
    g.demo(1000);
    g
}
fn act(g: &mut Game, index: usize, command: Command, now: u64) -> Result<(), String> {
    let id = g.players[index].id.clone();
    g.execute(Some(&id), &command, now, 97, true)
}
fn kill(g: &mut Game, index: usize, now: u64) -> Result<(), String> {
    let target_id = g.players[index].id.clone();
    act(
        g,
        0,
        Command::Kill {
            target_id,
            interaction: Interaction::Qr,
        },
        now,
    )
}
fn solution(kind: &str, c: &Value) -> Value {
    match kind {
        "wires" => {
            let order = c["right_order"].as_array().unwrap();
            json!({"connections":(0..4).map(|i|order.iter().position(|v|v==i).unwrap()).collect::<Vec<_>>()})
        }
        "code" | "breaker" => json!({"answer":c["code"]}),
        "signal" | "frequency" => json!({"value":c["target"],"held_ms":c["hold_ms"]}),
        "battery" | "timing" => json!({"hits":[c["zone_start"],c["zone_start"],c["zone_start"]]}),
        "memory" => json!({"sequence":c["sequence"]}),
        "anomaly" => json!({"index":c["odd_index"]}),
        "circuits" | "route_power" => json!({"rotations":c["solution"]}),
        "password" => json!({"answer":c["answer"]}),
        "mixing" => json!({"a":c["target_a"],"b":c["target_b"],"held_ms":c["hold_ms"]}),
        "transfer" => json!({"progress":100,"held_ms":c["required_ms"]}),
        "fault" => json!({"index":c["fault_index"]}),
        "switches" => json!({"sequence":c["order"]}),
        "cargo" => {
            json!({"choices":c["items"].as_array().unwrap().iter().map(|i|i["category"].clone()).collect::<Vec<_>>()})
        }
        _ => unreachable!(),
    }
}
fn complete(g: &mut Game, index: usize, kind: &str, now: u64) {
    act(
        g,
        index,
        Command::StartTask {
            station_id: kind.into(),
            interaction: Interaction::Qr,
        },
        now,
    )
    .unwrap();
    let id = g.players[index].id.clone();
    let a = g.attempts[&id].clone();
    assert!(
        act(
            g,
            index,
            Command::SubmitTask {
                session_id: "stale-session".into(),
                result: solution(&a.kind, &a.challenge),
            },
            now
        )
        .is_err()
    );
    let result = solution(&a.kind, &a.challenge);
    act(
        g,
        index,
        Command::SubmitTask {
            session_id: a.id,
            result,
        },
        now,
    )
    .unwrap();
}

#[test]
fn roles_roster_and_reset() {
    let mut g = Game::default();
    assert!(g.start(&[], 0).is_err());
    assert!(g.join("  ", None, 0).is_err());
    assert!(g.join("A", Some("bad"), 0).is_err());
    for n in ["A", "B", "C"] {
        g.join(n, None, 0).unwrap();
    }
    let order = g
        .players
        .iter()
        .rev()
        .map(|p| p.id.clone())
        .collect::<Vec<_>>();
    g.start(&order, 100).unwrap();
    assert_eq!(g.players[2].role, Some(Role::Impostor));
    assert_eq!(g.tasks.len(), 2 * g.stations.len());
    assert!(g.join("Late", None, 0).is_err());
    g.reset(200);
    assert_eq!(g.players.len(), 3);
    assert!(g.players.iter().all(|p| p.role.is_none()));
    assert!(g.tasks.is_empty());
    assert!(!g.demo);
}
#[test]
fn rename_validates_names_and_phase_without_changing_player_state() {
    let mut g = Game::default();
    let id = g.join("Alex", None, 1000).unwrap();
    let rename = |g: &mut Game, player_id: &str, name: &str| {
        g.execute(
            None,
            &Command::RenamePlayer {
                player_id: player_id.into(),
                name: name.into(),
            },
            1000,
            0,
            false,
        )
    };
    let unicode = "ø".repeat(24);
    rename(&mut g, &id, &format!("  {unicode}  ")).unwrap();
    assert_eq!(g.player(&id).unwrap().name, unicode);
    for invalid in [
        "".into(),
        "   ".into(),
        "ø".repeat(25),
        "A\nB".into(),
        "A\0B".into(),
        "A\u{7f}".into(),
    ] {
        let before = g.snapshot(Some(&id), true, 1000, |_| true);
        assert!(rename(&mut g, &id, &invalid).is_err());
        assert_eq!(g.snapshot(Some(&id), true, 1000, |_| true), before);
        assert!(Game::default().join(&invalid, None, 1000).is_err());
    }
    let before = g.snapshot(None, true, 1000, |_| true);
    assert_eq!(
        rename(&mut g, "unknown", "Name").unwrap_err(),
        "Unknown player"
    );
    assert_eq!(g.snapshot(None, true, 1000, |_| true), before);
    for phase in ["playing", "meeting", "ended"] {
        g.phase = phase.into();
        let before = g.snapshot(Some(&id), true, 1000, |_| true);
        assert!(rename(&mut g, &id, "Changed").is_err());
        assert_eq!(g.snapshot(Some(&id), true, 1000, |_| true), before);
    }

    let mut g = demo();
    let id = g.players[1].id.clone();
    complete(&mut g, 1, "code", 1000);
    act(
        &mut g,
        1,
        Command::StartTask {
            station_id: "memory".into(),
            interaction: Interaction::Qr,
        },
        1000,
    )
    .unwrap();
    let player = g.player(&id).unwrap().clone();
    let mut expected = g.snapshot(Some(&id), true, 1000, |_| true);
    expected["players"][1]["name"] = json!("Renamed");
    expected["me"]["name"] = json!("Renamed");
    rename(&mut g, &id, " Renamed ").unwrap();
    assert_eq!(g.snapshot(Some(&id), true, 1000, |_| true), expected);
    let renamed = g.player(&id).unwrap();
    assert_eq!(
        (
            &renamed.id,
            &renamed.color,
            renamed.role,
            renamed.alive,
            renamed.body,
            renamed.kill_ready_at
        ),
        (
            &player.id,
            &player.color,
            player.role,
            player.alive,
            player.body,
            player.kill_ready_at
        )
    );
    assert_eq!(expected["task_progress"]["completed"], 1);
    let attempt = g.attempts[&id].clone();
    act(
        &mut g,
        1,
        Command::SubmitTask {
            session_id: attempt.id,
            result: solution("memory", &attempt.challenge),
        },
        1000,
    )
    .unwrap();
    for phase in ["meeting", "ended"] {
        g.phase = phase.into();
        rename(&mut g, &id, phase).unwrap();
        assert_eq!(g.player(&id).unwrap().name, phase);
    }
}
#[test]
fn station_locations_validate_and_survive_round_changes() {
    let mut g = Game::default();
    let set = |g: &mut Game, station: &str, location: &str| {
        g.execute(
            None,
            &Command::SetStationLocation {
                station_id: station.into(),
                location: location.into(),
            },
            1000,
            0,
            false,
        )
    };
    let stations = |g: &Game, host| g.snapshot(None, host, 1000, |_| true)["stations"].clone();
    assert_eq!(
        stations(&g, true).as_array().unwrap().len(),
        g.stations.len()
    );
    assert!(
        stations(&g, false)
            .as_array()
            .unwrap()
            .iter()
            .all(|s| s["location"] == "")
    );
    set(&mut g, "wires", "  Scenerommet  ").unwrap();
    for host in [false, true] {
        assert_eq!(stations(&g, host)[0]["location"], "Scenerommet");
        assert_eq!(stations(&g, host)[0]["name"], "Connect Wires");
        assert_eq!(stations(&g, host)[0]["id"], "wires");
    }
    for invalid in [
        "x".repeat(81),
        "Room\nA".into(),
        "\tRoom".into(),
        "Room\u{7f}".into(),
    ] {
        assert!(set(&mut g, "wires", &invalid).is_err());
        assert_eq!(stations(&g, true)[0]["location"], "Scenerommet");
    }
    assert_eq!(
        set(&mut g, "unknown", "Room").unwrap_err(),
        "Unknown station"
    );
    let unicode = "ø".repeat(80);
    set(&mut g, "code", &format!("  {unicode}  ")).unwrap();
    assert_eq!(stations(&g, false)[1]["location"], unicode);
    set(&mut g, "code", "   ").unwrap();
    assert_eq!(stations(&g, true)[1]["location"], "");
    g.execute(None, &Command::Reset, 1000, 0, false).unwrap();
    assert_eq!(stations(&g, true)[0]["location"], "Scenerommet");
    g.execute(None, &Command::Demo, 1000, 0, false).unwrap();
    assert_eq!(stations(&g, false)[0]["location"], "Scenerommet");
    for phase in ["playing", "meeting", "ended"] {
        g.phase = phase.into();
        assert_eq!(
            set(&mut g, "wires", "Changed").unwrap_err(),
            "Station setup requires the lobby"
        );
        assert_eq!(stations(&g, true)[0]["location"], "Scenerommet");
    }
    assert_eq!(stations(&Game::default(), true)[0]["location"], "");
}
#[test]
fn kill_rules_hidden_body_and_report() {
    let mut g = demo();
    let target_id = g.players[2].id.clone();
    assert!(
        act(
            &mut g,
            1,
            Command::Kill {
                target_id,
                interaction: Interaction::Qr
            },
            1000
        )
        .is_err()
    );
    kill(&mut g, 1, 1000).unwrap();
    assert!(kill(&mut g, 2, 1001).is_err());
    assert!(kill(&mut g, 1, 27000).is_err());
    let crew = g.players[2].id.clone();
    let view = g.snapshot(Some(&crew), false, 1000, |_| true);
    assert!(view["players"][1].get("alive").is_none());
    assert!(view["players"][0].get("role").is_none());
    assert!(!view["events"].to_string().contains("was killed"));
    assert_eq!(view["tasks"].as_array().unwrap().len(), g.stations.len());
    assert_eq!(view["task_progress"]["total"], 5 * g.stations.len());
    assert!(
        act(
            &mut g,
            1,
            Command::StartTask {
                station_id: "code".into(),
                interaction: Interaction::Qr
            },
            1000
        )
        .is_err()
    );
    let victim = g.players[1].id.clone();
    act(
        &mut g,
        2,
        Command::Report {
            target_id: victim.clone(),
        },
        1001,
    )
    .unwrap();
    assert_eq!(g.phase, "meeting");
    assert!(!g.players[1].body);
    assert!(act(&mut g, 1, Command::Vote { target_id: None }, 1002).is_err());
    g.execute(None, &Command::EndMeeting, 1002, 0, false)
        .unwrap();
    complete(&mut g, 1, "code", 1003);
    assert!(act(&mut g, 2, Command::Report { target_id: victim }, 1004).is_err());
}
#[test]
fn interaction_modes_and_explicit_presence_fact() {
    let mut g = demo();
    g.settings.kill_mode = Mode::Proximity;
    let id = g.players[0].id.clone();
    let target_id = g.players[1].id.clone();
    let c = Command::Kill {
        target_id: target_id.clone(),
        interaction: Interaction::Proximity,
    };
    assert!(g.execute(Some(&id), &c, 1000, 0, false).is_err());
    assert!(
        g.execute(
            Some(&id),
            &Command::Kill {
                target_id,
                interaction: Interaction::Qr
            },
            1000,
            0,
            true
        )
        .is_err()
    );
    g.execute(Some(&id), &c, 1000, 0, true).unwrap();
    g.settings.task_mode = Mode::Proximity;
    let id = g.players[2].id.clone();
    let c = Command::StartTask {
        station_id: "code".into(),
        interaction: Interaction::Proximity,
    };
    assert!(g.execute(Some(&id), &c, 1000, 0, false).is_err());
    g.execute(Some(&id), &c, 1000, 0, true).unwrap();
}
#[test]
fn all_puzzles_validate_and_reject_bad_results() {
    for seed in [0, 1, 2, 7, 97, u64::MAX] {
        for kind in activities::KINDS {
            let challenge = activities::challenge(kind, seed);
            assert!(
                activities::valid(kind, &challenge, &solution(kind, &challenge)),
                "{kind} seed {seed}"
            );
            assert!(!activities::valid(kind, &challenge, &json!({})), "{kind}");
        }
    }
    let c = json!({"masks":[15,15,15,15,15,15,15,15,15],"solution":[1,1,1,1,1,1,1,1,1]});
    assert!(activities::valid(
        "circuits",
        &c,
        &json!({"rotations":[0,0,0,0,0,0,0,0,0]})
    ));
    assert!(!activities::valid(
        "circuits",
        &c,
        &json!({"rotations":[4,0,0,0,0,0,0,0,0]})
    ));
    let c = json!({"masks":[8,0,0,0,0,0,0,0,2]});
    assert!(!activities::valid(
        "circuits",
        &c,
        &json!({"rotations":[0,0,0,0,0,0,0,0,0]})
    ));
}
#[test]
fn task_sessions_wrong_retry_fake_and_cancellation() {
    let mut g = demo();
    complete(&mut g, 0, "code", 1000);
    assert!(g.tasks.iter().all(|t| !t.completed));
    act(
        &mut g,
        1,
        Command::StartTask {
            station_id: "code".into(),
            interaction: Interaction::Qr,
        },
        1000,
    )
    .unwrap();
    let id = g.players[1].id.clone();
    let a = g.attempts[&id].clone();
    assert!(
        act(
            &mut g,
            1,
            Command::SubmitTask {
                session_id: a.id.clone(),
                result: json!({"answer":"wrong"})
            },
            1000
        )
        .is_err()
    );
    assert!(g.attempts.contains_key(&id));
    act(
        &mut g,
        1,
        Command::SubmitTask {
            session_id: a.id.clone(),
            result: solution("code", &a.challenge),
        },
        1000,
    )
    .unwrap();
    assert!(
        act(
            &mut g,
            1,
            Command::SubmitTask {
                session_id: a.id,
                result: json!({})
            },
            1000
        )
        .is_err()
    );
    assert!(
        act(
            &mut g,
            1,
            Command::StartTask {
                station_id: "code".into(),
                interaction: Interaction::Qr
            },
            1000
        )
        .is_err()
    );
    act(
        &mut g,
        1,
        Command::StartTask {
            station_id: "memory".into(),
            interaction: Interaction::Qr,
        },
        1000,
    )
    .unwrap();
    g.execute(None, &Command::Emergency, 1000, 0, false)
        .unwrap();
    assert!(g.attempts.is_empty());
    assert!(
        act(
            &mut g,
            1,
            Command::StartTask {
                station_id: "memory".into(),
                interaction: Interaction::Qr
            },
            1000
        )
        .is_err()
    );
    g.reset(1001);
    assert!(g.attempts.is_empty());
}
#[test]
fn meeting_ties_skip_deadline_cooldown_and_ejection() {
    let mut g = demo();
    g.execute(None, &Command::Emergency, 1000, 0, false)
        .unwrap();
    let a = g.players[0].id.clone();
    let b = g.players[1].id.clone();
    act(
        &mut g,
        0,
        Command::Vote {
            target_id: Some(a.clone()),
        },
        1001,
    )
    .unwrap();
    assert!(act(&mut g, 0, Command::Vote { target_id: None }, 1002).is_err());
    act(&mut g, 1, Command::Vote { target_id: Some(b) }, 1002).unwrap();
    g.tick(91000);
    assert_eq!(g.phase, "playing");
    assert!(g.last_meeting.as_ref().unwrap()["ejected_id"].is_null());
    assert!(
        g.execute(None, &Command::Emergency, 91001, 0, false)
            .is_err()
    );
    g.execute(None, &Command::Emergency, 121000, 0, false)
        .unwrap();
    act(&mut g, 0, Command::Vote { target_id: None }, 121001).unwrap();
    act(
        &mut g,
        1,
        Command::Vote {
            target_id: Some(a.clone()),
        },
        121002,
    )
    .unwrap();
    g.execute(None, &Command::EndMeeting, 121003, 0, false)
        .unwrap();
    assert!(g.last_meeting.as_ref().unwrap()["ejected_id"].is_null());
    g.execute(None, &Command::Emergency, 152000, 0, false)
        .unwrap();
    for i in 0..6 {
        act(
            &mut g,
            i,
            Command::Vote {
                target_id: Some(a.clone()),
            },
            152001,
        )
        .unwrap();
    }
    assert_eq!(g.phase, "ended");
    assert_eq!(g.winner, Some(Role::Crew));
    assert!(
        g.snapshot(None, false, 152001, |_| false)["players"][0]
            .get("role")
            .is_some()
    );
}
#[test]
fn parity_and_task_victories() {
    let mut g = demo();
    for (i, now) in [(1, 1000), (2, 27000), (3, 53000), (4, 79000)] {
        kill(&mut g, i, now).unwrap();
    }
    assert_eq!(g.phase, "ended");
    assert_eq!(g.winner, Some(Role::Impostor));
    assert!(kill(&mut g, 5, 105000).is_err());
    let mut g = demo();
    let stations = g.stations.iter().map(|s| s.id.clone()).collect::<Vec<_>>();
    for i in 1..6 {
        for station in &stations {
            complete(&mut g, i, station, 1000);
        }
    }
    assert_eq!(g.winner, Some(Role::Crew));
    assert_eq!(g.phase, "ended");
}

#[test]
fn rename_cannot_make_rejoining_ambiguous() {
    let mut g = demo();
    let id = g.players[1].id.clone();
    let duplicate = format!(" {} ", g.players[0].name.to_uppercase());
    let before = g.snapshot(None, true, 1000, |_| true);
    assert!(
        g.execute(
            None,
            &Command::RenamePlayer {
                player_id: id,
                name: duplicate
            },
            1000,
            0,
            false
        )
        .is_err()
    );
    assert_eq!(g.snapshot(None, true, 1000, |_| true), before);
}

#[test]
fn automatic_and_configured_groups_have_no_artificial_maximum() {
    let mut g = Game::default();
    let old = serde_json::to_value(&g.settings).unwrap();
    let mut legacy = old.clone();
    legacy.as_object_mut().unwrap().remove("player_count");
    let legacy: protocol::Settings = serde_json::from_value(legacy).unwrap();
    assert_eq!(legacy.player_count, None);
    for i in 0..50 {
        g.join(&format!("Player {i}"), None, 0).unwrap();
    }
    let mut settings = g.settings.clone();
    settings.impostors = 15;
    g.execute(
        None,
        &Command::Configure {
            settings: settings.clone(),
        },
        0,
        0,
        false,
    )
    .unwrap();
    let order = g.players.iter().map(|p| p.id.clone()).collect::<Vec<_>>();
    g.start(&order, 0).unwrap();
    assert_eq!(
        g.players
            .iter()
            .filter(|p| p.role == Some(Role::Impostor))
            .count(),
        15
    );
    assert_eq!(g.tasks.len(), 35 * g.stations.len());
    g.reset(1);
    settings.player_count = Some(49);
    assert!(
        g.execute(None, &Command::Configure { settings }, 1, 0, false)
            .is_err()
    );
    assert_eq!(g.settings.player_count, None);
    // Extremely large impostor inputs reject at start without multiplication overflow.
    g.settings.impostors = usize::MAX;
    assert!(g.start(&order, 1).is_err());
    g.demo(2);
    assert_eq!(g.players.len(), 6);
    assert_eq!(g.settings.impostors, 1);
    assert!(g.settings.player_count.is_none_or(|n| n == 6));

    let mut g = Game::default();
    let mut settings = g.settings.clone();
    for count in [0, 1, 2] {
        settings.player_count = Some(count);
        assert!(
            g.execute(
                None,
                &Command::Configure {
                    settings: settings.clone()
                },
                0,
                0,
                false
            )
            .is_err()
        );
    }
    settings.player_count = Some(20);
    settings.impostors = 7;
    g.execute(
        None,
        &Command::Configure {
            settings: settings.clone(),
        },
        0,
        0,
        false,
    )
    .unwrap();
    for i in 0..19 {
        g.join(&format!("Player {i}"), None, 0).unwrap();
    }
    let mut order = g.players.iter().map(|p| p.id.clone()).collect::<Vec<_>>();
    assert!(g.start(&order, 0).is_err());
    order.push(g.join("Twentieth", None, 0).unwrap());
    assert!(g.join("Extra", None, 0).is_err());
    g.start(&order, 0).unwrap();
    assert_eq!(
        g.players
            .iter()
            .filter(|p| p.role == Some(Role::Impostor))
            .count(),
        7
    );
    assert_eq!(g.tasks.len(), 13 * g.stations.len());
    g.reset(1);
    settings.player_count = Some(21);
    settings.impostors = 10;
    g.execute(
        None,
        &Command::Configure {
            settings: settings.clone(),
        },
        1,
        0,
        false,
    )
    .unwrap();
    order.push(g.join("New seat", None, 1).unwrap());
    g.start(&order, 1).unwrap(); // Eleven crew outnumber ten impostors.
    g.reset(2);
    settings.player_count = None;
    settings.impostors = 11;
    g.execute(None, &Command::Configure { settings }, 2, 0, false)
        .unwrap();
    assert!(g.start(&order, 2).is_err());
}

#[test]
fn repeatable_stations_have_distinct_qr_identity_and_progress() {
    let mut g = Game::default();
    g.execute(
        None,
        &Command::SetStationLocation {
            station_id: "wires".into(),
            location: "First room".into(),
        },
        0,
        0,
        false,
    )
    .unwrap();
    g.execute(
        None,
        &Command::AddStation {
            kind: "wires".into(),
            location: " Second room ".into(),
        },
        0,
        0,
        false,
    )
    .unwrap();
    let extra = g.stations.last().unwrap().clone();
    assert_ne!(extra.id, extra.kind);
    assert_eq!(extra.kind, "wires");
    assert_eq!(extra.location, "Second room");
    let setup = serde_json::to_value(&g.stations).unwrap();
    g.demo(1000);
    assert_eq!(serde_json::to_value(&g.stations).unwrap(), setup);
    assert_eq!(g.tasks.len(), 5 * g.stations.len());
    let crew = g.players[1].id.clone();
    let view = g.snapshot(Some(&crew), false, 1000, |_| true);
    assert_eq!(
        view["available_activities"].as_array().unwrap().len(),
        activities::KINDS.len()
    );
    let tasks: Vec<_> = g
        .tasks
        .iter()
        .filter(|t| t.player_id == crew && t.kind == "wires")
        .collect();
    assert_eq!(tasks.len(), 2);
    assert_ne!(tasks[0].id, tasks[1].id);
    act(
        &mut g,
        0,
        Command::StartTask {
            station_id: extra.id.clone(),
            interaction: Interaction::Qr,
        },
        1000,
    )
    .unwrap();
    let fake = &g.attempts[&g.players[0].id];
    assert_eq!(fake.station_id, extra.id);
    assert_eq!(fake.kind, "wires");
    complete(&mut g, 0, &extra.id, 1000);
    assert!(g.tasks.iter().all(|t| !t.completed));
    act(
        &mut g,
        1,
        Command::StartTask {
            station_id: extra.id.clone(),
            interaction: Interaction::Qr,
        },
        1000,
    )
    .unwrap();
    let active = g.snapshot(Some(&crew), false, 1000, |_| true);
    assert_eq!(active["active_task"]["station_id"], extra.id);
    assert_eq!(active["active_task"]["kind"], "wires");
    complete(&mut g, 1, &extra.id, 1000);
    assert_eq!(g.tasks.iter().filter(|t| t.completed).count(), 1);
    assert!(
        !g.tasks
            .iter()
            .find(|t| t.player_id == crew && t.station_id == "wires")
            .unwrap()
            .completed
    );
    complete(&mut g, 1, "wires", 1000);
    assert_eq!(g.tasks.iter().filter(|t| t.completed).count(), 2);
    for command in [
        Command::AddStation {
            kind: "wires".into(),
            location: "".into(),
        },
        Command::RemoveStation {
            station_id: extra.id.clone(),
        },
        Command::SetStationLocation {
            station_id: extra.id.clone(),
            location: "Moved".into(),
        },
    ] {
        assert!(g.execute(None, &command, 1000, 0, false).is_err());
    }
    g.reset(1001);
    assert_eq!(serde_json::to_value(&g.stations).unwrap(), setup);
    g.execute(
        None,
        &Command::SetStationLocation {
            station_id: extra.id.clone(),
            location: "Moved".into(),
        },
        1001,
        0,
        false,
    )
    .unwrap();
    g.execute(
        None,
        &Command::RemoveStation {
            station_id: extra.id.clone(),
        },
        1001,
        0,
        false,
    )
    .unwrap();
    assert!(
        g.execute(
            None,
            &Command::RemoveStation {
                station_id: extra.id.clone()
            },
            1001,
            0,
            false
        )
        .is_err()
    );
    g.execute(
        None,
        &Command::AddStation {
            kind: "wires".into(),
            location: "Replacement".into(),
        },
        1001,
        0,
        false,
    )
    .unwrap();
    assert_ne!(g.stations.last().unwrap().id, extra.id);
    for (kind, location) in [("unknown", "Room"), ("wires", "bad\nroom")] {
        assert!(
            g.execute(
                None,
                &Command::AddStation {
                    kind: kind.into(),
                    location: location.into()
                },
                1001,
                0,
                false
            )
            .is_err()
        );
    }
    g.demo(1002);
    assert!(
        act(
            &mut g,
            1,
            Command::StartTask {
                station_id: extra.id,
                interaction: Interaction::Qr
            },
            1002
        )
        .is_err()
    );
}

#[test]
fn new_activity_boundaries_and_malformed_results() {
    for kind in activities::KINDS {
        let c = activities::challenge(kind, 97);
        for bad in [
            json!(null),
            json!([]),
            json!(false),
            json!("answer"),
            json!({}),
            json!({"value":"63","held_ms":999999,"hits":[null],"sequence":null,"index":-1,"rotations":[0],"connections":[0],"choices":[],"answer":123}),
        ] {
            assert!(!activities::valid(kind, &c, &bad), "{kind}: {bad}");
        }
    }
    for kind in [
        "mixing",
        "transfer",
        "fault",
        "frequency",
        "breaker",
        "timing",
        "route_power",
        "switches",
        "cargo",
    ] {
        let c = activities::challenge(kind, 0);
        assert_ne!(c, activities::challenge(kind, 1), "{kind} must vary");
        let mut bad = solution(kind, &c);
        match kind {
            "mixing" => {
                bad["a"] = json!(c["target_a"].as_f64().unwrap() + 3.0);
                assert!(activities::valid(kind, &c, &bad));
                bad["a"] = json!(c["target_a"].as_f64().unwrap() + 3.1);
            }
            "transfer" => {
                bad["progress"] = json!(101);
            }
            "fault" => {
                bad["index"] = c["lights"].clone();
            }
            "frequency" => {
                bad["value"] = json!(c["target"].as_f64().unwrap() + 2.0);
                assert!(activities::valid(kind, &c, &bad));
                bad["value"] = json!(c["target"].as_f64().unwrap() + 2.1);
            }
            "breaker" => {
                bad["answer"] = json!("000");
            }
            "timing" => {
                bad["hits"][0] = json!(c["zone_start"].as_u64().unwrap() - 1);
            }
            "route_power" => {
                bad["rotations"][0] = json!(4);
            }
            "switches" => {
                bad["sequence"][0] = bad["sequence"][1].clone();
            }
            "cargo" => {
                bad["choices"][0] = json!(99);
            }
            _ => unreachable!(),
        }
        assert!(!activities::valid(kind, &c, &bad), "{kind}");
    }
    for kind in ["mixing", "transfer", "frequency"] {
        let c = activities::challenge(kind, 1);
        let mut r = solution(kind, &c);
        let required = c[if kind == "transfer" {
            "required_ms"
        } else {
            "hold_ms"
        }]
        .as_u64()
        .unwrap();
        r["held_ms"] = json!(required - 1);
        assert!(!activities::valid(kind, &c, &r));
        r["held_ms"] = json!(required);
        assert!(activities::valid(kind, &c, &r));
        r["held_ms"] = json!("999999");
        assert!(!activities::valid(kind, &c, &r));
    }
    let c = activities::challenge("timing", 1);
    for hits in [
        json!([]),
        json!([50, 50]),
        json!([50, 50, 50, 50]),
        json!([50, "50", 50]),
    ] {
        assert!(!activities::valid("timing", &c, &json!({"hits":hits})));
    }
}

#[test]
fn activity_budget_assigns_distinct_stations_and_counts_only_assigned_work() {
    let mut g = demo();
    g.reset(2000);
    g.settings.activities_per_player = Some(3);
    let order: Vec<String> = g.players.iter().map(|p| p.id.clone()).collect();
    g.start(&order, 3000).unwrap();
    assert_eq!(g.tasks.len(), 15);
    for player in g.players.iter().filter(|p| p.role == Some(Role::Crew)) {
        let ids: std::collections::HashSet<_> = g
            .tasks
            .iter()
            .filter(|t| t.player_id == player.id)
            .map(|t| &t.station_id)
            .collect();
        assert_eq!(ids.len(), 3);
    }
    let absent = g
        .stations
        .iter()
        .find(|s| {
            !g.tasks
                .iter()
                .any(|t| t.player_id == g.players[1].id && t.station_id == s.id)
        })
        .unwrap()
        .id
        .clone();
    assert!(
        act(
            &mut g,
            1,
            Command::StartTask {
                station_id: absent,
                interaction: Interaction::Qr
            },
            4000
        )
        .is_err()
    );
    for task in g.tasks.clone() {
        g.execute(
            Some(&task.player_id),
            &Command::StartTask {
                station_id: task.station_id,
                interaction: Interaction::Qr,
            },
            4000,
            42,
            true,
        )
        .unwrap();
        let snapshot = g.snapshot(Some(&task.player_id), false, 4000, |_| true);
        let session = &snapshot["active_task"];
        let result = solution(&task.kind, &session["challenge"]);
        g.execute(
            Some(&task.player_id),
            &Command::SubmitTask {
                session_id: session["id"].as_str().unwrap().into(),
                result,
            },
            5000,
            42,
            true,
        )
        .unwrap();
    }
    assert_eq!(g.winner, Some(Role::Crew));
    g.reset(6000);
    g.settings.activities_per_player = Some(g.stations.len() + 1);
    assert!(g.start(&order, 7000).is_err());
    g.settings.activities_per_player = Some(0);
    assert!(g.start(&order, 7000).is_err());
    g.settings.activities_per_player = None;
    g.stations.clear();
    assert!(g.start(&order, 7000).is_err());
}
