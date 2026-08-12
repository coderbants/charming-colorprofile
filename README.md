# charming-colorprofile

Cleanroom Rust port of [Charmbracelet's colorprofile](https://github.com/charmbracelet/colorprofile)
at **v0.4.3** — automatic downsampling of ANSI colors based on output, environment
variables, and Terminfo databases.

Ported by hand from the upstream Go source (checked out in `upstream-go/`, gitignored);
see `UPSTREAM_MAPPING.md` for the full accounting. Verified byte-for-byte against the Go
library via the workspace parity harness (`/Users/jonny/Projects/charming/tools/go-probes/`).


## Installation

```sh
cargo add charming-colorprofile
```
