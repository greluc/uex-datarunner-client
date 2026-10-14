# ONNX Runtime telemetry spike (issue #26)

Spike code for the M0 ONNX Runtime telemetry item ([04](../../docs/plan/04-roadmap.md) M0; R-NF-12; O-13; ADR-0005 Consequences). It lives on the branch `prototype/ort-telemetry` and never merges into `main`; only the result is carried into the documents that own the facts (02 §9 row X1, O-13, 09 §3a, 10 §2.5). It gates M2.

## What it is

A standalone crate (its own `[workspace]` and `Cargo.lock`, the dependency set of the link-mode spike: `ort` `=2.0.0-rc.14` with `load-dynamic` and `api-31`, and `sha2`) with two binaries and two scripts:

- `ort-telemetry-probe --variant none|api|env|env-only --ort-dir <absolute dir> [--hold <s>] [--linger <s>]` loads the pinned official ONNX Runtime 1.31.0 library with the verified load of the link-mode spike, builds the one environment, runs an `Identity` model in a first session, waits `--hold` seconds (default 10), runs a second session, then stays alive `--linger` seconds (default 120) so that a batched uploader has time to send. It prints its PID, the variant and the value of `ORT_DISABLE_TELEMETRY` it sees.
- `strace-summary <log>…` lists the IPv4 and IPv6 destinations of every `connect`, `sendto`, `sendmsg` and `sendmmsg` call in an `strace -f` log, split into loopback and remote.
- `scripts/measure-linux.sh` runs the probe under `strace` once per variant and twice in a row (the second run shares the first run's `HOME`, so events that the first run cached on disk and uploads at the next start are seen), then lists the files ONNX Runtime left in that `HOME`.
- `scripts/Measure-Windows.ps1` (elevated PowerShell 7.4 or later) runs the probe with an ETW trace session that enables ONNX Runtime's TraceLogging provider and the Kernel-Network, DNS-Client and WinHttp providers, and summarises the events of the probe process per run. It also records whether the `Diagtrack-Listener` session (Windows' own diagnostic-data collector) is running and names the provider's GUID; a provider group subscription would not show by GUID, so a "no" there is not proof that Windows ignores the events. tracerpt is assumed to put a TraceLogging event's name into `RenderingInfo/Task`; if it does not, the summary shows `EventID 0` and the names have to be read from `ort-events.xml`.

| Variant | `with_telemetry` | `ORT_DISABLE_TELEMETRY` |
|---|---|---|
| `none` | not called (`ort` default: on) | not set |
| `api` | `with_telemetry(false)` | not set |
| `env` | `with_telemetry(false)` | set to `1` by the probe's own code before the first `ort` call (`std::env::set_var`, `unsafe` in edition 2024) |
| `env-only` | not called | set as in `env` |

`env` is the mitigation R-NF-12 prescribes; `env-only` separates the effect of the variable from that of the API. The issue asks for the variable on Linux only; the scripts run both on Windows too, because they cost two runs and show whether the Windows library reads the variable. `runtime.rs` and `model.rs` repeat the verified load and the `Identity` model of the link-mode spike on purpose: each spike branch stands alone.

## Facts read at source before measuring (2026-10-14)

Read in the `v1.31.0` tag of `microsoft/onnxruntime` and in the published `ort` 2.0.0-rc.14; the two official libraries were checked by string search (the archives matched their pinned SHA-256 first).

- **The two official libraries carry different telemetry backends.** `libonnxruntime.so.1.31.0` contains the 1DS uploader: the collector URL `https://mobile.events.data.microsoft.com/OneCollector/1.0`, `libcurl`, the `ORT_DISABLE_TELEMETRY` check and the device-ID store (`DeveloperTools`, `onnxruntime.db`). `onnxruntime.dll` contains none of these strings but the TraceLogging provider name `Microsoft.ML.ONNXRuntime`: it is the ETW build. `docs/Privacy.md` of the same tag says the official Windows releases use 1DS; for this DLL the binary says otherwise.
- **The ETW provider** is `Microsoft.ML.ONNXRuntime`, GUID `3a26b1ff-7484-7484-7484-15261f42614d`, registered with `TraceLoggingOptionMicrosoftTelemetry()` (`onnxruntime/core/platform/windows/telemetry.cc`). Its events carry `MICROSOFT_KEYWORD_MEASURES` and a privacy tag; they leave the machine only if Windows' own diagnostic-data pipeline collects them.
- **`with_telemetry(false)` comes too late for the first event.** `EnvironmentBuilder::build` calls `CreateEnv` and only then `DisableTelemetryEvents` (`ort` `src/environment.rs`). `CreateEnv` logs `ProcessInfo` once per process while events are still enabled (`enabled_` starts `true` in both providers; `core/session/environment.cc`). So `api` emits at least `ProcessInfo` on both platforms.
- **On Windows an ETW session can switch the events on again.** The provider's enable callback sets `enabled_` to whatever the enabling session asks for (`ORT_TL_EtwEnableCallback`), so any trace session that enables the provider after `DisableTelemetryEvents` undoes it. The run `3-api-late-trace` measures this.
- **On Linux `ORT_DISABLE_TELEMETRY` is a full opt-out, read once.** The 1DS provider checks it in `Initialize`, when the first environment is created; if set (`1`, `true`, `yes`, `on`, `y`), it creates no uploader, emits no event and writes no device ID, latched for the process lifetime (`core/platform/telemetry_1ds.cc`, `telemetry_environment.h`). Without it, `Initialize` creates the uploader, a persistent device ID (`$XDG_CACHE_HOME/Microsoft/DeveloperTools/.onnxruntime/deviceid`, sent as a SHA-256 hash) and an offline event cache (`onnxruntime.db` in the same directory), whatever the API setting. Events are batched with the `BEST_EFFORT` transmit profile (uploads every 18 to 72 s by event priority, per the 1DS SDK's `TransmitProfiles.cpp`), and at exit pending events are written to the cache, not sent.
- **1DS suppresses itself in CI.** If any of 13 CI variables is set to a truthy value (among them `CI`, `GITHUB_ACTIONS`, `TF_BUILD`, `GITLAB_CI`, `JENKINS_URL`), or `ORT_RUNNING_UNIT_TESTS` is, the Linux library sends nothing (`IsRunningInCI`). A measurement or smoke test in GitHub Actions is therefore falsely clean unless the app process runs without these variables; `measure-linux.sh` starts the probe with `env -i`. The Windows ETW provider ignores the CI variables and checks only `ORT_RUNNING_UNIT_TESTS`.
- On this development machine no ONNX Runtime device-ID key (`HKCU\SOFTWARE\Microsoft\DeveloperTools\.onnxruntime`) existed before or after a local `env` run, consistent with the ETW build.

Expected outcome, to be confirmed or refuted by the measurement: on Linux `none` and `api` contact the 1DS collector and write the device ID and cache, `env` and `env-only` contact nothing and write nothing; on Windows the probe process opens no connection in any variant, `none` emits ETW events for environment and session creation, `env` and `env-only` behave like `api` and `none`, `api` emits at least `ProcessInfo`, and `api` with a late trace session emits session events again.

## Commands

```powershell
cargo test --locked --manifest-path prototype/ort-telemetry/Cargo.toml
cargo build --release --locked --manifest-path prototype/ort-telemetry/Cargo.toml
```

The workflow `.github/workflows/ort-telemetry-spike.yml` runs on a push to this branch, on `ubuntu-latest` and `windows-latest` (both runners are elevated): it fetches the official archive, checks it against the pinned SHA-256, runs every test, builds the probe and runs the measurement script of its OS; the summary lands in the job summary. The raw logs stay on the runner and are deleted with it: the job summary is the CI evidence, because no artefact upload action is pinned in this repository. On Linux it installs `strace` from the Ubuntu archive if the image lacks it; that unpinned install is acceptable for a spike that never reaches `main`.

### The owner's Windows run

In an **elevated** PowerShell 7.4 or later, from the repository root on this branch, with the pinned `onnxruntime.dll` alone in an empty directory:

```powershell
cargo build --release --locked --manifest-path prototype/ort-telemetry/Cargo.toml
./prototype/ort-telemetry/scripts/Measure-Windows.ps1 -Probe ./prototype/ort-telemetry/target/release/ort-telemetry-probe.exe -OrtDir 'D:\path\to\ort-install' -OutDir "$env:TEMP\ort-telemetry-measurement"
```

It takes about twelve minutes (five runs of about 130 to 140 s). `summary.md` in the output directory is the result to hand back; it holds counts, event names and relative paths only. `trace.etl`, `trace.xml`, `ort-events.xml` and `network-events.csv` per run stay local, because the system-wide network trace also records other processes of the machine. Running `none` and `api` lets ONNX Runtime emit its events on that machine, which Windows forwards to Microsoft only if its diagnostic-data settings collect them; that is what the measurement is about.

## Result (2026-10-14)

Measured in CI run 38591511090 (commit `000cf80`, `ubuntu-latest` and `windows-latest`) and on the owner's Windows 11 machine (build 26300, elevated, commit `e3ca4c5`). The probe ran both sessions in every variant on both OSes; the library was the pinned 1.31.0 file on both.

### Linux: `ORT_DISABLE_TELEMETRY=1` removes the telemetry, the API alone does not

| Variant | Remote connections per run (both runs) | Files left in the isolated `HOME` |
|---|---|---|
| `env` | none | none |
| `env-only` | none | none |
| `api` | one HTTPS connection to a Microsoft address (`4.150.223.102`, `4.150.223.113`), after a DNS query naming `microsoft` | `.cache/Microsoft/DeveloperTools/.onnxruntime/deviceid` (36 bytes) and `onnxruntime.db` (24,576 bytes) |
| `none` | one HTTPS connection to a Microsoft address (`40.79.141.156`, `20.184.175.12`), after a DNS query naming `microsoft` | the same two files |

The variable alone is enough (`env-only` is as clean as `env`), and `with_telemetry(false)` alone still uploads and writes the device ID. The probe ran with `env -i`, so no CI variable suppressed the uploader; the `cargo test` runs of the same job, with `GITHUB_ACTIONS` set, are no measurement.

### Windows: no connection from the process, but ETW events remain with every mitigation

The probe process opened no connection in any variant on either machine (no Kernel-Network, DNS-Client or WinHttp event names its PID) and wrote nothing to `LOCALAPPDATA`, `APPDATA` or the device-ID key. Its ETW events, from the owner's machine (CI had the same counts, without `DriverInfo`):

| Variant | Events of `Microsoft.ML.ONNXRuntime` from the probe |
|---|---|
| `env` | 4: `ProcessInfo`, `DriverInfo`, `RegisterEpLibraryStart`, `RegisterEpLibraryEnd` |
| `api` | the same 4 |
| `api`, trace session started after the first session | 12: the full session set again (`SessionCreationStart`, `SessionCreation`, `SessionCreationEnd`, `ModelLoadStart`/`End`, `EvaluationStart`/`Stop`, `EpAutoSelection`, `EpDeviceUsage`, `ProviderOptions`, `RuntimePerf`) |
| `env-only` | 28, the same as `none` |
| `none` | 28: the 4 initialisation events and two sets of session, model-load, evaluation and execution-provider events |

- `with_telemetry(false)` suppresses the session events but not the 4 events of environment creation, and any ETW session that enables the provider afterwards switches all events on again, as read at source.
- `ORT_DISABLE_TELEMETRY` has no effect on the Windows library (`env-only` equals `none`).
- **On the owner's machine the events are collected:** the `DiagTrack` service runs, the diagnostic-data level is 1 (Required, `AllowTelemetry` under `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\DataCollection`), and the `Diagtrack-Listener` session enables the ONNX Runtime provider by its GUID. Whether DiagTrack uploads these events at that level is Windows' decision and was not observed; the events reach Windows' local collector on that machine. On the CI runner DiagTrack is disabled.
- Another process on the owner's machine also uses ONNX Runtime (about 256 `EvaluationStart`/`EvaluationStop` events per run from other PIDs); not ours.
- The first version of the script counted Kernel-Network events by the process context in which they were logged; that context is arbitrary, and the event's `PID` field names the owner. Recounted by that field, every reported "network event of the probe" belonged to another process; the script now uses the field.

### Consequence for R-NF-12 and O-13

- Linux: the mitigation works; `ORT_DISABLE_TELEMETRY=1` set by our own code before the first ONNX Runtime call is required, the API setting is not sufficient.
- Windows: telemetry remains with the mitigation (initialisation events always, all events once any session enables the provider), and on a machine with default diagnostic-data settings Windows' collector subscribes to them. Per R-NF-12 that requires the decision on an ONNX Runtime build with `--no_telemetry` before M2; whether such a build removes the TraceLogging provider or only its telemetry option has to be verified at source and by repeating this measurement against it.
- The M2 smoke test of R-NF-12 must start the app without the CI variables, or it measures nothing on Linux.
