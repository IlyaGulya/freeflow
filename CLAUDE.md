# Wrenflow

macOS menu bar app for speech-to-text dictation. Hold a key to record, release to transcribe.

## Build & Run

```bash
just build  # Build debug .app bundle (Rust → UniFFI → Swift → Bundle → codesign)
just run    # Build, kill running instance, and (re)launch the app
just clean  # Remove build/ and .build/
just dmg    # Build + create DMG installer
```

`just run` automatically kills any running Wrenflow instance before launching, so it always starts fresh.

The app is built as `build/Wrenflow Debug.app` via justfile (Rust + Swift Package Manager).
Do NOT run the binary directly from `.build/debug/Wrenflow` — always use `just run` (or `open "build/Wrenflow Debug.app"`).

## Project Structure

- `Sources/` — all Swift source files (flat, no subdirectories)
- `Resources/` — app icon source
- `Package.swift` — SPM manifest
- `justfile` — builds .app bundle, codesigns, creates DMG
- `Info.plist`, `Wrenflow.entitlements` — app metadata

### Key Files
- `AppState.swift` — central state, `@Published` properties, transcription pipeline
- `AppDelegate.swift` — app lifecycle, setup wizard flow, menu bar setup
- `SetupView.swift` — onboarding wizard (multi-step)
- `SettingsView.swift` — settings window
- `LocalTranscriptionService.swift` — on-device Parakeet transcription
- `TranscriptionService.swift` — Groq (Whisper) cloud transcription
- `HotkeyManager.swift` — global hotkey monitoring

## Data Storage

- Debug app SQLite: `~/Library/Application Support/Wrenflow Debug/PipelineHistory.sqlite`
- Release app SQLite: `~/Library/Application Support/Wrenflow/PipelineHistory.sqlite`
- CoreData uses `Z`-prefixed tables (e.g. `ZPIPELINEHISTORYENTRY`) and `Z`-prefixed columns (e.g. `ZMETRICSJSON`)
- Pipeline metrics stored as JSON in `metricsJSON` column via `PipelineMetrics` (Codable)

## Architecture Notes

- SwiftUI app with `@EnvironmentObject` AppState
- Two transcription providers: Local (Parakeet via FluidAudio) and Groq (Whisper API)
- `TranscriptionProvider` enum in `AppState.swift`
- `localTranscriptionService.initialize()` downloads the model — only call after user explicitly chooses local transcription
- Setup wizard state managed via `SetupStep` enum with rawValue-based navigation


<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:b9766037 -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd dolt push
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
<!-- END BEADS INTEGRATION -->
