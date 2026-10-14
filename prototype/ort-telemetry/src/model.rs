// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! A hand-encoded ONNX model, so the probe needs no model file.

/// Returns an ONNX model with one `Identity` node from float input `x` of shape `[3]` to output `y`.
#[must_use]
pub fn identity_model() -> Vec<u8> {
    let tensor_type = |name: &str| {
        let dim = field_bytes(1, &varint_field(1, 3));
        let shape = field_bytes(2, &dim);
        let tensor = [varint_field(1, 1), shape].concat();
        let type_proto = field_bytes(1, &tensor);
        [string_field(1, name), field_bytes(2, &type_proto)].concat()
    };
    let node = [
        string_field(1, "x"),
        string_field(2, "y"),
        string_field(3, "identity"),
        string_field(4, "Identity"),
    ]
    .concat();
    let graph = [
        field_bytes(1, &node),
        string_field(2, "spike"),
        field_bytes(11, &tensor_type("x")),
        field_bytes(12, &tensor_type("y")),
    ]
    .concat();
    let opset = [string_field(1, ""), varint_field(2, 13)].concat();
    [
        varint_field(1, 7),
        field_bytes(7, &graph),
        field_bytes(8, &opset),
    ]
    .concat()
}

fn varint(mut value: u64) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let low = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            out.push(low);
            return out;
        }
        out.push(low | 0x80);
    }
}

fn varint_field(number: u64, value: u64) -> Vec<u8> {
    [varint(number << 3), varint(value)].concat()
}

fn field_bytes(number: u64, bytes: &[u8]) -> Vec<u8> {
    [
        varint((number << 3) | 2),
        varint(bytes.len() as u64),
        bytes.to_vec(),
    ]
    .concat()
}

fn string_field(number: u64, value: &str) -> Vec<u8> {
    field_bytes(number, value.as_bytes())
}
