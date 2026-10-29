# impostor-url: architecture and first build

An in-person social deduction game played through phones, initially using printed QR codes and simulated beacons. Real player badges and activity boards come later.

Players are trusted. The server checks ordinary game rules; we do not need anti-cheat infrastructure.

## Scope and decisions

- One running game per server.
- One Rust server with all live state in memory.
- One web frontend with player, host, emergency-button, and hardware-test views.
- No database. Restarting the server resets the game.
- QR interactions make the first version playable without hardware.
- Simulated presence and later BLE/UWB feed the same small presence interface.
- Device feedback starts with virtual vibration/light indicators.
- A small Cargo workspace separates game logic, shared messages, and the server. Firmware crates join it when hardware work starts.
- No microservices, message broker, event sourcing, or dynamic plugin loading.
- Use [the visual design direction](DESIGN.md), based on the supplied Impostor IRL reference.

## How it fits together

```text
Player phones                 Host / simulator browser
  - role and tasks              - configure/start/reset
  - scan player/station QR      - simulate nearby players/stations
  - kill/report/vote            - inspect virtual device feedback
           |                              |
           +---------- web connection ----+
                              |
                    Rust server process
                    -------------------
                    Current game state
                    Game rules
                    Presence table
                    Connected clients/devices
                              |
                     device connection
                              |
                    Future ESP32 devices
                    - report presence / button inputs
                    - receive vibration / light commands
```

The frontend sends commands over a WebSocket. The server changes state and sends an updated view to each connected client. On connection or reconnect, send a full current view. For a small group, full views are simpler than patching state or replaying events.

Phones receive their own role and tasks plus the public game information. The host/simulator is a trusted administration view.

The server also serves the frontend. The phones need a reachable HTTPS address for PWA installation and in-app camera scanning. Deployment and certificates are setup work, not additional game services.

## What things exist?

| Thing | Information it holds |
| --- | --- |
| Game | Phase, settings, players, stations, task assignments, current meeting, winner |
| Player | ID, name, role, alive/dead status, kill cooldown, reconnect token, optional paired badge |
| Station | ID, name, activity kind, optional venue location, optional paired hardware device |
| Task assignment | Player, station, activity progress, completed status |
| Meeting | Caller, optional reported victim, votes, voting deadline |
| Presence entry | A pair of players/stations, near/far status, expiry time |
| Device connection | Device ID, paired player/station, connection status |

A body is a killed player's unreported death, rather than a separate object. Ejected players do not leave reportable bodies. A device is optional: a player or station can exist with only a QR code.

Host configuration can give each activity a venue location. Players and printed station cards see labels such as `Connect Wires (Scenerommet)`. Location changes do not change the station ID or QR contents. Retain locations when resetting a round, but not across server restarts.

The victim stays lying down where they were killed. There is no body card, spawned marker, or host-managed body. The server only needs to know which deaths are still reportable.

We do not need coordinates, rooms, or a floor plan for the rules. A later map-style simulator may maintain coordinates locally and translate them into presence updates.

## Commands: things someone asks to do

Commands are requests. The server either applies one or returns a short reason it cannot happen.

| Command | Source | Meaning |
| --- | --- | --- |
| JoinGame / Reconnect | Phone | Join the lobby or recover the same player session |
| ConfigureGame | Host | Set impostor count, interaction modes, and timers |
| StartGame | Host | Assign roles/tasks and start play |
| AttemptKill | Phone | Try to kill a selected player |
| ReportBody | Phone | Identify the player whose body was found |
| CallMeeting | Emergency-button screen or ESP32 | Request an emergency meeting from the shared physical meeting point |
| StartTask | Phone | Start an assigned activity at a station |
| SubmitTaskInput | Phone or station | Submit an answer, button press, or other activity input |
| CastVote | Phone | Vote for a player or skip |
| ResetGame | Host | Clear the round and return to the lobby |

The server derives who is acting from the connection's player session. A rejected command does not need a global event.

QR scans resolve to a player or station ID. Scanning presents the relevant kill or task action. The requested action includes whether it came from a QR scan or a nearby-target interaction. We trust that declaration. Body reports do not require a QR scan: the finder selects the victim from the full player roster and confirms the report. Do not reveal a list of unreported deaths.

Host controls also pair a device with a player/station and set simulated presence. These are administration inputs, not gameplay actions.

## Shared emergency button

Open `/emergency` on a PC or tablet at the meeting point. It displays a large emergency button and the current game phase. Pressing it sends `CallMeeting`; this is a shared station, so it does not require signing in as a player. We trust players to use it only when eligible.

Later, a paired ESP32 with a physical button sends the same command. No game-rule changes are needed. The server checks that play is active and that the shared emergency cooldown has expired. Body reports bypass this emergency-button cooldown.

Use one shared cooldown rather than tracking an allowance per person at an anonymous button. Its duration is a round setting. No personal emergency button is needed on player phones.

When a meeting starts, badges receive meeting feedback and the configured horn output is triggered. The PC screen can show the meeting state; the horn can later be attached to an ESP32 output.

## Events: things that have happened

Keep events as plain Rust values handled inside the server. They are useful for notifications and device effects. They are not an event log that reconstructs the game, and we do not store them in a database.

| Event | What uses it |
| --- | --- |
| PlayerJoined | Update lobby views |
| GameStarted | Show private roles and task lists |
| TaskStarted | Open the activity screen; optionally update a station indicator |
| TaskCompleted | Update task progress and show completion feedback |
| PlayerKilled | Show the victim's death screen; vibrate their paired badge |
| MeetingStarted | Open meeting screens; vibrate badges; trigger the horn output |
| VoteCast | Update how many players have voted; keep choices private until resolution |
| MeetingEnded | Reveal the result and any ejection; return to play |
| GameEnded | Show winner and roles; send end-of-round feedback |
| GameReset | Clear round screens and return to the lobby |

Presence updates are telemetry, not game events. Device connection changes update device status. Add more events only when some actual behavior needs them.

Process each command to completion before the next one: check rules, update state, resolve any resulting win, then send views and feedback. This gives simultaneous requests a straightforward order.

## Presence and QR interactions

The rules need one query:

```text
is_near(player, other_player_or_station) -> yes / no / unknown
```

The input is deliberately small:

```text
PresenceUpdate {
    first: Player(id) or Station(id),
    second: Player(id) or Station(id),
    near: true or false,
    valid_for_ms: duration
}
```

The server treats the pair as unordered and starts the expiry timer when the update arrives. Expired or missing data is unknown. Only `yes` permits an interaction that requires proximity. The adapter decides how to combine physical observations into this simple result.

| Adapter | How it supplies presence |
| --- | --- |
| Simulator | Host sets a pair near/far; simulator refreshes the selection while connected |
| BLE, later | Convert radio observations into near/far, with enough smoothing to avoid flickering |
| UWB, later | Convert measured distance into near/far using a configured threshold |

Choose one presence source per pair in the initial implementation. Switching adapters does not change kill or task rules. Raw radio readings and ranging configuration stay with the adapter.

Round settings select independently:

```text
kill_mode: QR | proximity | either
task_mode: QR | proximity | either
```

In QR mode, scanning the appropriate code satisfies the interaction requirement. In proximity mode, current presence satisfies it. Both paths then use the same game rules.

For the first version, task proximity is checked when starting the task. Requiring continuous presence can be added for specific activities later.

### Swapping interaction modes

Keep three responsibilities separate:

1. **Identify a target:** QR scanning, a nearby-target list, or a future NFC reader identifies a player or station.
2. **Check the interaction requirement:** The round configuration accepts a QR interaction, current proximity, or either. This check lives in the game rules and uses normalized facts, not camera or radio APIs.
3. **Perform the action:** The same kill or task rules run after the interaction check.

```text
QR scanner ------------> target ID + QR interaction -----+
                                                        |
Simulator / BLE / UWB -> current presence -> target ID ---+--> AttemptKill / StartTask
                                                               |
                                                    configured interaction rule
                                                               |
                                                     ordinary game rules
```

For example, both paths create `AttemptKill(target, interaction)`. For QR, the trusted scan identifies the target. For proximity, the rules consult the current presence supplied by the server. A proximity request must still have current presence; choosing a target alone does not satisfy it.

The frontend has a shared target-selection component with QR and nearby-list implementations. The activity screen and kill confirmation do not care which selector supplied the target. Configuration chooses the enabled implementations, so changing QR to proximity does not require rewriting those screens.

BLE and UWB are presence providers, rather than separate game modes. Changing BLE to UWB changes how presence is produced. Changing QR to proximity changes which interaction requirement the round uses.

The PC emergency page and physical emergency button similarly translate their input into the same `CallMeeting` command. Physical activity inputs translate into the same named activity inputs used by the browser. Badge vibration and virtual feedback are alternative outputs for the same notification.

These are explicit interfaces and built-in adapters compiled with the app. Plug and play here means connecting/configuring a supported adapter, not loading arbitrary third-party code at runtime. A new hardware technology still needs a driver and adapter, but not rewritten game rules.

## Devices and feedback

Game events become simple device commands:

```text
Notify(kind: killed | meeting | task_complete | game_end)
SetIndicator(state: idle | active | complete)
SoundHorn()
```

The device implements the actual vibration pattern or light behavior. The game does not know motor timings, GPIO pin numbers, or radio details.

Initially these commands light up virtual badges/stations and add a line to a short simulator feedback feed. The host can trigger test feedback without starting a round. Later, a device connection delivers the same commands to hardware.

Feedback goes to currently connected devices and is not replayed on reconnect. A reconnect gets current game status so the device can restore persistent indicators. A disconnected badge does not prevent a kill; the phone still receives the result.

Physical stations can later send named inputs such as `ButtonPressed("confirm")`. Station firmware maps GPIO pins to those names. Use built-in activity handlers; no plugin system is needed.

## What "simulator" means

The simulator is a hardware test screen, available at `/simulator`, for developing without ESP32s. It supplies fake hardware inputs to the real server and displays the outputs the real hardware would receive.

For example:

```text
Fake presence
  Alice <-> Bob           [Near] [Far]
  Alice <-> Wiring table  [Near] [Far]

Virtual badge: Bob
  Connected: yes
  Last notification: killed
  Vibration preview: active

Virtual emergency button
  [Press]
```

Marking Alice and Bob near makes Bob an eligible nearby target if Alice is an impostor and the ordinary kill rules allow it. Killing Bob through Alice's player screen then shows the notification on Bob's virtual badge. Stopping simulated updates lets us check that proximity expires.

This is not a separate game, an AI player system, or a requirement during a real QR round. Real phones still play the actual game. Multiple browser sessions let a developer test several players. A draggable floor plan is optional later; simple near/far controls are enough initially.

## First-round mechanics

- Host chooses impostor count, kill cooldown, emergency-button cooldown, meeting duration, and QR/proximity modes.
- Starting the round assigns roles and a small set of tasks.
- An alive impostor can kill an alive crewmate while play is active and their cooldown has expired. The chosen interaction mode must be satisfied.
- The victim gets a death screen and stays lying down at the location. Their eventual badge also vibrates.
- A living player finds the victim, presses Report on their phone, selects the victim from the full roster, and confirms. The server accepts reports only for killed players whose bodies have not been cleared by a meeting. We trust the finder to be physically there; no scan or presence check is required.
- A report or press of the shared emergency button interrupts active tasks and stops kills. The button can be a PC/tablet interface or an ESP32 input.
- Players discuss in person and vote on their phones. End voting when everyone eligible has voted or the timer expires. A tie, or skip tying/winning, ejects nobody.
- Once a meeting starts, all existing bodies are cleared from play. Dead players can leave their positions, remain silent, and continue tasks after the meeting. They cannot kill, report, call meetings, or vote.
- Crew wins when all assigned tasks are complete or all impostors are eliminated. Impostors win when living impostors equal or outnumber living crew. Check victory immediately after relevant state changes, before allowing another action.

## Mobile activities in the first release

All eight activities are in the initial scope. The durations below are target play times, not forced waits or strict timeouts. Build them incrementally, starting with Enter Code to establish the shared task flow.

| Activity | Interaction and completion condition | Target duration |
| --- | --- | --- |
| Connect Wires | Drag between matching colored terminals. Add symbols or labels so color is not the only cue. Complete all correct connections. | 10-20 sec |
| Enter Code | Briefly show a 4-6 digit code, hide it, and accept the matching entry. Offer a retry after a mistake. | 10-15 sec |
| Calibrate Signal | Adjust a slider to match a fluctuating signal to a marked target frequency. Hold the correct setting briefly to complete. | 15-25 sec |
| Charge Battery | Stop a moving indicator inside a target zone several times. Each successful stop adds charge. | 10-20 sec |
| Pattern Memory | Watch flashing colored buttons, then repeat the sequence. Give buttons distinct symbols as well. | 15-25 sec |
| Find Anomaly | Choose the incorrect symbol among similar symbols. Complete the configured set of rounds. | 10-15 sec |
| Align Circuits | Rotate tiles to connect a continuous path from start to finish. Generate boards from a known valid solution. | 20-30 sec |
| Crack Password | Use a small set of logic clues to discover a 3-digit PIN. Use puzzles with one valid answer. | 15-30 sec |

Each task uses the same small lifecycle:

```text
StartTask -> activity screen -> activity inputs -> TaskCompleted
                                |
                           cancel / meeting
```

Each built-in activity defines its configuration, progress, inputs, and completion check. The server creates the puzzle and owns task completion; the browser renders it and handles interaction/animation. For timing games, the browser can report the captured indicator position or timing result. We trust the player and do not need frame-by-frame server simulation.

Leaving a task or starting a meeting cancels the current attempt. A completed task stays complete. Wrong answers allow retrying. Later physical activities can use the same start/input/complete lifecycle.

Sabotages, cooperative stations, and special roles are later additions. First prove the basic round is fun.

## Cargo workspace and code structure

Use separate crates for boundaries with different responsibilities and build targets. They remain one application deployment, not independent services.

```text
Cargo.toml                       Workspace definition
crates/
  protocol/                      Shared IDs, data shapes, wire messages
  game/                          Rules, state, commands, events, activities
  server/                        Executable: networking and orchestration

firmware/                        Added when hardware work starts
  common/                        Shared device connection/feedback behavior
  player/                        Player badge executable
  station/                       Activity/emergency station executable

web/
  player/            Lobby, QR scanner, tasks, kill/report, meetings
  host/              Settings, start/reset, device pairing
  emergency/         Shared PC/tablet emergency button
  simulator/         Hardware test screen: presence and virtual devices
  activities/        One UI module per mobile activity
  interactions/      QR and nearby-target selectors
  shared/            Visual components and connection state
```

| Crate | Owns | Must not own |
| --- | --- | --- |
| `protocol` | IDs, shared activity input/data shapes, client/device messages, presence updates, feedback commands | Game decisions, sockets, hardware drivers |
| `game` | In-memory game state, role/task assignment, command validation, interaction requirements, activity rules, meetings, victory, events | HTTP/WebSocket connections, camera handling, BLE/UWB drivers, GPIO |
| `server` | Client sessions, connections, current presence table/expiry, clock/randomness inputs, command ordering, views, event-to-feedback routing | A second implementation of game rules |
| `firmware/common` | Device registration, reconnection, translating notifications to device behavior, shared named-input handling | Roles, kills, voting, task completion decisions |
| `firmware/player` | Badge-specific ranging integration, motor, battery, indicators | Station-specific wiring |
| `firmware/station` | Activity inputs/outputs, emergency button, station-specific ranging integration | Player role logic |

Dependencies point in one direction:

```text
server ------------> game ------------> protocol
   +---------------------------------> protocol

firmware/player ---> firmware/common -> protocol
firmware/station --> firmware/common -> protocol
```

The simulator browser uses the server's protocol, just like real devices. It does not need its own Rust crate. Browser code uses matching frontend message types; document/update them alongside the Rust messages initially. Automatic type generation can be added if maintaining the small shared schema becomes a problem.

Keep the protocol free of web-server and chip-specific dependencies. Group browser and device messages separately inside it, so embedded builds can use only the device-facing portion. Choose any embedded allocation/`no_std` constraints when selecting the actual firmware platform; do not force those constraints onto the host game engine now.

Inside `game`, keep eight activity implementations as modules behind a common start/input/completion flow. A built-in activity enum and dispatch are enough; eight separate crates and dynamic activity plugins would add little value.

Inside `server`, keep presence, sessions, transport, and device routing as modules. Add a separate adapter crate only if a real integration needs distinct dependencies or reuse. BLE/UWB drivers belong with the hardware that runs them; the server receives the normalized observations.

The game receives current time, generated puzzle/role randomness, and presence facts as inputs. It never opens a socket, waits on a timer, or reads a radio. The server drives deadlines and calls into the game. This makes rules testable without phones or firmware.

These frontend directories are logical areas in one web app. Select the frontend framework during implementation. Keep chip toolchain configuration local to firmware; the server and game checks must run without an ESP32 toolchain. Firmware binaries are built/flashed explicitly for their target.

## Build plan

1. **Workspace and playable QR round foundation.** Create `protocol`, `game`, and `server` crates plus the frontend. Implement lobby, reconnect, role assignment, player/station QR codes, Enter Code, kills, body reports without QR, shared emergency-button screen, meetings, voting, and victory. All state in memory. Use the supplied visual direction from the start.
2. **Complete the mobile activities.** Implement the other seven activities using the same task flow. All eight belong to the first release.
3. **Hardware test screen.** Near/far controls, expiry, nearby target/station views, and QR/proximity mode selection. Pair virtual badges/stations, show feedback, simulate station/emergency button presses and disconnections. Use the same game commands as the phone UI. Try a full round with multiple phones and no ESP32s.
4. **Real hardware, after the software release.** Add shared firmware and player/station executable crates. Implement a presence adapter and feedback transport, then validate with development boards before designing player and activity PCBs. UWB can later replace the presence adapter while keeping the game interface.

Keep verification focused: illegal kills, cooldowns, expired presence, body reports, both emergency-button inputs, meeting voting/ties, task completion, victory, and reconnect behavior. Play each of the eight activities, then manually play through a complete QR round and a simulated-proximity round.

## Why no database?

There is nothing we currently need to retain between server runs. Roles, votes, cooldowns, tasks, and presence all belong to the current round. A browser refresh reconnects to the state still held by the server.

The explicit tradeoff is that a server restart loses the round. That is acceptable for this version. If restart recovery becomes useful, add a local state snapshot then; it does not require redesigning the game around persistence now.

## Software release extension

The running implementation now supports seventeen activity kinds. Nine mobile games extend the original eight: Chemical Mixing, Data Transfer, Find the Fault, Frequency Hunt, Code Breaker, Timing Challenge, Route the Power, Switch Sequence, and Cargo Sorting. Hardware examples are future attachments; every game currently runs on a phone.

Activity kinds and physical stations are separate. A station has a unique ID, kind and location; repeated kinds at different locations have independent QR identities and task progress. The host can add/remove placements while in the lobby. Player tasks reference station IDs; activity modules dispatch by kind.

The host sets `player_count` (optional expected capacity) and `impostors`. There is no fixed maximum for either. Automatic capacity uses the joined roster. Configured capacity must be filled before starting; the final crew count must exceed impostors. Six virtual players are a demo fixture, never a release limit.
