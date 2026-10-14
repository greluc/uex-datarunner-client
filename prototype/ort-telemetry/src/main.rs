// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The probe: loads the pinned ONNX Runtime library in one variant, runs two sessions and stays alive for the telemetry uploader.

use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;
use std::time::Duration;

use ort::session::Session;
use ort::value::Tensor;
use tallyline_ort_telemetry_spike::model::identity_model;
use tallyline_ort_telemetry_spike::runtime::{
    DISABLE_TELEMETRY_VARIABLE, disable_telemetry_by_environment, load, pinned_for_target,
};
use tallyline_ort_telemetry_spike::variant::{EnvironmentOptOut, Variant};

const USAGE: &str = "usage: ort-telemetry-probe --variant none|api|env|env-only --ort-dir <absolute directory> [--hold <seconds>] [--linger <seconds>]";
const DEFAULT_HOLD_SECONDS: u64 = 10;
const DEFAULT_LINGER_SECONDS: u64 = 120;

struct Options {
    variant: Variant,
    ort_dir: PathBuf,
    hold: Duration,
    linger: Duration,
}

fn parse_options(mut args: impl Iterator<Item = String>) -> Result<Options, String> {
    let mut variant = None;
    let mut ort_dir = None;
    let mut hold = Duration::from_secs(DEFAULT_HOLD_SECONDS);
    let mut linger = Duration::from_secs(DEFAULT_LINGER_SECONDS);
    while let Some(flag) = args.next() {
        let value = args.next().ok_or_else(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--variant" => variant = Some(value.parse::<Variant>().map_err(|e| e.to_string())?),
            "--ort-dir" => ort_dir = Some(PathBuf::from(value)),
            "--hold" => hold = seconds(&value)?,
            "--linger" => linger = seconds(&value)?,
            other => return Err(format!("unknown argument `{other}`")),
        }
    }
    Ok(Options {
        variant: variant.ok_or("--variant is missing")?,
        ort_dir: ort_dir.ok_or("--ort-dir is missing")?,
        hold,
        linger,
    })
}

fn seconds(value: &str) -> Result<Duration, String> {
    value
        .parse()
        .map(Duration::from_secs)
        .map_err(|_| format!("`{value}` is not a number of seconds"))
}

fn run_identity(
    environment: &ort::environment::Environment,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut session = Session::builder(environment)?.commit_from_memory(&identity_model())?;
    let input = Tensor::from_array(([3_usize], vec![1.5_f32, -2.0, 3.25]))?;
    let outputs = session.run(ort::inputs!["x" => input])?;
    let output = outputs.get("y").ok_or("the model has no output `y`")?;
    let (_, values) = output.try_extract_tensor::<f32>()?;
    if values == [1.5_f32, -2.0, 3.25] {
        Ok(())
    } else {
        Err(format!("unexpected output {values:?}").into())
    }
}

fn probe(options: &Options) -> Result<(), Box<dyn std::error::Error>> {
    println!("pid={}", std::process::id());
    println!("variant={}", options.variant.name());
    println!(
        "{DISABLE_TELEMETRY_VARIABLE}={}",
        std::env::var(DISABLE_TELEMETRY_VARIABLE).unwrap_or_else(|_| "<unset>".to_owned())
    );
    let pinned = pinned_for_target().ok_or("no pinned runtime for this target")?;
    let runtime = load(&options.ort_dir, pinned, options.variant.telemetry_events())?;
    println!("loaded={}", runtime.path.display());
    println!("ort_info={}", ort::info());
    run_identity(&runtime.environment)?;
    println!("session=1 inference=ok");
    println!("hold={}", options.hold.as_secs());
    thread::sleep(options.hold);
    run_identity(&runtime.environment)?;
    println!("session=2 inference=ok");
    println!("linger={}", options.linger.as_secs());
    thread::sleep(options.linger);
    drop(runtime);
    println!("done");
    Ok(())
}

fn main() -> ExitCode {
    let options = match parse_options(std::env::args().skip(1)) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    match options.variant.environment_opt_out() {
        EnvironmentOptOut::Set => unsafe { disable_telemetry_by_environment() },
        EnvironmentOptOut::Unset => {}
    }
    match probe(&options) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("probe failed: {error}");
            ExitCode::FAILURE
        }
    }
}
