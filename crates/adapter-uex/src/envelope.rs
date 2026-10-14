// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The UEX response envelope `{"status", "http_code", "data", "message"}` (R-API-5).

use serde_json::Value;

use crate::api::ReadError;

const OK_STATUS: &str = "ok";

/// Returns the `data` of an `ok` envelope, or the [`ReadError`] the answer stands for.
pub(crate) fn data(http_status: u16, body: &[u8]) -> Result<Value, ReadError> {
    let Ok(answer) = serde_json::from_slice::<Value>(body) else {
        return Err(ReadError::NotJson { http_status });
    };
    let Value::Object(mut envelope) = answer else {
        return Err(ReadError::Malformed {
            detail: "the answer is not a JSON object".to_owned(),
        });
    };
    let Some(Value::String(status)) = envelope.remove("status") else {
        return Err(ReadError::Malformed {
            detail: "the envelope has no status string".to_owned(),
        });
    };
    if status != OK_STATUS {
        let message = match envelope.remove("message") {
            Some(Value::String(message)) => message,
            Some(_) | None => String::new(),
        };
        return Err(ReadError::Rejected {
            http_status,
            code: status,
            message,
        });
    }
    envelope.remove("data").ok_or_else(|| ReadError::Malformed {
        detail: "the ok envelope has no data".to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::data;
    use crate::api::ReadError;

    #[test]
    fn a_json_value_that_is_not_an_object_is_malformed() {
        assert!(matches!(
            data(200, b"[1, 2]"),
            Err(ReadError::Malformed { .. })
        ));
    }

    #[test]
    fn an_object_without_a_status_is_malformed() {
        assert!(matches!(
            data(200, br#"{"data": 1}"#),
            Err(ReadError::Malformed { .. })
        ));
    }

    #[test]
    fn an_ok_envelope_without_data_is_malformed() {
        assert!(matches!(
            data(200, br#"{"status": "ok"}"#),
            Err(ReadError::Malformed { .. })
        ));
    }

    #[test]
    fn an_error_envelope_without_a_message_has_an_empty_message() {
        assert_eq!(
            data(500, br#"{"status": "error", "http_code": 500}"#),
            Err(ReadError::Rejected {
                http_status: 500,
                code: "error".to_owned(),
                message: String::new(),
            })
        );
    }

    #[test]
    fn an_ok_envelope_yields_its_data() {
        assert_eq!(
            data(200, br#"{"status": "ok", "data": {"live": "4.10.1"}}"#),
            Ok(json!({"live": "4.10.1"}))
        );
    }
}
