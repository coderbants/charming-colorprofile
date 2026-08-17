# Upstream Go File Mapping: `rusty-colorprofile`

Target Upstream Tag: `github.com/charmbracelet/colorprofile@v0.4.3`

This mapping accounts for **every** file in the upstream repository at this pin. The full
repo is checked out locally in `upstream-go/` (gitignored). Both rusty-lipgloss v2.0.5
and rusty-bubbletea v2.0.8 require colorprofile v0.4.3, so a single pin is needed
(no diff-forward required).

## Source Files

| Upstream Go File | Rust Equivalent / Status | Notes / Description |
| :--- | :--- | :--- |
| `doc.go` | `src/lib.rs` | Package docs; `Profile` enum + `Convert` (cache omitted: perf-only, like the upstream `sync.RWMutex` map) |
| `profile.go` | `src/lib.rs` | `Profile` constants, `String()`, `Convert` (ANSI256/ANSI downsampling via `rusty-x-ansi` `convert_256`/`convert_16`/`ansi256_to_16`) |
| `env.go` | `src/env.rs` — **Ported** | `Detect` (tty-ness passed as a parameter instead of a `term.File` writer), `Env`, `colorProfile`, `envColorProfile`, `envNoColor`/`cliColor`/`cliColorForced` (Go `strconv.ParseBool` semantics), `Terminfo` (DB loader deferred: returns the ANSI baseline, matching Go's nil-db behavior), `Tmux` (runs `tmux info`), `environ` helpers |
| `env_other.go` | `src/env.rs` | Non-Windows `windowsColorProfile` stub (returns None) |
| `env_windows.go` | `src/env.rs` — Deferred | Windows Console API profile detection (ConEmuANSI/ANSICON/NT build numbers) |
| `writer.go` | `src/writer.rs` — **Ported** | `Writer` (generic over `W: Write`): TrueColor passthrough, NoTTY strip, downsample via `DecodeSequence` + `ReadStyleColor`; `handleSgr` mirrors `foregroundColorString`/`backgroundColorString`/`underlineColorString` (Basic -> `3n`/`9n`/`4n`/`10n`, Indexed -> `38;5;n`/`48;5;n`, RGB -> `38;2;r;g;b`/`48;2;r;g;b`) |

## Test Files

| Upstream Go Test File | Rust Equivalent / Status | Notes / Description |
| :--- | :--- | :--- |
| `profile_test.go` | `tests/profile_test.rs` + `src/lib.rs` (tests) | `Convert`/`Convert256` vectors |
| `env_test.go` | `tests/env_test.rs` + `src/env.rs` (tests) | Detect/Env/terminfo/tmux vectors (TERM matrix, NO_COLOR, CLICOLOR, non-TTY) |
| `writer_test.go` | `tests/writer_test.rs` + `src/writer.rs` (tests) | Writer golden vectors (TrueColor/ANSI256/ANSI/ASCII), middleware, benchmarks |

## Examples

| Upstream Go Example | Rust Equivalent / Status | Notes / Description |
| :--- | :--- | :--- |
| `examples/colors/main.go` | `examples/colors.rs` — Pending | Prints a color grid for the detected profile |
| `examples/profile/main.go` | `examples/profile.rs` — Pending | Prints the detected profile |
| `examples/writer/writer.go` | `examples/writer.rs` — Pending | Writer demo feeding text through the profile writer |

## Verification

- `cargo test --all-targets` (all green), `cargo build` zero warnings.
- Writer outputs cross-checked against the real Go library via the workspace parity
  harness (`/Users/jonny/Projects/rusty/tools/go-probes/colorprofile/`).
- `scripts/verify_mapping.sh` verifies every upstream `*.go` file is accounted for here.
