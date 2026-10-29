# Deploy and play Impostor IRL

## 1. Build and run one game server

You need current stable Rust and Node.js 22 or newer on the computer that will run the game. Keep the whole checkout on that computer. From its root directory:

```sh
npm --prefix web ci
npm run build
npm start
```

Open `http://localhost:3001/host` yourself. Players use `http://localhost:3001/` when testing on this same computer. The Rust server serves the built PWA, HTTP API, and WebSocket updates together; you do not need Vite or a database for a game night.

Leave the terminal running and prevent the computer from sleeping. Ctrl+C stops the server. The current build locates `web/dist` relative to its compiled source checkout: build on the destination machine and keep that checkout in place. A copied executable by itself is not a deployment bundle.

Run exactly one server instance for the group. All state is in memory: restarting loses the room, players, settings and round. Rejoining works while the same server remains running. Reset game keeps players and placements in that running instance.

## 2. Give phones a reachable address

### Local Wi-Fi test

Connect the server and phones to the same Wi-Fi. On Windows run `ipconfig` and find the active Wi-Fi/Ethernet IPv4 address, for example `192.168.1.50`. Players then open:

```
http://192.168.1.50:3001/
```

Allow the server through the computer firewall on your private network if prompted. Guest networks may block devices from reaching one another. `localhost` on a phone means the phone itself, not the host PC.

HTTP over a LAN works for joining, playing and the scanner's manual printed-code entry. Phone camera scanning and PWA installation need a trusted HTTPS address. Do not expect a certificate warning or an ordinary HTTP LAN address to provide those browser features.

### HTTPS deployment for phone cameras

Run the same build on an always-on computer/server. Put an HTTPS reverse proxy in front of port 3001 and use one stable address for everyone. For example, with Caddy installed, point your domain DNS to that computer and make ports 80/443 reachable. Create `Caddyfile`:

```caddyfile
game.example.com {
    reverse_proxy 127.0.0.1:3001
}
```

Replace `game.example.com` with your domain. Keep `npm start` running and run `caddy run --config Caddyfile` in another terminal, or run both through your server's service manager. Players open `https://game.example.com/`; the host opens `https://game.example.com/host`. Route the entire site, including `/api` and WebSockets, to the same Rust process. Use one instance, without multiple replicas or automatic sleep during play.

Caddy handles certificates for a reachable domain and proxies WebSockets. See the official [HTTPS setup](https://caddyserver.com/docs/quick-starts/https), [reverse proxy guide](https://caddyserver.com/docs/quick-starts/reverse-proxy), and [WebSocket support](https://caddyserver.com/docs/caddyfile/directives/reverse_proxy#streaming).

Host URLs and APIs are unauthenticated in this trusted-friends version. Player screens do not link to them; knowing a URL still allows opening it. Use this within your trusted group. URL-only discovery does not provide a host password or access restriction.

## 3. Set up the room

1. Open `/host` directly. Host navigation is absent from the player screens.
2. Set Expected players, Impostors, and Activities per player. Remaining players become crew; crew must outnumber impostors. Leave Expected players blank to use everyone who joins. Leave Activities per player blank to assign every placement.
3. Choose QR codes for kill and task interaction for a hardware-free game night. Choose either/proximity when using the simulator. Set kill, emergency and meeting timers, then save settings.
4. Add/remove activity placements. Give each a real location, such as Connect Wires at Scenerommet and another Connect Wires at Kitchen. Both get separate QR codes and separate task progress. Save location edits.
5. Open `/print`. Set the address to the actual reachable HTTPS/LAN game address, not localhost. Print the join code and station cards. Put each station code at its named location.
6. Friends open `/` or `/join`, enter the room code and distinct names, and wait in the lobby. A configured player count must be filled before starting. You can edit names in the host roster.
7. Print player QR badges after players have joined. Each player carries their own code or shows My badge on their phone. These support QR kills; they are not body markers.
8. Open `/emergency` on a shared PC/tablet at the meeting point. Enable Meeting horn if wanted, allow audio with that button, and check Test sound. Leave it open to receive meeting events.
9. Back in `/host`, press Start mission. Players see their private roles and assignments.

## 4. Play

- Crew walk to assigned task locations, open the task, scan that station's code and finish its mobile puzzle. Only assigned work contributes to crew progress.
- Impostors use Scan & kill, scan a nearby player's badge and confirm the target. Physical proximity is trusted in QR mode. The server enforces roles, phase and cooldowns.
- A victim lies down where they were killed and stays silent. Another player uses Report body and selects the victim's name. No printed body cards are needed. After a meeting, ghosts may finish assigned tasks.
- Anyone can press the shared emergency button when its cooldown allows it. Gather, discuss in person and vote on phones. Dead players cannot vote. A tie/skip ejects nobody; voting or the timer resolves the meeting.
- Crew win by finishing their assigned tasks or eliminating all impostors. Impostors win when they reach parity with living crew.

## 5. Rejoin and start another round

Refresh or reopen the same address to resume the remembered player. If browser storage is missing, use the same name and room code to reclaim that player. This requires the original server to still be running.

Open `/host` directly to manage the next round. Reset game is at the top of every host page. Confirm to clear roles/votes/progress while keeping players and placements, then edit setup and start again. Setup is locked during an active round and provides Reset & edit setup.

## 6. Preview alone

Open `/dashboard` directly and choose Try solo demo, or launch the demo from `/host` or `/simulator`. The demo supplies six virtual players; it is not the real player limit. Switch characters with the demo operative selector. `/activities` lets you practice every puzzle without changing round progress. `/simulator` can refresh near/far pairs and show virtual badge/horn outputs. These preview tools are not linked from player entry or join screens.

If the preview says Connecting, start the server again. If images or layout look stale after an update, reload the page. Rebuild before `npm start` when code changes.
