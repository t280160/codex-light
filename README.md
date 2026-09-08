# CodexLight — Phase 1

Phase 1 is a Rust command-line program that reads the quota of the Codex account
already signed in on this computer. It calls the usage API first and falls back
to recent entries in Codex's local SQLite logs.

It does not contain the Tauri shell, Vue UI, tray icon, caching, notifications,
or autostart behavior planned for later phases.

## Run

Rust 1.75 or newer is required.

```bash
cargo run
```

Machine-readable output:

```bash
cargo run -- --json
```

Tests:

```bash
cargo test
```

## Phase 1 installers

Build the Apple Silicon macOS command-line installer:

```bash
./scripts/package-macos.sh
```

Build the Windows x64 command-line installer from macOS (requires MinGW-w64 and
NSIS):

```bash
./scripts/package-windows.sh
```

Unsigned packages are written to `dist/`. These install the Phase 1 CLI only;
they do not provide a menu bar or system tray UI.

## Desktop red-green-light app

The Tauri desktop build provides the visible menu bar/tray experience:

On macOS the menu bar item combines the quota-colored light with the current
5-hour remaining percentage. The popup's title bar is draggable, and its X
hides it back to the tray. A Rust background task refreshes the tray every five
minutes even while the popup is hidden, and sends fresh values back to the UI.
The interval can be changed in the popup to 1, 5, 10, 15, 30, or 60 minutes and
is retained across restarts.

```bash
npm install
npm run build:desktop:mac
```

The macOS `.app` and `.dmg` are written under
`src-tauri/target/release/bundle/`. The Windows x64 NSIS build can be produced
from macOS when MinGW-w64 and NSIS are installed:

```bash
npm run build:desktop:win
```

`CODEX_HOME` is honored when set; otherwise the program uses `~/.codex`.
Credentials are read locally and kept in Rust memory. The access token and auth
file contents are never printed or returned in command output.
