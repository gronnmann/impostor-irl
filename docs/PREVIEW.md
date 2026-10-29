# Preview and solo play

## Start locally

Install current stable Rust and Node.js (Node 22+), then run from the repository root:

```powershell
npm --prefix web install
npm run dev
```

Open **http://localhost:5173**. The runner builds the Rust server and starts it on port 3001, plus the frontend dev server on 5173. Stop both with Ctrl+C. If either port is occupied, stop the previous instance before restarting.

To preview the production build through one server:

```powershell
npm run build
npm start
```

Open **http://localhost:3001**. Run from this checkout; the build includes a reference to its `web/dist` directory. Copying just the executable is not a standalone distribution.

## Routes

| Route | Purpose |
| --- | --- |
| `/` | Mobile game entry, or the saved player's game |
| `/dashboard` | Explicit host mission dashboard |
| `/host` | Round settings and start/reset |
| `/join` | Join from a player's phone |
| `/play` | Full-screen mobile game scene; phone-width even on desktop |
| `/activities` | Practice all seventeen puzzles without affecting the round |
| `/crew` | Trusted host roster and player QR cards |
| `/print` | Printable player and activity QR codes |
| `/simulator` | Fake presence and virtual badge feedback |
| `/emergency` | Shared PC/tablet emergency button |

Host pages intentionally expose all game state and have no authentication for this trusted-friends release. Give real players the join link. Player screens contain no host links. Hosts enter `/host` or `/dashboard` directly. These URLs are intentionally unauthenticated for the trusted-friends release; hiding navigation is not access control.

## Test alone, with no hardware or people

1. Open `/dashboard` directly and start a solo demo from the host dashboard. This creates Nova, Atlas, Echo, Luna, Orion and Vega and opens the mobile game. Nova is the impostor; the others are crew.
2. Use the demo player selector to choose an operative, then open **Play game**. This uses that player's actual server session. During a real round this switching control is unavailable. Edit names in the host roster or crew screen to use your friends' names.
3. As a crewmate, open an activity and complete it. In the demo, starting a task simulates presenting the station QR. In normal play the player scans the actual station code or uses configured proximity.
4. Use `/simulator` to set two players near. Choose Nova in **Play as**, then return to `/play`: the other player appears under nearby targets. Confirm the kill.
5. Switch to the victim to see their death state. Their virtual badge shows kill feedback. In real play they lie down until a meeting clears the bodies.
6. Switch to another living player, press **Report body**, and select the victim. No QR is needed. Inspect the horn and badge notifications in the simulator.
7. Switch between living players and cast votes. Dead players cannot vote. Resolve early using the simulator's **Resolve meeting now**, or wait for the voting timer.
8. Use the shared `/emergency` page to call another meeting once the cooldown permits. Set **Meeting horn: ON** once to allow browser audio; the page then sounds for new meeting horn signals, including reports from other terminals. Set it to **OFF** to mute meeting sounds. **Test sound** previews the horn explicitly. Simulator output also shows the future hardware horn signal.
9. Load a new demo whenever you want a clean test. This replaces the current round. Use independent browser tabs or browsers for simultaneous players; each tab retains its own selected session. Demo replacement invalidates previous demo tokens.

Presence stays refreshed while the simulator screen is open. **Release** stops refreshing a pair, so its entry disappears after five seconds. **Set far** changes it immediately. In a separate browser tab, keep the simulator open while using a player terminal to test continuous presence.

## Activities

The practice deck is available independently of a round. Wires support both dragging and tapping terminals. Memory/wire activities use shapes as well as color. Animated activities honor reduced-motion preferences for decorative effects; battery timing remains an essential interaction.

Real game activity attempts are stored server-side. Refreshing the tab restores an active attempt. Wrong answers can be retried. A meeting or death cancels the attempt. Completed tasks remain completed. Network errors offer **Retry submission** so a finished timing puzzle need not be replayed.

## Phones and QR scanning

Connect phones and the server to the same network. Replace `localhost` with the host's reachable address for phone access. A phone's localhost means that phone, not your PC.

In-app camera access and PWA installation need a trusted HTTPS origin on phones. Plain LAN HTTP is useful for layout/manual-code testing, but does not enable camera access. Arrange local trusted HTTPS or an HTTPS forwarding setup before a real QR game. On the host PC, `http://localhost` supports camera development.

Print station/player QR codes from `/print`. The scanner also accepts the printed text, for example `impostor:station:code`, when no camera is available. The QR generator and decoder are tested together. Physical phone camera scanning and install prompts still need checking on your actual devices.

Set activity locations in `/host` while the game is in the lobby. For example, set Connect Wires to `Scenerommet`; player assignments and printed cards then read **Connect Wires (Scenerommet)**. Locations survive a round reset but are lost on a server restart. QR contents stay the same when a station moves.

The `/print` page offers checkboxes for player badges and activity stations, a full printable sheet, and individual **Download PNG** links. Set the reachable game address before printing the join QR. Player cards include the assigned colored avatar. No body cards are used.

The service worker caches visited frontend assets for a reconnect screen and practice. It does not cache game API responses or enable offline multiplayer. The Rust server must remain running. Restarting it loses the round, by design.

## Easy rejoining and names

Your browser remembers the last player token, name and room code. Refreshing or reopening the game resumes that player while the server still has the round. Tabs retain their own selected demo identities.

If browser storage is missing, enter the **same name and room code** on the join screen. Names are trimmed and matched without case sensitivity. The server returns your existing identity, role and task progress, including during meetings or after a round ends. It does not add a duplicate player. This is intentionally trust-based for your group; no account or password is required.

The host can edit names in the lobby, and at any time in the solo demo. Names must be distinct so recovery remains unambiguous. A rename changes the labels on player QR cards without changing their QR contents. Opening the join page while a saved player is active offers **Jump back in!** or **Join with another name**.

## Checks

```powershell
npm test
npm --prefix web run build
npm run test:smoke
cargo fmt --all -- --check
```

The smoke check builds and launches an isolated server on an unused port. It never resets your preview game. It covers all seventeen server-validated activities, fake tasks, presence expiry, kills/privacy, reports, horn feedback, voting/victory, emergency cooldown, reconnect, normal start and production frontend routes. Build the web app before running it.

## Group sizes and repeated placements

In `/host`, set **Expected players** and **Impostors**, then save. There is no artificial player or impostor maximum. The crew must outnumber impostors when the round starts. Leave Expected players blank to use the joined roster; otherwise the server waits for that many players and reserves that capacity. The six-person demo is only a preview preset.

Add an activity placement with a location such as `Scenerommet` or `Kitchen`. Multiple placements of the same game are supported. Each has a unique station ID and printable QR code at `/print`; task progress is tracked separately per placement. Edit/remove placements in the lobby. Locations appear on the player task list and active puzzle. Venue configuration survives reset and demo loads during the same server session.

## Reset and activity budgets

Every host page has a **Reset game** button at the top. It opens a confirmation that explains what will be cleared. Confirming returns to editable setup while keeping joined players and station placements. During an active round, setup shows **Reset & edit setup** above the locked controls.

**Activities per player** assigns that many distinct physical stations to each crewmate. Assignments are spread across the available placements. Leave blank to assign all placements. Players see only their assigned stations; the win progress counts only assigned work. The practice deck still includes every game.
