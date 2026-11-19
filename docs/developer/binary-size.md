# Binary size

A release ships one real program, `gitpoulpi` (UI, CLI, MCP server), plus `gitpoulpi-updater` and a std-only relay (`gitpoulpi-shim`, about 0.3 MiB) copied as `gitpoulpi-tui` and `gitpoulpi-mcp`. The `gitpoulpi-tui` and `gitpoulpi-mcp` columns below measure those crates' own binaries, which are built only for tests and `cargo run`: read the `gitpoulpi` column for what a release weighs.

Official builds: `scripts/build-opti.sh` / `.ps1` (the `dist` profile: thin LTO, 16 codegen units, stripped, `panic = unwind`). Speed comes first: nothing here trades runtime speed for bytes.

**The current figures are in "Re-measured 2026-10-29" below; every older section of this page is historical and kept for the reasoning, its numbers are out of date.**

Dist sizes on Linux x86_64, 2026-10-23 (rustc 1.98.1, MB = MiB): `gitpoulpi` 62.8, `gitpoulpi-tui` 48.8, `gitpoulpi-mcp` 31.2. They were 34.3 / 27.6 / 14.6 on 2026-10-04: the growth is the history-rewrite engine with its embedded Lua, plus review, board and the new hosting providers. Per-feature deltas are in `docs/user/custom-builds.md` ("What it saves"). The "Everything off, English only" row keeps LFS on: a build with `--no-lfs` does not compile at the moment (`gitpoulpi-cli-backend/src/history/mod.rs` calls `crate::lfs::filters_configured` unconditionally), so "LFS off" is not measured. The 2026-10-04 size pass left the then sizes unchanged to within 0.1 MB (the build was already tight); what it changed is the dependency graph.

## Re-measured 2026-10-29

Linux x86_64, commit `65999ef5c`, rustc as installed that day. One reference `dist` build (`./build-opti.sh`, fat LTO, one codegen unit, stripped: the `dist` profile of that day, now thin LTO with 16 codegen units), then one `custom` build per row (same settings) with `cargo run -p xtask -- build-custom <flags>`, one row at a time with `target/custom` deleted between rows (`CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`). The Full row is the `dist` build. Each build took 8 to 18 minutes (the whole run, 20 builds, 4 h 47 min). Flags per row: `--langs en`, `--no-mcp`, `--no-ai`, `--integrations none`, `--no-ai --integrations none --no-avatar-fetch`, `--no-history-rewrite`, `--no-rewrite-scripts`, `--no-global-search`, `--no-workspaces`, `--no-radar`, `--no-board`, `--no-review`, `--no-mermaid`, `--no-lfs`, `--no-worktree`, `--no-submodules`, `--no-avatar-fetch`, `--no-image-view`; the last two rows are `--langs en --integrations none` plus every `--no-*` feature flag (the first keeping LFS, the second with `--no-lfs`).

Exact sizes, bytes:

| Build | `gitpoulpi` (bytes) | `gitpoulpi-tui` (bytes) | `gitpoulpi-mcp` (bytes) |
|---|---|---|---|
| Full (everything on) | 67,715,920 | 52,908,912 | 33,281,040 |
| English only | 54,492,432 | 42,131,184 | 33,279,568 |
| MCP off | 64,619,088 | 52,047,504 | not built |
| AI off | 65,593,744 | 50,996,144 | 31,954,480 |
| All integrations off | 59,073,456 | 45,833,904 | 28,019,248 |
| AI + integrations + avatar fetch off | 54,781,840 | 41,791,536 | 24,077,168 |
| History workbench off (`--no-history-rewrite`) | 64,571,472 | 51,327,440 | 29,934,192 |
| Rewrite scripts off | 66,160,816 | 51,351,568 | 32,573,424 |
| Global search off | 66,578,896 | 51,897,808 | 33,142,640 |
| Workspaces off | 66,609,776 | 51,956,784 | 33,061,968 |
| Radar off | 66,491,504 | 51,690,416 | 33,282,000 |
| Board off | 66,640,816 | 51,877,392 | 33,243,408 |
| Review off | 66,298,064 | 51,423,984 | 33,282,000 |
| Mermaid off | 66,713,456 | 51,951,600 | 33,179,184 |
| LFS off | 66,590,320 | 51,936,848 | 33,011,920 |
| Worktree off | 66,791,696 | 52,021,328 | 33,246,544 |
| Submodules off | 66,753,456 | 51,990,480 | 33,180,592 |
| Avatar fetch off | 66,795,888 | 51,989,104 | 33,264,784 |
| Image view off | 66,410,128 | 51,597,232 | 33,282,064 |
| Everything off, English only (LFS kept) | 36,866,832 | 30,339,184 | not built |
| **Everything off, English only** | 36,584,624 | 30,217,104 | not built |

Same data in MiB, difference from Full in parentheses:

| Build | `gitpoulpi` (MiB) | `gitpoulpi-tui` (MiB) | `gitpoulpi-mcp` (MiB) |
|---|---|---|---|
| Full (everything on) | 64.6 | 50.5 | 31.7 |
| English only | 52.0 (-12.6) | 40.2 (-10.3) | 31.7 (0.0) |
| MCP off | 61.6 (-3.0) | 49.6 (-0.8) | not built |
| AI off | 62.6 (-2.0) | 48.6 (-1.8) | 30.5 (-1.3) |
| All integrations off | 56.3 (-8.2) | 43.7 (-6.7) | 26.7 (-5.0) |
| AI + integrations + avatar fetch off | 52.2 (-12.3) | 39.9 (-10.6) | 23.0 (-8.8) |
| History workbench off (`--no-history-rewrite`) | 61.6 (-3.0) | 48.9 (-1.5) | 28.5 (-3.2) |
| Rewrite scripts off | 63.1 (-1.5) | 49.0 (-1.5) | 31.1 (-0.7) |
| Global search off | 63.5 (-1.1) | 49.5 (-1.0) | 31.6 (-0.1) |
| Workspaces off | 63.5 (-1.1) | 49.5 (-0.9) | 31.5 (-0.2) |
| Radar off | 63.4 (-1.2) | 49.3 (-1.2) | 31.7 (0.0) |
| Board off | 63.6 (-1.0) | 49.5 (-1.0) | 31.7 (0.0) |
| Review off | 63.2 (-1.4) | 49.0 (-1.4) | 31.7 (0.0) |
| Mermaid off | 63.6 (-1.0) | 49.5 (-0.9) | 31.6 (-0.1) |
| LFS off | 63.5 (-1.1) | 49.5 (-0.9) | 31.5 (-0.3) |
| Worktree off | 63.7 (-0.9) | 49.6 (-0.8) | 31.7 (0.0) |
| Submodules off | 63.7 (-0.9) | 49.6 (-0.9) | 31.6 (-0.1) |
| Avatar fetch off | 63.7 (-0.9) | 49.6 (-0.9) | 31.7 (0.0) |
| Image view off | 63.3 (-1.2) | 49.2 (-1.3) | 31.7 (0.0) |
| Everything off, English only (LFS kept) | 35.2 (-29.4) | 28.9 (-21.5) | not built |
| **Everything off, English only** | **34.9 (-29.7)** | **28.8 (-21.6)** | not built |

- `--no-lfs` compiles again: the 2026-10-23 note above (`crate::lfs::filters_configured` called unconditionally) no longer applies, so the LFS off row and the fully-off row without LFS are measured.
- Every row built with exit status 0. The `gitpoulpi-mcp` differences for radar, board, review and image view are 0.0 (the MCP server does not link those UIs); their saving is in `gitpoulpi` and `gitpoulpi-tui` only.
- Full grew from 62.8 / 48.8 / 31.2 MiB (2026-10-23) to 64.6 / 50.5 / 31.7 MiB.

## What the pass did

- Removed unused deps: `clap` from `gitpoulpi-cli` (it comes through `gitpoulpi-commands`), the direct `hyper` dep of `gitpoulpi-mcp` (`hyper-util` enables the same features), the `tracing-subscriber` workspace entry.
- `git2` with `default-features = false`: the `ssh` and `https` features pulled `openssl-sys` and `libssh2-sys` (a second TLS stack next to rustls, plus two C builds). Fetch, push and clone already go through the system `git` (`Git2Backend` delegates them to its fallback), and git2 is only used for local reads and writes. `cargo tree -i openssl-sys` is now empty. Binary effect: about -15 KB (LTO had already dropped the unused code); build effect: no OpenSSL/libssh2 to compile, which matters on Windows.

## Measured and rejected

- `opt-level = 2` for the whole `dist` profile: -0.4 %, not worth any speed risk.
- `opt-level = "s"` on cold crates (`gitpoulpi-commands`, `gitpoulpi-mcp`, `rmcp`, `clap_builder`, `toml*`, `jpeg-decoder`): `gitpoulpi` -0.8 MB but `gitpoulpi-mcp` +0.9 MB (less inlining), `gitpoulpi-tui` unchanged. Net noise, rejected.
- `reqwest` `http2` (in `gitpoulpi-hosting`): `h2` is about 0.2 MB. Kept: multiplexing for the hosting API calls.
- `syntect` `default-syntaxes` / `default-themes`: kept, every language must highlight.
- Linux-only `-Wl,-z,pack-relative-relocs` (RELR) would cut `.rela.dyn` (3.6 MB) to a few hundred KB, but needs glibc 2.36+ at run time (breaks Ubuntu 22.04). Not enabled.
- `panic = "abort"`: forbidden, daemon threads must survive a panic.

## Where the weight is

`.text` ~21 MB, `.rodata` 5 MB, `.rela.dyn` 3.6 MB, `.data.rel.ro` 3 MB, unwind tables ~2.4 MB. By crate (`gitpoulpi`): `gitpoulpi_tui` 4 MB, `std` 2.5 MB, `gitpoulpi_commands` 1.8 MB, `serde_core` 1.7 MB (monomorphised derives), `gitpoulpi_daemon` 0.7 MB.

## i18n catalog as one blob (2026-10-05)

After the git-vocabulary work (8 variants per message) `gitpoulpi` had grown 34.3 -> 37.8 MiB. Cause: `build.rs` emitted `static PLAIN_n: [&str; 8567]` x 18 languages plus per-message `&[&[Seg]]` tables, about 154k `&str` entries for about 50k distinct strings. In a PIE binary each `&str` in a static is 16 bytes plus a 24-byte `R_X86_64_RELATIVE` entry in `.rela.dyn`, so about 6 MB per binary and relocation work at every start.

Now `messages.rs` holds ONE deduplicated `static BLOB: &str` plus relocation-free integer tables: `STR_OFFS` (string i = `BLOB[offs[i]..offs[i+1]]`), `PLAIN: [[u16; N]; langs]` of string ids, `SEGS`/`ALT_OFFS` (deduplicated argument-template alternatives as `u32` codes) and `ArgMsg { map: [u16; ..] }`. Lookup stays O(1) (two array reads and a safe slice); the public API is unchanged. `tests/lookup_snapshot.rs` hashes all 596,736 (language, vocabulary, variant, key, plural count) lookups and pins the value computed before the change; `blob_tests` checks the offsets sit on char boundaries.

| dist, Linux (bytes) | before | after |
|---|---|---|
| `gitpoulpi` | 39,640,824 | 33,091,576 |
| `gitpoulpi-tui` | 32,546,712 | 26,001,720 |
| `gitpoulpi-mcp` | 15,402,728 | 15,402,728 |

(`gitpoulpi-mcp` links no UI strings catalog pieces that matter here: unchanged.) For `gitpoulpi`: `.rela.dyn` 5.3 MB -> 0.55 MB, `.data.rel.ro` 4.4 MB -> 0.45 MB, `.rodata` 5.7 -> 7.8 MB (the blob), `.text` unchanged. This also makes the RELR idea above mostly moot.

Remaining i18n tables in `.rodata` (unstripped `nm --size-sort`): `ARG_MSGS` 0.9 MB (576 `u16` per argument message, mostly repeated; a per-message dedup could take it to a few hundred KB), `PLAIN` 0.6 MB, `STR_OFFS` 0.27 MB, `SEGS` 0.2 MB. Everything else big is third party (`ecp_nistz256_precomputed` 150 KB, `BIG5_LOW_BITS`, `CRC32_TABLE`). Top `.text` crates unchanged: `gitpoulpi_tui` 4.3 MiB, `std` 2.5, `gitpoulpi_commands` 1.8, `serde_core` 1.7.

## Custom builds, measured (2026-10-06, S10)

Linux x86_64, `--profile custom` (the `dist` settings of that day: fat LTO, one codegen unit, stripped; the profile is now thin LTO with 16 codegen units), 2026-10-06, MiB, difference from the full build in parentheses. One `cargo build` over the three binary crates per row, via `cargo run -p xtask -- build-custom`.

| Build (from `full` unless noted) | `gitpoulpi` | `gitpoulpi-tui` | `gitpoulpi-mcp` |
|---|---|---|---|
| Full (everything on) | 32.0 | 25.2 | 14.8 |
| English only | 26.0 (-5.9) | 20.0 (-5.2) | 14.8 (0.0) |
| MCP off | 29.7 (-2.3) | 24.7 (-0.4) | not built |
| AI off | 30.9 (-1.1) | 24.2 (-1.0) | 14.7 (-0.1; was 17.4, see below) |
| Both integrations off | 29.6 (-2.3) | 23.6 (-1.6) | 13.7 (-1.1; was 16.4, see below) |
| AI + integrations + avatar fetch off | 26.4 (-5.5) | 20.4 (-4.7) | 13.6 (-1.1) |
| LFS off | 31.3 (-0.7) | 24.6 (-0.6) | 14.5 (-0.3) |
| Worktree off | 31.5 (-0.5) | 24.7 (-0.5) | 14.7 (0.0) |
| Submodules off | 31.4 (-0.5) | 24.7 (-0.5) | 14.7 (-0.1) |
| Avatar fetch off | 31.5 (-0.5) | 24.7 (-0.5) | 14.7 (0.0) |
| **Everything off, English only** | **18.7 (-13.3)** | **15.5 (-9.7)** | not built |

- Languages are the biggest single win (about -5.9 MiB), then the HTTP stack: AI + integrations + avatar fetch off together is about as much as English only, and turning all three off *and* English only removes 13 MiB of `gitpoulpi`. All-off lands at 18.7 MiB, the plan's 18-22 MB estimate.
- The `gitpoulpi-mcp` anomaly (AI off 17.4, integrations off 16.4 MiB) is fixed, see below. The all-three-off row (13.6) was measured before the fix and was already unaffected only because it hit the same path less.
- LFS, worktree, submodules and avatar fetch each save 0.5-0.7 MiB, more than the plan guessed (0.1-0.5).

### The MCP anomaly: a catalog message kept the i18n blob alive (2026-10-06)

Root cause, found by diffing the two unstripped binaries: `.text` was the same (11.9 MiB) but `.rodata` grew from 0.8 to 3.7 MiB. `gitpoulpi-mcp` normally links no translated strings (the linker drops the i18n blob), but `gitpoulpi-commands::config_cmd::not_built_note` read `msg::settings::AI_NOT_BUILT` / `INTEGRATIONS_NOT_BUILT` from the catalog. That code is compiled only when `ai` or `hosting` is off, so exactly those builds pulled the whole blob plus its tables into the MCP server. Feature unification was identical apart from `chrono`'s clock. Fix: the note is now an English constant in `config_cmd.rs` (the same text as the English catalog entry; the catalog entries stay for the TUI). Result (stripped `custom`, MiB): AI off 17.4 -> 14.66, integrations off 16.4 -> 13.65 (full 14.8). Rule: do not call `gitpoulpi_i18n::msg` from `gitpoulpi-commands` code the MCP server runs, or the MCP binary grows by about 3 MiB.

## Re-measure

```
cargo install --locked cargo-bloat cargo-machete
cargo shear                     # or: cargo machete
# cargo-bloat needs symbols, so override the profile's strip:
CARGO_PROFILE_DIST_STRIP=false cargo bloat --profile dist --crates -n 40 -p gitpoulpi-cli
cargo tree -d                   # duplicate versions
```

Delete `target/dist` afterwards (or rebuild) so the unstripped binary is not mistaken for the shipped one.

To re-measure a custom build: `cargo run -p xtask -- build-custom --no-mcp ...` prints the sizes (about 4-6 minutes each); delete `target/custom` between runs when disk is tight.

## Image view (`image-view` feature), measured (2026-10-08, S8)

Linux x86_64, `--profile custom` (the `dist` settings), all languages and integrations on, built twice with `cargo run -p xtask -- build-custom` (the second with `--no-image-view`), bytes:

| Binary | with `image-view` | without | difference |
|---|---|---|---|
| `gitpoulpi` | 34,537,400 | 34,095,032 | +442,368 (0.42 MiB) |
| `gitpoulpi-tui` | 27,200,216 | 26,762,872 | +437,344 (0.42 MiB) |
| `gitpoulpi-mcp` | 15,968,168 | 15,968,936 | 0 (the MCP server links no viewer) |

Inside the plan's 0.3 to 0.5 MiB budget: the GIF, WebP, BMP and ICO decoders (`gif`, `image-webp`, hand-rolled BMP/ICO), the viewport, raster, half-block and worker code, and the Edit in runner. PNG and JPEG decoding stays in both (avatars use it). Note that the whole workspace has grown since the 2026-10-06 table (full `gitpoulpi` was 32.0 MiB), so the absolute sizes above are not comparable with it.

## Provider features (`azure-devops`, `bitbucket`, `jira`, `trello`), measured (2026-10-08, S13)

Linux x86_64, `--profile custom`, all languages on, every other feature on, built with `cargo run -p xtask -- build-custom --langs all --integrations <list>`; bytes, difference from the full build in parentheses. Each row switches off one provider; the last keeps only GitHub and GitLab. (The HTTP stack, `reqwest` and `rustls`, is shared with GitHub and GitLab, so it is not counted here.)

| Build | `gitpoulpi` | `gitpoulpi-tui` | `gitpoulpi-mcp` |
|---|---|---|---|
| Full (all six integrations) | 35,506,744 | 28,066,456 | 16,748,456 |
| without `azure-devops` | 35,342,104 (-164,640) | 27,906,264 (-160,192) | 16,587,240 (-161,216) |
| without `bitbucket` (Cloud and Data Center) | 35,205,656 (-301,088) | 27,773,784 (-292,672) | 16,452,712 (-295,744) |
| without `jira` | 35,401,848 (-104,896) | 27,970,008 (-96,448) | 16,647,592 (-100,864) |
| without `trello` | 35,431,160 (-75,584) | 27,999,288 (-67,168) | 16,672,904 (-75,552) |
| GitHub and GitLab only | 34,835,000 (-671,744) | 27,399,480 (-666,976) | 16,077,640 (-670,816) |

Together the four providers cost about 0.64 MiB of `gitpoulpi` (0.64 MiB of `gitpoulpi-tui`, 0.64 MiB of `gitpoulpi-mcp`): Bitbucket the most (about 0.29 MiB: two clients, the avatar source and SSH keys), then Azure DevOps (0.16), Jira (0.10) and Trello (0.07). The four single-provider differences sum to 646,208 bytes against 671,744 for all four off, the rest being the shared spec, auth and reference code that only disappears when none is built. The full `gitpoulpi` is 33.9 MiB (32.9 MiB in the image-view table above, before the providers).

## Embedded Lua for history rewrite scripts (2026-10-22, S10)

`mlua` 0.11 with `lua54` + `vendored` (+ `serialize`) adds the Lua 5.4.8 interpreter and its glue to every binary that links `gitpoulpi-cli-backend` with `rewrite-scripts` (the default). Measured: the Lua C sources compile to about 230 KiB of `.text` and 5 KiB of data at `-O2` (`gcc -O2` over `lua-src`'s files, without `lua.c` / `luac.c`); the mlua glue and the script runtime add an estimated 100-150 KiB, so expect about +0.35 MiB on `gitpoulpi`, `gitpoulpi-tui` and `gitpoulpi-mcp` (not measured end to end: a `dist` build per binary was out of the disk budget of the slice). Turn it off with `--no-default-features` plus the other features, or leave `history-rewrite` out; the feature-matrix run "history-rewrite, scripts off" keeps that build compiling. Build requirement: a C compiler (`cc` crate) at build time, already needed for libgit2. Third-party notice: `packaging/notices/bundled.txt` (Lua, MIT).

## Terminal run mode for local agents (2026-10-30)

The `terminal-agent` feature of `gitpoulpi-ai` (turned on by the TUI's and the commands crate's `ai` feature) links `gitpoulpi-pty`: `portable-pty` (Unix pty / Windows ConPTY) and `vt100` (screen emulation), both already built for `xtask`'s test driver, plus the client and the extraction code. Not measured end to end (a `dist` build per binary was out of the disk budget of the slice); expect on the order of 0.2 to 0.3 MiB on `gitpoulpi` and `gitpoulpi-tui` (the pty and terminal-emulation crates and the client), nothing on `gitpoulpi-mcp` unless its `ai` feature is on (then it links the same client). A custom build without `ai` does not compile any of it; one with `ai` but without the terminal mode is not offered (the mode is part of `ai`), and `gitpoulpi-ai` built with `--no-default-features` alone keeps compiling without `gitpoulpi-pty` (a profile set to the terminal mode then fails with a clear error).
