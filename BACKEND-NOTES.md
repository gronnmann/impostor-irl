# Backend integration

Implemented in `protocol`, `game`, and `server`, following `IMPLEMENTATION-CONTRACT.md`. No contract deviations or new dependencies. All state stays in memory; no database or hardware is required.

## Run and transport

`cargo run -p server` binds `0.0.0.0:3001`; `PORT` overrides the port. Rust serves `web/dist` with an `index.html` SPA fallback. The frontend must be built separately.

- `GET /api/health` returns `{ok:true}`.
- `GET /api/state?token=...&view=player|host` returns a snapshot. Default is player view; host view is intentionally trusted and unauthenticated.
- `POST /api/command` accepts `{token?:string,command:Command}` and returns `{ok,error?,token?,player_id?,state}`. Application errors, including malformed command JSON, use HTTP 200 with `ok:false`.
- `GET /api/ws?token=...&view=player|host` sends `{type:"state",state}` initially, after commands, and every second. Server timers advance without connected clients.
- Commands with tokens return that player's private view; anonymous host commands return host view. The shared anonymous emergency button returns public player view. Host authority and name recovery remain trust-based.

## Configurable groups

`Settings` contains `impostors:usize`, `player_count:usize|null`, the existing cooldown/meeting timer fields, and `kill_mode`/`task_mode`. Missing `player_count` deserializes as null for old callers. Default is automatic capacity (`null`) and one impostor.

There is no hard-coded maximum player count or impostor count. A configured player count must be at least three and cannot be below the joined roster. Only a configured capacity blocks new joins. Capacity may be raised in the lobby or cleared to automatic. Start requires exactly the configured count when present, at least three joined players, at least one impostor, and strictly more crew than impostors. The comparison avoids multiplication overflow even for extremely large impostor settings. Crew count is roster count minus impostors. Cooldowns retain 0–600-second validation and meetings retain 5–600 seconds.

Normal start shuffles the full roster before role assignment. `demo` replaces the roster with Nova, Atlas, Echo, Luna, Orion and Vega, resets settings to automatic/one impostor, assigns Nova as impostor, and makes initial kill/emergency cooldowns ready. Previous large-group settings cannot break the six-player preset. `demo_session {player_id}` returns a player's token only during demo mode.

## Stations and final snapshot schema

`stations` is the configured physical placement list. Every record has `{id,kind,name,description,duration,location}`. A station ID is its QR/presence identity; kind selects the activity. Default setup includes all 17 kinds with base IDs matching their kinds, retaining the original eight IDs. Extra placements have opaque unique `station-N` IDs that are not reused after removal/reset/demo. Never derive a station ID from kind.

`available_activities` is a separate supported-kind catalog, currently using the same metadata record shape with base IDs and empty locations. It remains complete even when a default station is removed. Use its `kind` to add a placement; only `stations` supplies usable QR/presence targets.

Lobby-only trusted commands:

```json
{"type":"add_station","kind":"wires","location":"Scenerommet"}
{"type":"remove_station","station_id":"station-1"}
{"type":"set_station_location","station_id":"station-1","location":"Kitchen"}
```

Add returns the new placement in response `state.stations`; no extra response field is needed. Locations are trimmed, limited to 80 Unicode characters, reject controls, and may be empty. Unknown kinds/IDs and setup edits outside the lobby reject. Reset and demo preserve added/deleted placements, IDs and locations; process restart restores defaults. Removed IDs reject task start, presence and feedback. Removal also prunes presence pairs for that ID immediately.

Every crew player gets one assignment per configured station: `{id,player_id,station_id,kind,completed}`. Task IDs are unique per player and station. Duplicate activities at different locations have independent progress. Total progress is crew count times configured station count, and counts only crew assignments. Impostors can open fake sessions at any actual station without advancing progress.

`active_task` is private and nullable: `{id,task_id,station_id,kind,challenge}`. Fake sessions include the actual station ID too. Use `station_id` for modal location lookup and `kind` for activity rendering. Start uses `start_task {station_id,interaction}`; submit uses `submit_task {session_id,result}`. Wrong answers retain the session; stale submissions, repeated completion and invalid phases reject. Cancellation, meetings, reset and ending the round clear attempts.

## Mobile activity validation

All 17 kinds have seeded challenge variation and catalog names/descriptions/durations (10–30 seconds). Existing kinds remain `wires`, `code`, `signal`, `battery`, `memory`, `anomaly`, `circuits`, `password`. New kinds and accepted result requirements:

| Kind | Result validation |
| --- | --- |
| mixing | Numeric `a` and `b` each within challenge tolerance of `target_a`/`target_b`, integer `held_ms` >= `hold_ms`. |
| transfer | Numeric `progress` exactly 100, integer `held_ms` >= `required_ms`. Challenge supplies browser tracking tolerance. |
| fault | Unsigned integer `index` equals `fault_index`. |
| frequency | Numeric `value` within challenge tolerance of target, integer `held_ms` >= `hold_ms`. |
| breaker | String `answer` exactly equals the three-digit challenge code. |
| timing | `hits` has exactly `required_hits` numeric values, each in inclusive `zone_start`/`zone_end`. |
| route_power | Nine integer rotations in 0–3; reciprocal pipe connectivity from west of tile 0 to east of tile 8. |
| switches | Integer `sequence` exactly equals challenge `order`. |
| cargo | Integer `choices` matches every item's category, in order, with exact item count. |

Challenge/result field names follow the contract table. Mix targets total 100, transfer duration varies 5000–7000ms, fault has four lights, breaker codes are three digits, timing has a narrow zone, switches has four distinct indexes, and cargo has six labeled items and two categories. Browser-reported timing remains trusted; no server wall-clock gate or firmware was added. Non-object/missing/wrong-type/incorrect-length results reject.

Wire results invert `right_order`. Circuits and route_power share the connectivity implementation, accepting alternate connected solutions rather than requiring the supplied solution rotations. Masks use N=1/E=2/S=4/W=8 and clockwise rotations. Signal and battery also validate using their challenge parameters.

## Identity, privacy and round behavior

Join issues a random reconnect token. A valid token reconnects its own identity regardless of submitted name/code. Tokens survive reset; demo replaces them, and process restart loses all identities. Polling/WS refreshes connected status, which expires after 30 seconds without activity; disconnected tokens remain usable.

Without a valid token, an existing trimmed name can reclaim the old player only with an explicitly supplied correct room code, in any phase. Name matching uses Unicode lowercase; room codes use ASCII case-insensitive matching and are not trimmed. A missing code rejects duplicate names; a wrong code rejects. Recovery preserves ID, token, stored name, roles, tasks, attempts and progress, including at full configured capacity. New names remain lobby-only. Names require 1–24 Unicode characters after trimming and no controls.

`rename_player {player_id,name}` shares name validation, rejects case-insensitive duplicates and unknown IDs, and is allowed in lobby or demo. It changes only the stored name; QR IDs, colors, roles, tasks, attempts and reconnect tokens remain stable.

Lobby roles are null. Player views hide others' roles until the round ends and hide others' deaths outside meetings/ended rounds. Host sees roles/status/tasks. Private tasks, attempts and feedback are scoped to the acting player. Kill events never appear in player event feeds, including the victim's; victims learn through `me` and feedback.

Reporting uses a roster player ID; meetings clear bodies and active sessions. Dead crew with a cleared body can continue tasks. Living players vote once; meetings resolve on all living votes, deadline, or `end_meeting`. Missing votes abstain; ties and skip leaders eject nobody. Vote tally skip key is `skip`. Crew wins by completing all assigned tasks or eliminating impostors; impostors win at living parity.

Presence uses actual player/station IDs as unordered pairs, expires at the exact deadline, and accepts `valid_for_ms` from 1–60000. Proximity actions require a current near pair. Player snapshots expose only pairs involving that player. Feedback is a bounded recent simulator feed; player events route privately, meetings also target `horn`, and test feedback accepts actual player/station IDs or `horn`.

The engine has no runtime, network, wall-clock or RNG dependency. Tests supply explicit time, seed, role order and proximity facts. The server handles random role shuffling, tokens, serialized commands, presence and feedback.

## Verification of this extension

`cargo test --workspace` passes: 13 engine integration tests and 11 server/unit tests. `cargo fmt --all -- --check` passes after `cargo fmt --all`.

Coverage retains previous name recovery, rename, station-location, privacy, body/ghost, cooldown, voting, reset, fake activity, stale session and victory checks. New cases cover 50-player automatic capacity with 15 impostors, configured 20 players with seven impostors, incomplete/full/increased capacity, rejoin at capacity, invalid capacity and overflow-sized impostor inputs, six-player demo after large settings, duplicate-wire QR identity and independent progress, active/fake station IDs, setup preservation, removal/pruned presence/feedback rejection, all 17 puzzle solutions across multiple seeds including u64::MAX, all nine new activities' incorrect answers and structural invalids, tolerance/hold boundaries, and challenge variation. Totals derive from configured station counts.

No release build, service start/stop, frontend edit, live port-3001 request, or browser/device test was performed for this extension. The root agent must rebuild/restart the live release when ready and verify the concurrent mobile UI. All state remains volatile and host/rejoin authority remains intentionally trusted.
