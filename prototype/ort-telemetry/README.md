# ONNX Runtime telemetry spike (issue #26)

Spike code for the M0 ONNX Runtime telemetry item ([04](../../docs/plan/04-roadmap.md) M0; R-NF-12; O-13; ADR-0005 Consequences). It lives on the branch `prototype/ort-telemetry` and never merges into `main`; only the result is carried into the documents that own the facts (02 §9 row X1, O-13, 09 §3a, 10 §2.5). It gates M2.

## What it is

A standalone crate (its own `[workspace]` and `Cargo.lock`, the dependency set of the link-mode spike: `ort` `=2.0.0-rc.14` with `load-dynamic` and `api-31`, and `sha2`) with two binaries and two scripts:

- `ort-telemetry-probe --variant none|api|env --ort-dir <absolute dir> [--hold <s>] [--linger <s>]` loads the pinned official ONNX Runtime 1.31.0 library with the verified load of the link-mode spike, builds the one environment, runs an `Identity` model in a first session, waits `--hold` seconds (default 10), runs a second session, then stays alive `--linger` seconds (default 120) so that a batched uploader has time to send. It prints its PID, the variant and the value of `ORT_DISABLE_TELEMETRY` it sees.
- `strace-summary <log>…` lists the IPv4 and IPv6 destinations of every `connect`, `sendto`, `sendmsg` and `sendmmsg` call in an `strace -f` log, split into loopback and remote.
- `scripts/measure-linux.sh` runs the probe under `strace` once per variant and twice in a row (the second run shares the first run's `HOME`, so events that the first run cached on disk and uploads at the next start are seen), then lists the files ONNX Runtime left in that `HOME`.
- `scripts/Measure-Windows.ps1` (elevated PowerShell 7.4 or later) runs the probe with an ETW trace session that enables ONNX Runtime's TraceLogging provider and the Kernel-Network, DNS-Client and WinHttp providers, and summarises the events of the probe process per run.

| Variant | `with_telemetry` | `ORT_DISABLE_TELEMETRY` |
|---|---|---|
| `none` | not called (`ort` default: on) | not set |
| `api` | `with_telemetry(false)` | not set |
| `env` | `with_telemetry(false)` | set to `1` by the probe's own code before the first `ort` call (`std::env::set_var`, `unsafe` in edition 2024) |

The issue asks for `env` on Linux only; the scripts run it on Windows too, because it costs one run and shows whether the Windows library reads the variable.

## Facts read at source before measuring (2026-10-14)

Read in the `v1.31.0` tag of `microsoft/onnxruntime` and in the published `ort` 2.0.0-rc.14; the two official libraries were checked by string search (the archives matched their pinned SHA-256 first).

- **The two official libraries carry different telemetry backends.** `libonnxruntime.so.1.31.0` contains the 1DS uploader: the collector URL `https://mobile.events.data.microsoft.com/OneCollector/1.0`, `libcurl`, the `ORT_DISABLE_TELEMETRY` check and the device-ID store (`DeveloperTools`, `onnxruntime.db`). `onnxruntime.dll` contains none of these strings but the TraceLogging provider name `Microsoft.ML.ONNXRuntime`: it is the ETW build. `docs/Privacy.md` of the same tag says the official Windows releases use 1DS; for this DLL the binary says otherwise.
- **The ETW provider** is `Microsoft.ML.ONNXRuntime`, GUID `3a26b1ff-7484-7484-7484-15261f42614d`, registered with `TraceLoggingOptionMicrosoftTelemetry()` (`onnxruntime/core/platform/windows/telemetry.cc`). Its events carry `MICROSOFT_KEYWORD_MEASURES` and a privacy tag; they leave the machine only if Windows' own diagnostic-data pipeline collects them.
- **`with_telemetry(false)` comes too late for the first event.** `EnvironmentBuilder::build` calls `CreateEnv` and only then `DisableTelemetryEvents` (`ort` `src/environment.rs`). `CreateEnv` logs `ProcessInfo` once per process while events are still enabled (`enabled_` starts `true` in both providers; `core/session/environment.cc`). So `api` emits at least `ProcessInfo` on both platforms.
- **On Windows an ETW session can switch the events on again.** The provider's enable callback sets `enabled_` to whatever the enabling session asks for (`ORT_TL_EtwEnableCallback`), so any trace session that enables the provider after `DisableTelemetryEvents` undoes it. The run `3-api-late-trace` measures this.
- **On Linux `ORT_DISABLE_TELEMETRY` is a full opt-out, read once.** The 1DS provider checks it in `Initialize`, when the first environment is created; if set (`1`, `true`, `yes`, `on`, `y`), it creates no uploader, emits no event and writes no device ID, latched for the process lifetime (`core/platform/telemetry_1ds.cc`, `telemetry_environment.h`). Without it, `Initialize` creates the uploader, a persistent device ID (`$XDG_CACHE_HOME/Microsoft/DeveloperTools/.onnxruntime/deviceid`, sent as a SHA-256 hash) and an offline event cache (`onnxruntime.db` in the same directory), whatever the API setting. Events are batched with the `BEST_EFFORT` transmit profile (uploads every 18 to 72 s by event priority, per the 1DS SDK's `TransmitProfiles.cpp`), and at exit pending events are written to the cache, not sent.
- **1DS suppresses itself in CI.** If any of 13 CI variables is set to a truthy value (among them `CI`, `GITHUB_ACTIONS`, `TF_BUILD`, `GITLAB_CI`, `JENKINS_URL`), or `ORT_RUNNING_UNIT_TESTS` is, the Linux library sends nothing (`IsRunningInCI`). A measurement or smoke test in GitHub Actions is therefore falsely clean unless the app process runs without these variables; `measure-linux.sh` starts the probe with `env -i`. The Windows ETW provider ignores the CI variables and checks only `ORT_RUNNING_UNIT_TESTS`.
- On this development machine no ONNX Runtime device-ID key (`HKCU\SOFTWARE\Microsoft\DeveloperTools\.onnxruntime`) existed before or after a local `env` run, consistent with the ETW build.

Expected outcome, to be confirmed or refuted by the measurement: on Linux `none` and `api` contact the 1DS collector and write the device ID and cache, `env` contacts nothing and writes nothing; on Windows the probe process opens no connection in any variant, `none` emits ETW events for environment and session creation, `api` emits at least `ProcessInfo`, and `api` with a late trace session emits session events again.

## Commands

```powershell
cargo test --locked --manifest-path prototype/ort-telemetry/Cargo.toml
cargo build --release --locked --manifest-path prototype/ort-telemetry/Cargo.toml
```

The workflow `.github/workflows/ort-telemetry-spike.yml` runs on a push to this branch, on `ubuntu-latest` and `windows-latest` (both runners are elevated): it fetches the official archive, checks it against the pinned SHA-256, runs every test, builds the probe and runs the measurement script of its OS; the summary lands in the job summary. On Linux it installs `strace` from the Ubuntu archive if the image lacks it; that unpinned install is acceptable for a spike that never reaches `main`.

### The owner's Windows run

In an **elevated** PowerShell 7.4 or later, from the repository root on this branch, with the pinned `onnxruntime.dll` alone in an empty directory:

```powershell
cargo build --release --locked --manifest-path prototype/ort-telemetry/Cargo.toml
./prototype/ort-telemetry/scripts/Measure-Windows.ps1 -Probe ./prototype/ort-telemetry/target/release/ort-telemetry-probe.exe -OrtDir 'D:\path\to\ort-install' -OutDir "$env:TEMP\ort-telemetry-measurement"
```

It takes about ten minutes (four runs of about 130 to 140 s). `summary.md` in the output directory is the result to hand back; it holds counts, event names and relative paths only. `trace.etl`, `trace.xml`, `ort-events.xml` and `network-events.csv` per run stay local, because the system-wide network trace also records other processes of the machine. Running `none` and `api` lets ONNX Runtime emit its events on that machine, which Windows forwards to Microsoft only if its diagnostic-data settings collect them; that is what the measurement is about.

## Result

Pending: the CI run on both OSes and the owner's Windows run.
