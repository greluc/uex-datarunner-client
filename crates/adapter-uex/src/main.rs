// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The UEX API spike: read-only GET calls against the live UEX API, printed as shapes only.
//!
//! Usage: `tallyline-uex-spike [--host space|uk] <command>` with the commands `game-versions`,
//! `status-levels`, `prices <id_terminal>`, `account` and `report-access`. `account` reads the
//! secret key from `TALLYLINE_UEX_SECRET_KEY`; `report-access` also reads the app token from
//! `TALLYLINE_UEX_APP_TOKEN`. Nothing is written to disk, and neither a credential, the personal
//! data of an answer nor the message text of a UEX error is printed.

#![forbid(unsafe_code)]

use std::env;
use std::io::{self, Write};
use std::process::ExitCode;

use tallyline_adapter_uex::api::model::{PriceRow, StatusLevel, TerminalId};
use tallyline_adapter_uex::api::{
    AppToken, ClientIdentity, ReadError, ReaderSettings, SecretKey, UexReader,
};

const SPACE_HOST: &str = "https://api.uexcorp.space/2.0/";
const UK_HOST: &str = "https://api.uexcorp.uk/2.0/";
const SECRET_KEY_VARIABLE: &str = "TALLYLINE_UEX_SECRET_KEY";
const APP_TOKEN_VARIABLE: &str = "TALLYLINE_UEX_APP_TOKEN";
const USAGE: &str = "usage: tallyline-uex-spike [--host space|uk] \
                     game-versions | status-levels | prices <id_terminal> | account | report-access";

enum Command {
    GameVersions,
    StatusLevels,
    Prices(TerminalId),
    Account,
    ReportAccess,
}

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let mut stdout = io::stdout().lock();
    let outcome = match parse(&arguments) {
        Ok((host, command)) => run(host, &command, &mut stdout),
        Err(message) => Err(message),
    };
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            writeln!(io::stderr(), "tallyline-uex-spike: {message}").ok();
            ExitCode::FAILURE
        }
    }
}

fn parse(arguments: &[String]) -> Result<(&'static str, Command), String> {
    let (host, rest) = match arguments {
        [flag, host, rest @ ..] if flag == "--host" => match host.as_str() {
            "space" => (SPACE_HOST, rest),
            "uk" => (UK_HOST, rest),
            _ => return Err(USAGE.to_owned()),
        },
        rest => (SPACE_HOST, rest),
    };
    let command = match rest {
        [name] if name == "game-versions" => Command::GameVersions,
        [name] if name == "status-levels" => Command::StatusLevels,
        [name, terminal] if name == "prices" => {
            Command::Prices(TerminalId(terminal.parse().map_err(|_| USAGE.to_owned())?))
        }
        [name] if name == "account" => Command::Account,
        [name] if name == "report-access" => Command::ReportAccess,
        _ => return Err(USAGE.to_owned()),
    };
    Ok((host, command))
}

fn run(host: &str, command: &Command, out: &mut impl Write) -> Result<(), String> {
    let identity = ClientIdentity::new("tallyline-uex-spike", env!("CARGO_PKG_VERSION"));
    let settings = ReaderSettings::new(host, identity).map_err(|error| format!("{error:?}"))?;
    let reader = UexReader::new(&settings).map_err(|error| format!("{error:?}"))?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    let lines = runtime.block_on(lines(&reader, command))?;
    writeln!(out, "host: {host}").map_err(|error| error.to_string())?;
    for line in lines {
        writeln!(out, "{line}").map_err(|error| error.to_string())?;
    }
    Ok(())
}

async fn lines(reader: &UexReader, command: &Command) -> Result<Vec<String>, String> {
    match command {
        Command::GameVersions => {
            let versions = reader.game_versions().await.map_err(describe)?;
            Ok(vec![format!(
                "live: {}, ptu: {}",
                versions.live,
                versions.ptu.as_deref().unwrap_or("none")
            )])
        }
        Command::StatusLevels => {
            let levels = reader.commodity_status_levels().await.map_err(describe)?;
            let mut lines = vec![format!(
                "buy levels: {}, sell levels: {}",
                levels.buy.len(),
                levels.sell.len()
            )];
            lines.extend(levels.buy.iter().map(|level| status_line("buy", level)));
            lines.extend(levels.sell.iter().map(|level| status_line("sell", level)));
            Ok(lines)
        }
        Command::Prices(terminal) => {
            let rows = reader.price_rows(*terminal).await.map_err(describe)?;
            let mut lines = vec![format!("rows: {}", rows.len())];
            lines.extend(rows.iter().map(price_line));
            Ok(lines)
        }
        Command::Account => {
            let key = secret_key()?;
            let account = reader.account(&key).await.map_err(describe)?;
            Ok(vec![format!(
                "account: datarunner {}, banned {}, staff {} (username withheld)",
                account.is_datarunner, account.is_datarunner_banned, account.is_staff
            )])
        }
        Command::ReportAccess => {
            let key = secret_key()?;
            let token = AppToken::new(&variable(APP_TOKEN_VARIABLE)?);
            let account = reader.account(&key).await.map_err(describe)?;
            let access = reader
                .report_access(&token, &key, &account.username)
                .await
                .map_err(describe)?;
            Ok(vec![format!(
                "data_info with token and key: ok, {} row(s) of the own reports",
                access.rows
            )])
        }
    }
}

fn status_line(side: &str, level: &StatusLevel) -> String {
    format!(
        "{side} {}: {} / {} / {}",
        level.code, level.name, level.name_short, level.name_abbr
    )
}

fn price_line(row: &PriceRow) -> String {
    let sizes: Vec<String> = row
        .container_sizes
        .iter()
        .map(|size| size.scu().to_string())
        .collect();
    format!(
        "terminal {} commodity {}: side {:?}, status {}, container_sizes [{}]",
        row.terminal.0,
        row.commodity.0,
        row.side,
        row.status
            .map_or_else(|| "none".to_owned(), |status| status.to_string()),
        sizes.join(",")
    )
}

fn secret_key() -> Result<SecretKey, String> {
    Ok(SecretKey::new(&variable(SECRET_KEY_VARIABLE)?))
}

fn variable(name: &str) -> Result<String, String> {
    env::var(name).map_err(|_| format!("{name} is not set"))
}

fn describe(error: ReadError) -> String {
    match error {
        ReadError::Rejected {
            http_status, code, ..
        } => format!("UEX refused the request: HTTP {http_status}, code {code}"),
        ReadError::NotJson { .. }
        | ReadError::Redirect { .. }
        | ReadError::Malformed { .. }
        | ReadError::BodyTooLarge { .. }
        | ReadError::Transport(_) => format!("{error:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::{Command, SPACE_HOST, TerminalId, UK_HOST, parse};

    fn arguments(text: &str) -> Vec<String> {
        text.split_whitespace().map(str::to_owned).collect()
    }

    #[test]
    fn the_space_host_is_the_default() {
        assert!(matches!(
            parse(&arguments("game-versions")),
            Ok((SPACE_HOST, Command::GameVersions))
        ));
    }

    #[test]
    fn the_uk_host_is_chosen_by_flag() {
        assert!(matches!(
            parse(&arguments("--host uk prices 89")),
            Ok((UK_HOST, Command::Prices(TerminalId(89))))
        ));
    }

    #[test]
    fn an_unknown_host_command_or_terminal_is_refused() {
        for text in [
            "--host example.com account",
            "data-submit",
            "prices eighty",
            "",
        ] {
            assert!(parse(&arguments(text)).is_err(), "{text}");
        }
    }
}
