# Wrenflow

Cross-platform menu bar app for speech-to-text dictation. Hold a key to record, release to transcribe locally via Parakeet.

## Build & Run

All tools managed via mise. Run `mise install` first.

```bash
mise run build     # Generate bindings + XcodeGen + Flutter build macOS debug
mise run run       # Same + launch the app
mise run release   # Release build
mise run clean     # Remove all build artifacts
mise tasks         # List all available tasks
```

## Key Architecture Decisions
- **Flutter + Rust via rinf**: Dart for UI, Rust for all heavy logic (audio, transcription, hotkeys, paste)
- **Local-only transcription**: Parakeet TDT (on-device), no cloud APIs
- **raw-input crate**: CGEventTap for global hotkeys (replaces rdev which crashed on macOS)
- **Rinf signals**: Typed async message passing between Dart and Rust (no UniFFI)
- **XcodeGen**: project.yml generates xcodeproj (no manual pbxproj editing)
- **CocoaPods**: Required by rinf (SPM not yet supported by rinf)

## Data Storage

- SQLite history: `~/Library/Application Support/wrenflow/history.sqlite`
- Parakeet model: `~/Library/Application Support/wrenflow/models/parakeet-tdt/`
- Crash log: `~/Library/Application Support/wrenflow/crash.log`
- Settings: Flutter shared_preferences (NSUserDefaults on macOS)

## Code Signing

- Identity: `Developer ID Application: Ilya Gulya (T4LV8K9BGV)`
- Bundle ID: `me.gulya.wrenflow`
- Entitlements: audio-input, no sandbox (accessibility + hotkeys need it off)

## Design System

Original Swift app style (WrenflowStyle) ported to Flutter:
- Background: warm off-white `rgb(245,245,245)`
- Surface: `rgb(252,252,252)` with subtle borders `rgba(0,0,0,0.08)`
- Text: dark gray `rgb(38,38,38)`, secondary `rgb(115,115,115)`
- Cards: 12pt radius, soft shadow `black 8% blur 24 offset-y 8`
- Setup wizard: borderless floating card, no window chrome

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
