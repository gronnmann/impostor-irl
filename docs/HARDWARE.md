# Future hardware specification

Status: design for a later hardware phase; no firmware, PCB files, or physical-device transport ships in this software release. The working contracts are [IMPLEMENTATION-CONTRACT.md](../IMPLEMENTATION-CONTRACT.md), [BACKEND-NOTES.md](../BACKEND-NOTES.md), [protocol](../crates/protocol/src/lib.rs), and the server's [presence](../crates/server/src/presence.rs) and [feedback](../crates/server/src/feedback.rs) modules. Where [ARCHITECTURE.md](../ARCHITECTURE.md) describes future device connections or named inputs, those are intentions, not implemented wire messages.

Build for trusted friends: one Rust server, in-memory pairing and game state, phones for roles/tasks/votes, optional badges and stations. No anti-cheat, database, microservices, or message broker. Hardware must not become necessary to play the QR version.

## First hardware and decisions to close

Start with two player development boards and one station board on USB power. Use an Espressif ESP32-C3 development board as the initial candidate: the current official guide lists `ESP32-C3-DevKitM-1-N4X`, with Wi-Fi, BLE, header access, and USB-to-UART programming. Check the exact purchased revision against its schematic; the older unsuffixed ordering entry is marked EOL. This board is a bringup candidate, not a final wearable or a battery charger. Use the board's documented USB power option during bringup. [Espressif board guide](https://docs.espressif.com/projects/esp-dev-kits/en/latest/esp32c3/esp32-c3-devkitm-1/user_guide.html)

Use a module-based custom PCB after bringup rather than starting with a bare RF chip. Candidate wearable module: ESP32-C3-MINI-1 with PCB antenna; assess an external-antenna variant only if enclosure measurements justify it. A station with more simultaneous I/O may need a larger board/module or an I2C GPIO expander. Do not commit to the C3 until the pin budget includes programming, buses, reserved pins, buttons, and outputs. The module and board guide document the antenna variants and boot-strapping restrictions. [Espressif board/module reference](https://docs.espressif.com/projects/esp-dev-kits/en/latest/esp32c3/esp32-c3-devkitm-1/user_guide.html#related-documents)

Before schematic freeze, fill in this small build record:

| Decision | Initial direction | Must be recorded before PCB ordering |
| --- | --- | --- |
| MCU / module / dev board | ESP32-C3 candidate; station may need more I/O | Exact ordering code, revision, flash/RAM fit, toolchain and schematic |
| Wearable actuator | Small ERM vibration motor with dedicated driver | Motor model, ratings, driver model and measured peak load |
| Battery / charger / regulator | Protected single-cell rechargeable pack and matched power-management design | Pack model/capacity, charger, regulator, charge limits, thermal review |
| Station loads | USB-powered logic, button inputs, LED indication; optional separately rated horn module | Connector types, load ratings, output driver and enclosure |
| Pins / expansion | Logical names below; no numeric pin assignments yet | Reviewed pin table, voltage levels, connector orientation, reserved pins |
| Presence tuning | BLE first | Venue measurements, filter thresholds, update rate and expiry |
| Runtime | Measure before choosing pack size | Host's session-duration target, measured runtime and reserve |

## Player wearable PCB

The wearable advertises its device identity, observes nearby supported devices, sends normalized presence, receives private player feedback, and shows connection/power status. It does not assign roles or decide whether a kill succeeds.

Required PCB functions:

| Function | Requirement |
| --- | --- |
| MCU and radio | Selected ESP32 module, programming/debug access, reset/boot access, identity label matching the enclosure |
| Vibration | Dedicated motor driver, local supply support, bounded patterns and output off at boot/disconnect; never drive a motor directly from GPIO |
| Battery | Selected protected pack, mechanically secured connector, power switch or ship/off function, low-battery observation and orderly output shutdown |
| Regulator | Regulated logic rail over the selected pack's usable range; size for simultaneous radio, motor-driver, LED and expansion peaks, not average current alone |
| USB charging | Charger matched to the selected cell, input protection/current limiting, temperature/charge status, and power-path management if operation while charging is supported |
| Indicators | Small connection/low-battery indicator; private death feedback via vibration rather than a public role indicator |
| Mechanical | Insulated enclosure, retained battery, strain relief, accessible charging/programming, wearable attachment and protected test points |
| Expansion | Reserved, documented bus access as below; unpopulated in the BLE-only build |

Espressif's C3 schematic checklist recommends a 3.3 V supply with at least 500 mA capability for its single-supply design. Treat that as MCU reference guidance, not the total badge current budget. Select regulator topology and transient/decoupling design from the actual input range and measured combined load. [Espressif power checklist](https://docs.espressif.com/projects/esp-hardware-design-guidelines/en/latest/esp32c3/schematic-checklist.html#power-supply)

For an initial haptic breakout, TI's DRV2605L is a candidate with I2C control and support for ERM and LRA actuators. The final motor/driver pairing and calibration remain a hardware decision; supporting both actuator types in software is unnecessary for the first badge. [TI DRV2605L](https://www.ti.com/product/DRV2605L)

A charger with actual system power-path management is the appropriate class if the badge must run while charging; TI's BQ24074 is one documented example, not a selected BOM item or a universal charger for arbitrary packs. Charging limits and temperature handling must follow the chosen cell and charger documentation. This specification intentionally gives no battery wiring, charge-resistor values, or charging-current prescription. Use a documented assembled battery/power subsystem during prototypes, and have the final power design reviewed before wearable use. [TI BQ24074](https://www.ti.com/product/BQ24074)

Place the module antenna at the board/enclosure edge using its documented keepout. Keep battery, motor, cables, USB hardware and metal fasteners away from the antenna region; verify the complete enclosed badge while worn. Follow the selected module's layout rules rather than guessing a universal clearance. [Espressif module placement and RF layout](https://docs.espressif.com/projects/esp-hardware-design-guidelines/en/latest/esp32c3/pcb-layout-design.html#general-principles-of-pcb-layout-for-modules-positioning-a-module-on-a-base-board)

## Station PCB and connectors

Use one station firmware binary with a small board/input map for each installation. Begin with a USB-powered button-and-LED board. A station may serve one of the seventeen activities, the shared emergency button, or the `horn` output; those are different bindings, even if one physical controller eventually has several connectors.

Provide labeled, keyed connectors where practical and document signal direction, logic voltage, ground reference, allowed cable length and load rating. Connector names below are logical labels, not a physical pinout:

| Connector / function | Required behavior |
| --- | --- |
| `INPUT_CONFIRM`, `INPUT_CANCEL` | Debounced buttons emitting named press edges; no repeat while held unless an activity explicitly needs it |
| `INPUT_0` through `INPUT_3` | Four debounced activity buttons for the first memory-station prototype |
| `INPUT_EMERGENCY` | Shared meeting-point button, mapped to `emergency`; not a per-player allowance |
| `STATUS_LED` / activity LED connector | Rated LED output through appropriate limiting/driver; boot/disconnect state is known |
| `HORN_CONTROL` | Logic control of a separately specified low-voltage horn/sounder driver module; bounded pulse and off on boot, timeout or lost connection |
| `I2C_EXP`, `SPI_EXP`, `UART_EXP` | Expansion access with documented voltage and power budget; no assumed cross-compatibility between peripherals |
| Programming / power | Accessible debug/programming and selected USB input, with backfeed prevention appropriate to the design |

A GPIO is a signal, not a horn or LED-strip power supply. Specify an output stage appropriate to the actual load, suppression/protection where required, and a separately rated supply if necessary. No mains switching or direct horn wiring is specified here. Keep the horn disconnected or use a quiet indicator during initial bringup. Activity displays, keypad matrices, knobs and extra outputs are optional later station attachments; the first PCB need not implement all seventeen puzzles physically.

Reserve I2C for the haptic driver, later expanders/sensors; SPI with chip-select, interrupt and reset capacity for a possible UWB radio; UART TX/RX for diagnostics or a later module with a documented serial protocol. Include reference ground and only explicitly budgeted power on expansion connectors. Check bus sharing, pull-ups, voltage compatibility, boot straps, flash connections and programming conflicts before assigning pins. If all three buses cannot be exposed simultaneously on the chosen MCU, select a larger pin budget before layout; do not promise a header fixes that constraint.

## Firmware responsibilities

These are future crates, not existing build targets:

| Location | Owns |
| --- | --- |
| `firmware/common` | Device identity/registration, server connection/reconnect, heartbeat, message validation, feedback dispatch, named-input envelope and shared presence normalization |
| `firmware/player` | Badge executable: BLE/UWB adapter, selected haptic driver, battery/status handling and board mapping |
| `firmware/station` | Station executable: debouncing, named inputs, activity LEDs, emergency input, horn limits and station radio adapter |
| `crates/protocol` | Future device-facing types alongside the existing software contract; no GPIO, driver, socket or game rules |
| `crates/server` | Pairing, connection ownership, presence expiry, actor/session binding, game-command serialization and feedback routing |

Firmware maps board-specific GPIO to names; server code maps names to activity behavior. Keep motor timing and radio filtering out of `game`. Keep chip toolchains local to explicit firmware builds so `cargo test --workspace` for the software release does not require an embedded toolchain. The current protocol uses `serde_json::Value` and host-oriented types; choose allocation/serialization constraints when selecting the firmware stack, rather than claiming it already builds for a `no_std` target.

## Existing semantic contracts and exact REST examples

All bodies in this section go to `POST /api/command`. IDs are strings. Use the actual `player_id` returned by join or a current snapshot, and the token belonging to that player. The examples assume `player-2`, `player-5`, `TOKEN_FOR_PLAYER_2`, and `SESSION_FROM_START_TASK` have been replaced with current values. Device IDs such as `badge-07` are future pairing keys, never presence subjects.

### Presence

```json
{"command":{"type":"presence","first":"player-2","second":"memory","near":true,"valid_for_ms":2000}}
```

The unordered pair must contain two distinct existing player/station IDs. Station IDs are exactly `wires`, `code`, `signal`, `battery`, `memory`, `anomaly`, `circuits`, `password`. Player/player pairs use the same shape, replacing `memory` with e.g. `player-5`. The server accepts `valid_for_ms` from 1 through 60000 and computes `expires_at` from receipt time. Missing/expired presence fails proximity checks; explicit `near:false` also fails. Send a far update when known, and let loss of observations expire rather than refreshing a stale near reading. Initial prototype tuning target: refresh fresh observations every 500 ms with a 2000 ms lifetime; these are proposed adapter settings, not measured radio guarantees.

Presence never causes a kill or starts a task by itself. The paired player still requests an action:

```json
{"token":"TOKEN_FOR_PLAYER_2","command":{"type":"kill","target_id":"player-5","interaction":"proximity"}}
```

```json
{"token":"TOKEN_FOR_PLAYER_2","command":{"type":"start_task","station_id":"memory","interaction":"proximity"}}
```

Only current near presence permits the proximity path. Existing mode values are `qr`, `proximity`, `either`; there is no `ble` or `uwb` game mode. Task presence is checked at start, not continuously. Body reports select a player ID and do not require proximity.

### Named inputs: proposed device envelope, existing command mapping

The following is a concrete future mapping example for a memory station, not accepted configuration or a message type implemented in this release. Logical connector labels deliberately avoid unverified GPIO numbers:

```json
{
  "device_id":"station-memory-01",
  "station_id":"memory",
  "inputs":{
    "INPUT_0":{"name":"choice_0","edge":"pressed"},
    "INPUT_1":{"name":"choice_1","edge":"pressed"},
    "INPUT_2":{"name":"choice_2","edge":"pressed"},
    "INPUT_3":{"name":"choice_3","edge":"pressed"},
    "INPUT_CONFIRM":{"name":"confirm","edge":"pressed"},
    "INPUT_CANCEL":{"name":"cancel","edge":"pressed"}
  }
}
```

Proposed event on the future device connection:

```json
{"type":"input","device_id":"station-memory-01","session_id":"SESSION_FROM_START_TASK","name":"choice_2","edge":"pressed"}
```

The server must explicitly bind this station to one player's current attempt before accepting inputs. Nearness alone cannot choose a player when several badges are present. For the first prototype, select/confirm the participant through their phone or trusted host; serialize use of each physical station. Validate device binding, supported input name, session owner, station, phase and current attempt before accumulating an answer. GPIO numbers and raw switch readings never enter game rules.

For example, five `choice_*` presses yielding `[0,2,1,3,0]`, followed by `confirm`, translate to this existing command for that player:

```json
{"token":"TOKEN_FOR_PLAYER_2","command":{"type":"submit_task","session_id":"SESSION_FROM_START_TASK","result":{"sequence":[0,2,1,3,0]}}}
```

This succeeds only if that is the current challenge's sequence. Firmware must use the server-issued challenge, never a fixed demonstration answer. The existing server accepts a final result object, not a stream of button events; named-input accumulation is new adapter work. `cancel` maps to `{"type":"cancel_task"}` under the player's session. Meetings/reset invalidate the binding; wrong answers leave the attempt open. For other future stations, translate into the existing result keys (`connections`, `answer`, `value`/`held_ms`, `hits`, `index`, `rotations`) exactly as documented in the integration contract. Physical success LEDs follow accepted results, not local guesses. Fake impostor task completions must preserve existing behavior: feedback without crew progress.

A separate shared emergency-button binding maps one debounced `emergency` press to this existing anonymous command:

```json
{"command":{"type":"emergency"}}
```

The game checks phase and shared cooldown. Handle `ok:false` even when HTTP status is 200. Do not queue a press during disconnection and trigger a meeting much later.

### Feedback and persistent indicators

The current feedback record shape is:

```json
{"id":"feedback-42","at":1790000000000,"target":"player-2","kind":"killed","message":"Example private death notification"}
```

This is an illustrative snapshot record, not a client command. Route by `target` and `kind`, never parse `message` to decide hardware behavior.

| Current kind / target | Actual server routing | Future hardware behavior |
| --- | --- | --- |
| `killed` / victim player ID | Victim only | Private bounded death vibration |
| `task_complete` / acting player ID | Acting player, including fake task completion | Short completion vibration |
| `meeting` / each player ID | Every player | Distinct meeting vibration |
| `meeting` / `horn` | Additional record on meeting start | One bounded horn pulse |
| `game_end` / each player ID | Every player | Distinct end vibration |

Automatic task feedback currently targets a player, not the activity station. Automatic game-end routing does not sound the horn. Station LEDs must be restored/updated from the selected player's task/session state through the future adapter, or receive an explicitly added targeted update. Architectural `Notify`, `SetIndicator(idle|active|complete)` and `SoundHorn` describe future device operations; those strings are not additional accepted REST commands today.

Host testing can already emit the four supported kinds to a player ID, station ID, or literal `horn`:

```json
{"command":{"type":"test_feedback","target":"horn","kind":"meeting"}}
```

The snapshot's `feedback` array is only the latest 100 simulator records. It is not a reliable hardware delivery queue: records persist across snapshots and can be truncated. Reconnecting a device must restore current status without vibrating or sounding the horn for old records. Deliver fresh effects only to currently connected paired outputs; deduplicate an effect ID within the connection. A disconnected badge never blocks a valid kill. Persistent indicators derive from current phase/own alive/task state, not from replaying feedback history.

## Adding the real device connection later

The release exposes REST commands and browser snapshot streaming, not a production physical-device protocol. `/api/ws` sends full snapshots; it does not dispatch incoming application commands. Do not point firmware at it and assume device registration or bidirectional messages exist. REST is the integration seam for a development-board bridge that translates observations/button actions into the existing commands above. A trusted host bridge can inspect host snapshots during bringup, but must not broadcast that private state to all badges.

Add the physical path inside the existing server process:

1. Define device-facing registration, heartbeat, input, presence and output types in `protocol`. Select a single connection transport after board bringup, for example a dedicated Wi-Fi WebSocket endpoint; this endpoint does not exist yet. BLE presence and the server connection are separate concerns.
2. Add an in-memory registry keyed by stable `device_id`, with firmware/board revision, supported capabilities, paired player/station/output, current connection and last-seen time. Host pairing explicitly maps `badge-07 -> player-2`, `station-memory-01 -> memory`, and `meeting-controller-01 -> horn`. Connection health is distinct from player browser `connected` status.
3. Register/reconnect and acknowledge the selected binding. Replace a stale connection for the same device; reject unknown or conflicting bindings and unsupported messages. Re-pair after server restart and clear stale player/session bindings after roster replacement. This needs ordinary validation, not an anti-cheat identity service.
4. Translate trusted radio peer identities through that registry into existing presence subjects. Translate named inputs into the bound player's current commands; keep browser reconnect tokens on the trusted server/bridge rather than distributing them to every station. The game remains the sole task/kill/meeting validator.
5. Extend feedback routing at the point fresh feedback is produced, including timer-driven events and host test feedback. Send new effects directly to connected paired outputs; retain the recent feed for the simulator. Do not use the snapshot watch channel as an event queue: coalesced snapshots can miss effects. Add only the targeted persistent-state view needed by that device.
6. On timeout/disconnect, stop effects locally, clear attempt ownership as appropriate and let observations expire. On reconnect send current permitted status, with no replay of old horn/vibration. Scope effect IDs to the running game/server session so restart cannot collide with a device's deduplication cache.

Use the existing serialized server command path for adapter inputs. Do not implement a second rules engine or a separate hardware service. Preserve player privacy when sending initial status: host views contain other players' roles/tasks, while badges need only their own permitted state and outputs.

## BLE first, UWB later

For the first prototype, badges and station boards advertise a supported identity and scan peers. Badges must observe each other as well as stations to support player/player proximity; station-only scanning cannot provide that. A single designated observer/provider owns each unordered pair at a time, so reciprocal sightings or simulator overrides do not fight each other.

Convert BLE RSSI observations into near/far using a short sample filter, separate enter/leave thresholds and a freshness timeout. Choose the thresholds from measurements with people wearing the enclosures, including orientation, crowds and walls. RSSI is a noisy presence hint, not an exact distance. Missing samples become unknown via expiry. Keep raw RSSI and tuning in the adapter; the server/game contract stays the same.

Initial bridge direction: Wi-Fi to the server, BLE for local discovery/presence. ESP32-C3 shares RF resources between Wi-Fi and BLE using time division, so validate scanning, advertising and feedback latency together under traffic. Do not infer battery life or scan reliability from either radio operating alone. [Espressif RF coexistence guide](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c3/api-guides/coexist.html)

UWB requires actual UWB hardware at both ranging endpoints. An ESP32-C3's built-in BLE/Wi-Fi radio does not become a UWB radio through a firmware switch. A candidate later module is Qorvo DWM3000: it provides a UWB transceiver/antenna, uses a customer MCU, and exposes SPI. Prototype with its documented evaluation hardware before committing a footprint. [Qorvo DWM3000](https://www.qorvo.com/products/p/DWM3000), [official interface specification](https://store.qorvo.com/products/detail/dwm3000-qorvo/681949/)

The upgrade therefore needs a chosen module, reviewed power/logic/interface and antenna placement, driver, ranging exchange/scheduling, peer-ID binding, antenna-delay calibration and venue tests. Reserve SPI/interrupt/reset capacity now; do not claim I2C or UART can substitute for the selected module's SPI interface. Apply a calibrated distance threshold with hysteresis/freshness, then emit the same presence command. BLE may remain for discovery and Wi-Fi for server transport. Keep exactly one active presence provider per pair during migration. There is no need for coordinates, a room map or new kill/task rules.

## Bringup and acceptance gates

Complete these on development boards before laying out either PCB. Record board revisions, firmware versions, chosen settings, observations and pass/fail results. Numeric targets below are prototype acceptance targets, not claimed component performance.

| Gate | Procedure and pass condition |
| --- | --- |
| Basic board bringup | Flash both future binaries, identify each board, read inputs and blink a harmless status output; repeat power/reset cycles with no unintended motor/horn activation |
| REST translation | Bridge submits the exact presence/emergency/task shapes above; valid requests succeed, malformed or unknown subjects and invalid lifetime fail visibly; inspect `ok`, not only HTTP status |
| Pairing/privacy | Two badges bind to different real player IDs, one activity station to `memory`, one output to `horn`; wrong/duplicate bindings are rejected; neither badge receives another player's death/role/task details |
| Presence expiry | Fresh near permits a proximity action; explicit far, expired and missing observations reject it; reversed subject order updates the same pair; stopping one observer causes expiry within the configured lifetime |
| Radio behavior | Test worn badges facing/turned away, adjacent stations, walls and a full group with Wi-Fi traffic; document enter/leave thresholds and false-near cases; if reliability is inadequate retain QR while improving the adapter |
| Input mapping | Each debounced physical press produces exactly one named edge, held/bouncing buttons do not duplicate it; wrong station/session/owner cannot complete an attempt; cancellation, meeting and reset discard stale inputs |
| Task validation | Play the memory challenge using actual server challenge data; correct sequence completes once, wrong sequence stays open, duplicate/stale submissions fail; fake task completion does not advance crew progress |
| Emergency equivalence | PC emergency page and physical button use the same rule path; playing/cooldown checks agree; disconnected presses are discarded rather than replayed |
| Feedback routing | Kill vibrates only victim; completion only actor; meeting reaches all badges plus one horn output; game end reaches badges; station LED state follows its explicitly bound attempt, not another player's recent completion |
| Disconnect/reconnect | Disconnect during play, kill/task/meeting and reconnect; phone outcome still works, no old vibration/horn replays, persistent status restores, outputs stop safely and obsolete station ownership cannot act |
| Latency | Initial target: new accepted feedback reaches a connected prototype within 1 second on the test LAN; measure repeated trials during concurrent scan/advertise/network traffic and record misses |
| Power and enclosure | With the selected assembled power subsystem, measure radio-plus-vibration peak behavior, low-battery cutoff/status, charging temperature and session runtime; no resets or unintended outputs; repeat RF tests in the intended enclosure |

Only then order first PCBs: close the build record, review schematic/pin allocation and power/RF layout against the exact parts, and repeat these gates on the assembled boards. UWB has a separate acceptance gate with at least two UWB-equipped endpoints, calibration and obstruction tests; a spare expansion header is not a passed UWB implementation.

## Research baseline

Primary manufacturer documentation checked on 2026-10-29 is linked beside the recommendations. Recheck exact board revisions, ordering codes and datasheets when purchasing/layout begins. Component examples establish feasible functions; they do not constitute a validated BOM, schematic, battery assembly or measured performance claim.
