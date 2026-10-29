use super::*;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

fn call(b: &mut Backend, token: Option<&str>, command: Command, now: u64) -> CommandResponse {
    b.command(
        CommandRequest {
            token: token.map(str::to_owned),
            command,
        },
        now,
    )
}

#[test]
fn name_rejoin_requires_room_code_and_reuses_existing_identity() {
    let mut b = Backend::new();
    b.game.room_code = "SHIP".into();
    let joined = call(
        &mut b,
        None,
        Command::Join {
            name: " Ålex ".into(),
            room_code: None,
        },
        1000,
    );
    assert!(joined.ok);
    assert_eq!(joined.state["me"]["name"], "Ålex");
    let token = joined.token.unwrap();
    let id = joined.player_id.unwrap();
    for supplied_token in [None, Some("invalid-token")] {
        for (room_code, error) in [
            (None, "Name already in use; supply the room code to rejoin"),
            (Some("WRONG"), "Room code does not match"),
            (Some(""), "Room code does not match"),
        ] {
            let response = call(
                &mut b,
                supplied_token,
                Command::Join {
                    name: " åLEX ".into(),
                    room_code: room_code.map(str::to_owned),
                },
                1000,
            );
            assert!(!response.ok);
            assert_eq!(response.error.as_deref(), Some(error));
            assert!(response.token.is_none());
            assert!(response.player_id.is_none());
            assert!(response.state["me"].is_null());
            assert_eq!(b.game.players.len(), 1);
        }
        let response = call(
            &mut b,
            supplied_token,
            Command::Join {
                name: " åLEX ".into(),
                room_code: Some("ship".into()),
            },
            1000,
        );
        assert!(response.ok);
        assert_eq!(response.token.as_deref(), Some(token.as_str()));
        assert_eq!(response.player_id.as_deref(), Some(id.as_str()));
        assert_eq!(response.state["me"]["name"], "Ålex");
        assert_eq!(b.game.players.len(), 1);
    }
    // A valid token always reconnects its own identity, ignoring claim inputs.
    let response = call(
        &mut b,
        Some(&token),
        Command::Join {
            name: "".into(),
            room_code: Some("WRONG".into()),
        },
        1000,
    );
    assert!(response.ok);
    assert_eq!(response.player_id.as_deref(), Some(id.as_str()));
    assert_eq!(response.token.as_deref(), Some(token.as_str()));
    assert_eq!(b.game.players.len(), 1);
    assert!(!call(&mut b, Some("invalid-token"), Command::CancelTask, 1000).ok);
    assert_eq!(
        b.snapshot(None, true, 1_000_000)["players"][0]["connected"],
        false
    );
    let response = call(
        &mut b,
        None,
        Command::Join {
            name: "ålex".into(),
            room_code: Some("SHIP".into()),
        },
        1_000_000,
    );
    assert!(response.ok);
    assert_eq!(response.token.as_deref(), Some(token.as_str()));
    assert_eq!(b.snapshot(Some(&token), false, 1_000_001)["me"]["id"], id);
    assert_eq!(b.game.players.len(), 1);
}

#[test]
fn name_rejoin_validates_names_and_keeps_normal_lobby_join() {
    let mut b = Backend::new();
    let room_code = b.game.room_code.clone();
    for name in [" ".into(), "é".repeat(25), "Al\0ex".into()] {
        let response = call(
            &mut b,
            None,
            Command::Join {
                name,
                room_code: Some(room_code.clone()),
            },
            1000,
        );
        assert!(!response.ok);
        assert_eq!(
            response.error.as_deref(),
            Some("Use a name of 1–24 characters")
        );
        assert!(b.game.players.is_empty());
    }
    for (name, code) in [
        ("é".repeat(24), None),
        ("New player".into(), Some(room_code)),
    ] {
        let response = call(
            &mut b,
            None,
            Command::Join {
                name,
                room_code: code,
            },
            1000,
        );
        assert!(response.ok);
        assert!(response.token.is_some());
    }
    assert_eq!(b.game.players.len(), 2);
}

#[test]
fn name_rejoin_preserves_round_state_in_playing_meeting_and_ended() {
    let mut b = Backend::new();
    call(&mut b, None, Command::Demo, 1000);
    b.game.tasks[0].completed = true;
    let killer = b.game.players[0].id.clone();
    let crew = b.game.players[1].id.clone();
    let crew_token = b.sessions.token_for(&crew).unwrap();
    let station_id = b
        .game
        .tasks
        .iter()
        .find(|t| t.player_id == crew && !t.completed)
        .unwrap()
        .station_id
        .clone();
    assert!(
        call(
            &mut b,
            Some(&crew_token),
            Command::StartTask {
                station_id,
                interaction: protocol::Interaction::Qr
            },
            1000,
        )
        .ok
    );
    assert!(b.game.attempts.contains_key(&crew));
    for phase in ["playing", "meeting", "ended"] {
        assert_eq!(b.game.phase, phase);
        for id in [&killer, &crew] {
            let token = b.sessions.token_for(id).unwrap();
            let expected = b.snapshot(Some(&token), false, 1000);
            let players = format!("{:?}", b.game.players);
            let tasks = serde_json::to_value(&b.game.tasks).unwrap();
            let attempts = serde_json::to_value(&b.game.attempts).unwrap();
            let name = b.game.player(id).unwrap().name.to_uppercase();
            let room_code = b.game.room_code.clone();
            let response = call(
                &mut b,
                None,
                Command::Join {
                    name,
                    room_code: Some(room_code),
                },
                1000,
            );
            assert!(response.ok, "{phase}: {:?}", response.error);
            assert_eq!(response.token.as_deref(), Some(token.as_str()));
            assert_eq!(response.player_id.as_deref(), Some(id.as_str()));
            assert_eq!(response.state, expected);
            assert_eq!(format!("{:?}", b.game.players), players);
            assert_eq!(serde_json::to_value(&b.game.tasks).unwrap(), tasks);
            assert_eq!(serde_json::to_value(&b.game.attempts).unwrap(), attempts);
            assert_eq!(b.game.players.len(), 6);
        }
        let room_code = b.game.room_code.clone();
        let response = call(
            &mut b,
            None,
            Command::Join {
                name: "New player".into(),
                room_code: Some(room_code),
            },
            1000,
        );
        assert!(!response.ok);
        assert_eq!(
            response.error.as_deref(),
            Some("The round has already started")
        );
        if phase == "playing" {
            assert!(call(&mut b, Some(&crew_token), Command::Emergency, 1000).ok);
        } else if phase == "meeting" {
            let tokens = b
                .game
                .players
                .iter()
                .map(|p| b.sessions.token_for(&p.id).unwrap())
                .collect::<Vec<_>>();
            for token in tokens {
                assert!(
                    call(
                        &mut b,
                        Some(&token),
                        Command::Vote {
                            target_id: Some(killer.clone())
                        },
                        1000
                    )
                    .ok
                );
            }
        }
    }
}

#[test]
fn rename_command_roundtrip_preserves_demo_token_and_private_view() {
    let mut b = Backend::new();
    call(&mut b, None, Command::Demo, 1000);
    let id = b.game.players[1].id.clone();
    let token = b.sessions.token_for(&id).unwrap();
    let command = serde_json::json!({"type":"rename_player", "player_id":id, "name":" New Name "});
    let request: CommandRequest =
        serde_json::from_value(serde_json::json!({"command":command})).unwrap();
    assert_eq!(serde_json::to_value(&request.command).unwrap(), command);
    let response = b.command(request, 1000);
    assert!(response.ok);
    assert_eq!(response.state["players"][1]["name"], "New Name");
    assert_eq!(response.state["players"][0]["role"], "impostor");
    assert_eq!(b.sessions.token_for(&id).as_deref(), Some(token.as_str()));
    let view = b.snapshot(Some(&token), false, 1000);
    assert_eq!(view["me"]["id"], id);
    assert_eq!(view["me"]["name"], "New Name");
    assert_eq!(view["players"][1]["name"], "New Name");
    assert!(view["players"][0].get("role").is_none());
    assert!(view["players"][0].get("alive").is_none());
    let response = call(
        &mut b,
        None,
        Command::RenamePlayer {
            player_id: "unknown".into(),
            name: "Name".into(),
        },
        1000,
    );
    assert!(!response.ok);
    assert_eq!(response.state["players"][0]["role"], "impostor");
}
#[test]
fn station_location_command_json_and_host_response() {
    let mut b = Backend::new();
    call(
        &mut b,
        None,
        Command::Join {
            name: "Alex".into(),
            room_code: None,
        },
        1000,
    );
    let request: CommandRequest = serde_json::from_value(serde_json::json!({
        "command": {"type":"set_station_location", "station_id":"wires", "location":" Scenerommet "}
    }))
    .unwrap();
    assert_eq!(
        serde_json::to_value(&request.command).unwrap()["type"],
        "set_station_location"
    );
    let response = b.command(request, 1000);
    assert!(response.ok);
    assert_eq!(response.state["stations"][0]["location"], "Scenerommet");
    assert!(response.state["players"][0].get("role").is_some());
    assert_eq!(
        b.snapshot(None, false, 1000)["stations"][0]["location"],
        "Scenerommet"
    );
    call(&mut b, None, Command::Demo, 1000);
    let response = call(
        &mut b,
        None,
        Command::SetStationLocation {
            station_id: "wires".into(),
            location: "Changed".into(),
        },
        1000,
    );
    assert!(!response.ok);
    assert_eq!(response.state["stations"][0]["location"], "Scenerommet");
    assert_eq!(response.state["players"][0]["role"], "impostor");
}

#[test]
fn reconnect_demo_switch_and_reset() {
    let mut b = Backend::new();
    let response = call(
        &mut b,
        None,
        Command::Join {
            name: "Alex".into(),
            room_code: None,
        },
        1000,
    );
    assert!(response.ok);
    let token = response.token.unwrap();
    let id = response.player_id.unwrap();
    assert_eq!(response.state["me"]["id"], id);
    assert_eq!(b.snapshot(Some(&token), false, 2000)["me"]["id"], id);
    assert!(
        b.snapshot(Some(&token), false, 32001)["players"][0]["connected"]
            .as_bool()
            .unwrap()
    );
    assert!(
        !b.snapshot(None, true, 63002)["players"][0]["connected"]
            .as_bool()
            .unwrap()
    );
    let response = call(
        &mut b,
        Some(&token),
        Command::Join {
            name: "ignored".into(),
            room_code: None,
        },
        64000,
    );
    assert_eq!(response.player_id, Some(id.clone()));
    assert_eq!(b.game.players.len(), 1);
    assert!(
        !call(
            &mut b,
            None,
            Command::DemoSession {
                player_id: id.clone()
            },
            64000
        )
        .ok
    );
    call(&mut b, None, Command::Reset, 64000);
    assert_eq!(b.snapshot(Some(&token), false, 64000)["me"]["id"], id);
    call(&mut b, None, Command::Demo, 65000);
    assert!(!call(&mut b, Some(&token), Command::CancelTask, 65000).ok);
    let demo_id = b.game.players[0].id.clone();
    let response = call(
        &mut b,
        None,
        Command::DemoSession {
            player_id: demo_id.clone(),
        },
        65000,
    );
    assert!(response.ok);
    assert_eq!(response.state["me"]["id"], demo_id);
    assert_eq!(response.state["me"]["role"], "impostor");
}
#[test]
fn feedback_private_state_and_proximity_expiry() {
    let mut b = Backend::new();
    call(&mut b, None, Command::Demo, 1000);
    let killer = b.game.players[0].id.clone();
    let victim = b.game.players[1].id.clone();
    let witness = b.game.players[2].id.clone();
    let killer_token = b.sessions.token_for(&killer).unwrap();
    let victim_token = b.sessions.token_for(&victim).unwrap();
    let witness_token = b.sessions.token_for(&witness).unwrap();
    call(
        &mut b,
        None,
        Command::Presence {
            first: killer.clone(),
            second: victim.clone(),
            near: true,
            valid_for_ms: 100,
        },
        1000,
    );
    let command = Command::Kill {
        target_id: victim.clone(),
        interaction: protocol::Interaction::Proximity,
    };
    assert!(!call(&mut b, Some(&killer_token), command.clone(), 1100).ok);
    call(
        &mut b,
        None,
        Command::Presence {
            first: victim.clone(),
            second: killer.clone(),
            near: true,
            valid_for_ms: 100,
        },
        1100,
    );
    assert!(call(&mut b, Some(&killer_token), command, 1101).ok);
    let view = b.snapshot(Some(&witness_token), false, 1101);
    assert!(view["feedback"].as_array().unwrap().is_empty());
    assert!(!view.to_string().contains("was killed"));
    assert!(view["players"][1].get("alive").is_none());
    assert_eq!(
        b.snapshot(Some(&victim_token), false, 1101)["feedback"][0]["kind"],
        "killed"
    );
    assert_eq!(
        b.snapshot(None, true, 1101)["feedback"][0]["target"],
        victim
    );
    assert!(
        call(
            &mut b,
            Some(&witness_token),
            Command::Report { target_id: victim },
            1102
        )
        .ok
    );
    let host = b.snapshot(None, true, 1102);
    assert!(
        host["feedback"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["target"] == "horn")
    );
    assert_eq!(host["meeting"]["reason"], "body");
    let rejected = call(&mut b, None, Command::Vote { target_id: None }, 1103);
    assert!(!rejected.ok);
    assert!(rejected.state["players"][0].get("role").is_none());
    // Deaths are public once a meeting clears the bodies, but roles stay private.
    assert_eq!(rejected.state["players"][1]["alive"], false);
    assert!(rejected.state["tasks"].as_array().unwrap().is_empty());
    call(&mut b, None, Command::Reset, 1200);
    assert!(b.presence.0.is_empty());
    assert!(b.feedback.recent.is_empty());
}
#[tokio::test]
async fn http_contract_and_bad_json() {
    let (router, _) = app();
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let request = Request::builder()
        .method("POST")
        .uri("/api/command")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"command":{"type":"join","name":"Alice"}}"#))
        .unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let joined: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(joined["ok"], true);
    assert!(joined["token"].is_string());
    assert_eq!(joined["state"]["me"]["name"], "Alice");
    let request = Request::builder()
        .method("POST")
        .uri("/api/command")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({"command": {
                "type":"rename_player", "player_id":joined["player_id"], "name":" Alicia "
            }})
            .to_string(),
        ))
        .unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let renamed: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(renamed["ok"], true);
    assert_eq!(renamed["state"]["players"][0]["name"], "Alicia");
    assert!(renamed["state"]["players"][0].get("role").is_some());
    let request = Request::builder()
        .uri(format!(
            "/api/state?view=player&token={}",
            joined["token"].as_str().unwrap()
        ))
        .body(Body::empty())
        .unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let state: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(state["me"]["name"], "Alicia");
    assert_eq!(state["phase"], "lobby");
    let request = Request::builder()
        .method("POST")
        .uri("/api/command")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"command":{"type":"kill","target_id":"x","interaction":"invalid"}}"#,
        ))
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let error: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(error["ok"], false);
    assert!(error["error"].is_string());
    assert!(error["state"].is_object());
}

#[test]
fn dynamic_station_commands_validate_actual_ids_and_prune_presence() {
    let mut b = Backend::new();
    let first = call(
        &mut b,
        None,
        Command::Join {
            name: "Crew".into(),
            room_code: None,
        },
        0,
    );
    let player = first.player_id.unwrap();
    let add: Command = serde_json::from_value(
        json!({"type":"add_station","kind":"wires","location":" Second room "}),
    )
    .unwrap();
    let response = call(&mut b, None, add, 0);
    assert!(response.ok);
    let station = response.state["stations"]
        .as_array()
        .unwrap()
        .last()
        .unwrap();
    let id = station["id"].as_str().unwrap().to_owned();
    assert_eq!(station["kind"], "wires");
    assert_eq!(station["location"], "Second room");
    assert!(response.state["players"][0].get("role").is_some());
    let presence = |station_id: &str| Command::Presence {
        first: player.clone(),
        second: station_id.into(),
        near: true,
        valid_for_ms: 5000,
    };
    let feedback = |station_id: &str| Command::TestFeedback {
        target: station_id.into(),
        kind: protocol::FeedbackKind::TaskComplete,
    };
    assert!(call(&mut b, None, presence(&id), 0).ok);
    assert!(call(&mut b, None, feedback(&id), 0).ok);
    assert_eq!(b.presence.0.len(), 1);
    let remove: Command =
        serde_json::from_value(json!({"type":"remove_station","station_id":id})).unwrap();
    let response = call(&mut b, None, remove, 0);
    assert!(response.ok);
    assert!(b.presence.0.is_empty());
    assert!(!call(&mut b, None, presence(&id), 0).ok);
    assert!(!call(&mut b, None, feedback(&id), 0).ok);
    // Remove the default placement too: a catalog kind alone is no valid target.
    assert!(
        call(
            &mut b,
            None,
            Command::RemoveStation {
                station_id: "wires".into()
            },
            0
        )
        .ok
    );
    assert!(!call(&mut b, None, presence("wires"), 0).ok);
    assert!(!call(&mut b, None, feedback("wires"), 0).ok);
    let setup = b.snapshot(None, true, 0)["stations"].clone();
    assert!(call(&mut b, None, Command::Demo, 1).ok);
    assert_eq!(b.snapshot(None, true, 1)["stations"], setup);
    let token = b.sessions.token_for(&b.game.players[1].id).unwrap();
    for station_id in [&id, "wires"] {
        assert!(
            !call(
                &mut b,
                Some(&token),
                Command::StartTask {
                    station_id: station_id.into(),
                    interaction: protocol::Interaction::Qr
                },
                1
            )
            .ok
        );
    }
    assert!(call(&mut b, None, Command::Reset, 2).ok);
    assert_eq!(b.snapshot(None, true, 2)["stations"], setup);
}

#[test]
fn configured_capacity_rejoin_and_random_start_role_counts() {
    let mut b = Backend::new();
    let mut settings = b.game.settings.clone();
    settings.player_count = Some(20);
    settings.impostors = 7;
    assert!(
        call(
            &mut b,
            None,
            Command::Configure {
                settings: settings.clone()
            },
            0
        )
        .ok
    );
    let room = b.game.room_code.clone();
    let mut first_token = None;
    for i in 0..20 {
        let response = call(
            &mut b,
            None,
            Command::Join {
                name: format!("Person {i}"),
                room_code: None,
            },
            0,
        );
        assert!(response.ok);
        if i == 0 {
            first_token = response.token;
        }
        if i == 18 {
            assert!(!call(&mut b, None, Command::Start, 0).ok);
        }
    }
    assert!(
        !call(
            &mut b,
            None,
            Command::Join {
                name: "Extra".into(),
                room_code: None
            },
            0
        )
        .ok
    );
    let reclaimed = call(
        &mut b,
        None,
        Command::Join {
            name: "person 0".into(),
            room_code: Some(room),
        },
        0,
    );
    assert!(reclaimed.ok);
    assert_eq!(reclaimed.token, first_token);
    assert_eq!(b.game.players.len(), 20);
    assert!(call(&mut b, None, Command::Start, 0).ok);
    assert_eq!(
        b.game
            .players
            .iter()
            .filter(|p| p.role == Some(game::Role::Impostor))
            .count(),
        7
    );
    assert_eq!(b.game.tasks.len(), 13 * b.game.stations.len());
    assert!(call(&mut b, None, Command::Demo, 1).ok);
    assert_eq!(b.game.players.len(), 6);
    assert_eq!(b.game.settings.impostors, 1);
}
