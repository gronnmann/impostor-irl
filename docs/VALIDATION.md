# Software release validation

Validated on 2026-10-29 in this workspace.

- `npm test`: 24 Rust engine/server tests and 6 browser utility tests passed.
- `npm --prefix web run build`: TypeScript and optimized Vite build passed.
- `cargo build -p server --release` and `cargo fmt --all -- --check`: passed.
- `npm run test:smoke`: isolated HTTP server passed all seventeen puzzle submissions, independent progress at repeated stations, 80 players / 12 impostors, configured capacity, rejoin, kill privacy, presence expiry, reporting, voting/victory, horn feedback, cooldowns and production routes.
- Collaborative browser at 390×844: `scripts/browser-qa.js` operated all seventeen real puzzle interfaces against the running Rust server. All passed; dragging displayed a loose wire before dropping it. No horizontal overflow observed. Synthetic pointer testing exercises event handlers and geometry; physical touch pointer capture still needs testing on a phone.
- Printing UI: 25 generated codes (join, 6 players, 18 stations), all downloadable. Two Connect Wires placements showed Scenerommet and Kitchen with distinct station QR contents. QR generator/decoder compatibility is also covered by the utility tests.

The six-person demo is a fixture, not a player limit. Physical cameras, phone installation prompts, real speakers, and future beacon hardware need real-device checks. No hardware or database ships in this release.

The host UI saved 80 expected players and 12 impostors; both numeric inputs had no maximum and Start stayed disabled while seats were unfilled. Adding and removing a repeated station through the UI also passed. Launching the solo demo afterward restored its six-player preset and preserved venue placements.

Current mobile task screenshot: [mobile-tasks.png](screenshots/mobile-tasks.png).

## Host recovery regression

Fixed missing styles for the actual `activity-node` markup; dashboard practice buttons now use a bounded flexible layout. Host pages have a visible top reset control and active-round setup explains the lock with a reset-and-edit action.

Added `activities_per_player` (optional; blank assigns all placements). The new Rust regression checks distinct assignments, rejects unassigned stations, verifies victory against assigned work only, and rejects invalid budgets/empty station setups. There are now 25 Rust tests.

Browser regression passed: emergency meeting → skip vote → confirmed vote → return to host dashboard → reset confirmation → editable setup → save 3 activities per player → start → 15 assigned tasks across 5 crewmates → player screen shows exactly 3. The desktop host dashboard had no horizontal overflow and activity nodes used their flex styling. Production frontend rebuilt and Rust release restarted.

## Player navigation separation

Removed host navigation from player entry, join and the game back button. Solo demo launch is now a host-URL action. Production build and utility tests passed; browser checks of `/` and `/join` found no host/sidebar links. Direct `/host` remains the host entry URL. Deployment and game-night instructions are in `DEPLOY-AND-PLAY.md`.
