# Agent Instructions for `charming-colorprofile`

> [!IMPORTANT]
> **Subsequent Cycle Requirement**: On every development cycle, before doing any work, the agent MUST inspect [`UPSTREAM_MAPPING.md`](file:///Users/jonny/Projects/charming/charming-colorprofile/UPSTREAM_MAPPING.md) to verify that all upstream Go files and examples are accounted for. When adding, modifying, or refactoring files, the agent MUST update [`UPSTREAM_MAPPING.md`](file:///Users/jonny/Projects/charming/charming-colorprofile/UPSTREAM_MAPPING.md) to reflect the current state.
>
> Run `scripts/verify_mapping.sh` to mechanically verify that every file in `upstream-go/` is accounted for in `UPSTREAM_MAPPING.md`.

## Core Rules & Workflow
1. Refer to the workspace-level rule in [`/Users/jonny/Projects/charming/AGENTS.md`](file:///Users/jonny/Projects/charming/AGENTS.md).
2. Maintain 100% rustdoc documentation.
3. Every ported file MUST include the guiding comment header:
   ```rust
   //! Cleanroom Rust port of upstream Go source file: `<upstream-go-filepath>`
   //! Upstream Target Tag / Version: `v0.4.3`
   ```
4. Verify all tests pass with `cargo test --all-targets` before committing.
5. The upstream is `github.com/charmbracelet/colorprofile` at tag `v0.4.3`, checked out in
   `upstream-go/` (gitignored). The `charming-x-ansi` sibling provides the ansi sequences,
   SGR color parsing (`read_style_color`), `DecodeSequence`, `Strip`, and the color
   conversions (`convert_256`/`convert_16`).

## Releases
- GitHub Releases MUST match upstream: upstream colorprofile publishes releases, so this
  repo must too. Push the `v0.4.3` tag to create the release (the publish workflow runs
  tests and creates the GitHub Release automatically).
- The crates.io publish step is tag-gated; dev pushes only run tests.
- Sibling `charming-*` repos referenced via `path` dependencies must be **public** on
  GitHub: the workflow `GITHUB_TOKEN` cannot clone private siblings, so CI fetches them via
  `actions/checkout` at `siblings/<name>` (moved into `../` afterwards).
