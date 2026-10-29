# Software release integration contract

This is the shared contract for the two implementation instances. Rust owns game rules; the frontend owns interaction and rendering. All JSON field names use snake_case. IDs are strings. Times are Unix milliseconds. One in-memory game.

## Transport

- `GET /api/state?token=...&view=player|host` returns a `Snapshot`. Host is trusted.
- `POST /api/command` accepts `{ token?: string, command: Command }`, returns `{ ok: bool, error?: string, token?: string, player_id?: string, state: Snapshot }`. Join returns token/player_id. Host commands have no token; player commands require the player's token.
- `GET /api/ws?token=...&view=player|host` sends `{ type: "state", state: Snapshot }` initially, on changes, and periodically for timers. Frontend can fall back to polling if needed.
- `GET /api/health` returns success.
- Rust serves `web/dist` as SPA fallback when present; default bind `0.0.0.0:3001`, configurable `PORT`. Frontend dev server proxies `/api` and websocket to port 3001.
- A shared PC emergency command is trusted with no player token.

## Snapshot

```text
{
  game_id: string,
  room_code: string,
  phase: "lobby"|"playing"|"meeting"|"ended",
  now: number,
  settings: { impostors: number, player_count: number|null, kill_cooldown_secs: number,
    emergency_cooldown_secs: number, meeting_duration_secs: number,
    kill_mode: "qr"|"proximity"|"either", task_mode: "qr"|"proximity"|"either" },
  players: [{ id, name, color, connected: bool, alive?: bool, role?: "crew"|"impostor" }],
  me: null | { id, name, role, alive, kill_ready_at: number, token?: string },
  stations: [{ id, name, kind, description, duration: string, location: string }],
  tasks: [{ id, player_id, station_id, kind, completed: bool }],
  active_task: null | { id: string, task_id: string, kind: string, challenge: object },
  meeting: null | { reason: "emergency"|"body", victim_id?: string,
    ends_at: number, voted_player_ids: string[], votes?: object },
  last_meeting: null | { ejected_id: string|null, skipped: bool, tally: object },
  winner: null | "crew"|"impostor",
  win_reason: string|null,
  task_progress: { completed: number, total: number },
  emergency_ready_at: number,
  presence: [{ first: string, second: string, near: bool, expires_at: number }],
  feedback: [{ id: string, at: number, target: string, kind: string, message: string }],
  events: [{ id: string, at: number, kind: string, message: string }],
  demo: bool
}
```

Host sees full player statuses, tasks, and recent feedback/events. Player view must not reveal unreported deaths, other roles, other private tasks, or kill-event messages. During play other-player `alive`/`role` fields are omitted (public ejections can be represented later); own status is in `me`. During meetings alive/dead status is public so only living players are offered as vote targets; roles remain hidden. All roles are visible after the round. Presence subjects use actual player/station IDs, no prefixes. Only live presence is returned. Public task progress includes crew assignments only.

## Commands (tagged by `type`)

```text
join { name: string, room_code?: string }
rename_player { player_id: string, name: string } // host, lobby or demo only
configure { settings: Settings }
set_station_location { station_id: string, location: string } // trusted host; lobby only
add_station { kind: string, location: string } // lobby; returns a new unique station in state
remove_station { station_id: string } // lobby
start {}
reset {}
demo {}                  // reset + six named demo players, first impostor,
                         // playing phase, all eight tasks per crew, demo=true
kill { target_id: string, interaction: "qr"|"proximity" }
report { target_id: string }
emergency {}
vote { target_id: string|null } // null = skip
end_meeting {}           // trusted host, useful for solo testing
start_task { station_id: string, interaction: "qr"|"proximity" }
submit_task { session_id: string, result: object }
cancel_task {}
presence { first: string, second: string, near: bool, valid_for_ms: number }
test_feedback { target: string, kind: "killed"|"meeting"|"task_complete"|"game_end" }
demo_session { player_id: string } // demo only: returns that player's token
```

Host demo-session switching is intentionally available ONLY while demo=true. Real lobby start assigns roles randomly. Demo creates enough crew that killing one does not end the game. Configure supported in lobby; reset retains roster (clears roles, tasks, feedback, presence), demo replaces roster. Reset invalidates active task sessions. Match is lost on server restart. Minimum start players = 3 and crew must outnumber impostors.

Players choose their name on join. The host can rename a player in the lobby or during a solo demo. Apply the same trimmed 1–24 character/no-controls validation as join, reject another player's name case-insensitively, and retain the player's ID, color, role, tasks, token and progress. Renames appear in all views and QR-card labels; the QR contents still use the same player ID.

Joining with a correct room code and an existing name (trimmed, case-insensitive) reclaims that player's existing token in any game phase. New players can join only in the lobby. A missing/wrong room code cannot reclaim an existing name. This recovery is intentionally trust-based. Browser storage remembers the last token/name/room; session storage keeps independently selected demo players stable in separate tabs. Server restart still loses the round and its identities.

Station locations are optional host-configured strings (trimmed, max 80 characters, no control characters). Keep them across reset/demo because they describe the venue, and include `location` in every snapshot's station catalog. Display names such as `Connect Wires (Scenerommet)` are composed by the frontend from the base name plus location; station IDs and QR contents stay unchanged. Unknown stations and edits outside the lobby are rejected. No persistence across server restarts.

## Configurable groups and repeatable stations

No hard-coded maximum player count or impostor count. `settings.player_count` is nullable (automatic from joined roster) or a host-configured integer >=3. If set, joining stops at that configured capacity and starting requires that many joined players. The host may increase it in the lobby; changing it below the seated count is rejected. `impostors` must be >=1; on start, living crew must outnumber impostors. Crew count is derived from total minus impostors. Old configure payloads without player_count default it to null.

Stations are game-owned records `{id,kind,name,description,duration,location}`. `kind` identifies the minigame; `id` identifies the physical placement and QR code. A station's ID must never be inferred from its kind. Multiple stations of the same kind are allowed. Add/remove/location edits are lobby-only; preserve setup across reset/demo. Starting assigns one task per station per crew player. Fake impostor activity sessions record the actual station ID too. Default setup has one station for each supported kind; existing eight retain their original IDs for compatibility. Extra placements receive unique IDs. `active_task` adds `station_id` so modal location lookup is unambiguous. Server presence and feedback target validation consult the actual station list, not a hard-coded kind list. A deleted station's code/presence cannot start a task. `available_activities` in Snapshot provides the supported kind catalog, separate from configured stations.

## Additional mobile activities

All nine visible additions are implemented as mobile/browser interactions. No hardware is required. Existing eight stay available. All activity result checks run on the server, with browser timing trusted as in the existing games.

| Kind | Name | Challenge | Result |
| --- | --- | --- | --- |
| mixing | Chemical Mixing | `{target_a:35,target_b:65,tolerance:3,hold_ms:1000}` | `{a:35,b:65,held_ms:1000}` |
| transfer | Data Transfer | `{required_ms:5000,tolerance:14}`; browser displays moving target and accumulates time while controlled cursor stays inside | `{progress:100,held_ms:5000}` |
| fault | Find the Fault | `{fault_index:2,lights:4}`; browser animates one light with irregular timing | `{index:2}` |
| frequency | Frequency Hunt | `{target:63,tolerance:2,hold_ms:1000}`; show signal/blink feedback, not the target value | `{value:63,held_ms:1000}` |
| breaker | Code Breaker | `{code:"583"}`; browser gives exact/misplaced digit feedback for guesses | `{answer:"583"}` |
| timing | Timing Challenge | `{zone_start:46,zone_end:54,required_hits:3}` | `{hits:[49,50,51]}` |
| route_power | Route the Power | Same pipe masks/rotations/solution schema as circuits | `{rotations:[...]}` validated by connectivity |
| switches | Switch Sequence | `{order:[2,0,3,1]}`; show ordered numbered switches | `{sequence:[2,0,3,1]}` |
| cargo | Cargo Sorting | `{categories:["Tools","Supplies"],items:[{label:"Wrench",category:0},{label:"Food",category:1},...]}` | `{choices:[0,1,...]}` matches every item category |

Names/description/duration belong in the activity catalog. New activity IDs are not limits on the number of physical stations. Tests should derive task totals from configured stations rather than assuming eight/forty forever.

## Activities

Kinds and station IDs: `wires`, `code`, `signal`, `battery`, `memory`, `anomaly`, `circuits`, `password`. Each station ID equals its kind. Task IDs are unique per player/station. Everyone on crew receives all eight tasks initially; impostors can open fake activities, whose completion never advances crew progress.

Each start generates an activity session. Browser sends the following result; Rust validates it, marks the task complete, and clears the session. Wrong answers keep the attempt open. Cancellation/meeting clears the attempt. Dead crew can do tasks after their body has been cleared at a meeting; before then they should stay lying down.

| Kind | Challenge | Submitted result |
| --- | --- | --- |
| wires | `{colors:["cyan","red","amber","violet"], right_order:[2,0,3,1]}` | `{connections:[2,0,3,1]}` where each left index maps to a right index; right_order lists left color index at each right position, so validate right_order[connections[i]] == i |
| code | `{code:"48271", reveal_ms:2500}` | `{answer:"48271"}` |
| signal | `{target:67,tolerance:3,hold_ms:1500}` | `{value:67,held_ms:1500}` |
| battery | `{zone_start:40,zone_end:60,required_hits:3}` | `{hits:[50,48,54]}` |
| memory | `{sequence:[0,2,1,3,0]}` | `{sequence:[0,2,1,3,0]}` |
| anomaly | `{symbols:["◇","◇","◈","◇",...],odd_index:2}` | `{index:2}` |
| circuits | `{rotations:[1,2,3,0,1,2,3,0,1],solution:[0,1,2,3,0,1,2,3,0],masks:[...nine bitmasks...]}` | `{rotations:[...nine values...]}`; masks represent unrotated pipes, bits N=1 E=2 S=4 W=8. Solution must give a connected path entering west of tile 0 and leaving east of tile 8. Frontend draws actual rotated masks. Rust accepts correct connected paths, not only exact solution. Generate by rotating a known snake path; irrelevant end-branch openings may be tolerated. |
| password | `{clues:["...three simple uniquely solvable clues..."],answer:"357"}` | `{answer:"357"}` |

Seed puzzle variation server-side. Hidden answers being in private activity challenges is acceptable for trusted friends; no anti-cheat needed. Avoid frontend timers being treated as security. Reject stale sessions, duplicate completion, wrong phase, invalid targets, dead voters, and expired proximity.

## Workspace ownership

Backend agent owns root Cargo.toml/Cargo.lock, `crates/**`, and backend tests. Frontend/root agent owns `web/**`, assets, scripts and docs. Do not edit another instance's files. Backend can add `BACKEND-NOTES.md` with integration notes. No hardware code in this release.

The private `me` snapshot includes `waiting_for_meeting: boolean`. It is true while a killed player still has an uncleared body. Task controls remain disabled until a meeting clears that state. It is not included in public roster entries.
