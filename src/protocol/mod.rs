//! The small, dependency-free wire foundation for the Owl/Avenys protocol.
//!
//! Message definitions remain deliberately separate from transport. This
//! module owns only protobuf wire primitives and length-prefixed frames so the
//! same messages can travel over FIFOs during bootstrap and Unix sockets later.

use std::fmt;

/// Hard upper bound for one framed protocol payload.
pub const MAX_FRAME_SIZE: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    UnexpectedEnd,
    VarintTooLong,
    FrameTooLarge(usize),
    InvalidFieldNumber(u32),
    InvalidWireType(u8),
    LengthOverflow,
    InvalidUtf8,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd => write!(formatter, "unexpected end of protocol data"),
            Self::VarintTooLong => write!(formatter, "protocol varint is too long"),
            Self::FrameTooLarge(size) => {
                write!(formatter, "protocol frame is too large: {size} bytes")
            }
            Self::InvalidFieldNumber(number) => {
                write!(formatter, "invalid protobuf field number: {number}")
            }
            Self::InvalidWireType(wire_type) => {
                write!(formatter, "unsupported protobuf wire type: {wire_type}")
            }
            Self::LengthOverflow => write!(formatter, "protocol length does not fit in memory"),
            Self::InvalidUtf8 => write!(formatter, "protocol string is not valid UTF-8"),
        }
    }
}

impl std::error::Error for ProtocolError {}

/// Append an unsigned protobuf varint.
pub fn encode_varint(mut value: u64, output: &mut Vec<u8>) {
    while value >= 0x80 {
        output.push((value as u8 & 0x7f) | 0x80);
        value >>= 7;
    }
    output.push(value as u8);
}

/// Read one unsigned protobuf varint from `input`.
pub fn decode_varint(input: &[u8], offset: &mut usize) -> Result<u64, ProtocolError> {
    let mut value = 0u64;
    for byte_index in 0..10 {
        let byte = *input.get(*offset).ok_or(ProtocolError::UnexpectedEnd)?;
        *offset += 1;

        if byte_index == 9 && byte > 1 {
            return Err(ProtocolError::VarintTooLong);
        }
        value |= u64::from(byte & 0x7f) << (byte_index * 7);
        if byte & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err(ProtocolError::VarintTooLong)
}

/// Prefix a protobuf payload with its varint byte length.
pub fn encode_frame(payload: &[u8]) -> Result<Vec<u8>, ProtocolError> {
    if payload.len() > MAX_FRAME_SIZE {
        return Err(ProtocolError::FrameTooLarge(payload.len()));
    }
    let mut frame = Vec::with_capacity(payload.len() + 10);
    encode_varint(payload.len() as u64, &mut frame);
    frame.extend_from_slice(payload);
    Ok(frame)
}

/// Decode one complete frame. `None` represents the zero-length session close.
/// The returned offset identifies the first byte after the frame, allowing a
/// stream reader to retain trailing bytes for the next message.
pub fn decode_frame(input: &[u8]) -> Result<(Option<&[u8]>, usize), ProtocolError> {
    let mut offset = 0;
    let declared_size = decode_varint(input, &mut offset)?;
    let size = usize::try_from(declared_size).map_err(|_| ProtocolError::LengthOverflow)?;
    if size > MAX_FRAME_SIZE {
        return Err(ProtocolError::FrameTooLarge(size));
    }
    if size == 0 {
        return Ok((None, offset));
    }
    let end = offset
        .checked_add(size)
        .ok_or(ProtocolError::LengthOverflow)?;
    let payload = input.get(offset..end).ok_or(ProtocolError::UnexpectedEnd)?;
    Ok((Some(payload), end))
}

/// Write a protobuf field key for wire type 0 or 2.
pub fn write_field_key(
    field_number: u32,
    wire_type: u8,
    output: &mut Vec<u8>,
) -> Result<(), ProtocolError> {
    if field_number == 0 || field_number > 0x1fffffff {
        return Err(ProtocolError::InvalidFieldNumber(field_number));
    }
    if wire_type != 0 && wire_type != 2 {
        return Err(ProtocolError::InvalidWireType(wire_type));
    }
    encode_varint(
        (u64::from(field_number) << 3) | u64::from(wire_type),
        output,
    );
    Ok(())
}

pub fn write_varint_field(
    field_number: u32,
    value: u64,
    output: &mut Vec<u8>,
) -> Result<(), ProtocolError> {
    write_field_key(field_number, 0, output)?;
    encode_varint(value, output);
    Ok(())
}

pub fn write_bytes_field(
    field_number: u32,
    value: &[u8],
    output: &mut Vec<u8>,
) -> Result<(), ProtocolError> {
    write_field_key(field_number, 2, output)?;
    encode_varint(value.len() as u64, output);
    output.extend_from_slice(value);
    Ok(())
}

pub fn write_string_field(
    field_number: u32,
    value: &str,
    output: &mut Vec<u8>,
) -> Result<(), ProtocolError> {
    write_bytes_field(field_number, value.as_bytes(), output)
}

pub fn read_length_delimited<'a>(
    input: &'a [u8],
    offset: &mut usize,
) -> Result<&'a [u8], ProtocolError> {
    let declared_size = decode_varint(input, offset)?;
    let size = usize::try_from(declared_size).map_err(|_| ProtocolError::LengthOverflow)?;
    let end = offset
        .checked_add(size)
        .ok_or(ProtocolError::LengthOverflow)?;
    let value = input
        .get(*offset..end)
        .ok_or(ProtocolError::UnexpectedEnd)?;
    *offset = end;
    Ok(value)
}

pub fn read_string(input: &[u8], offset: &mut usize) -> Result<String, ProtocolError> {
    let bytes = read_length_delimited(input, offset)?;
    String::from_utf8(bytes.to_vec()).map_err(|_| ProtocolError::InvalidUtf8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varint_golden_values_are_stable() {
        let cases = [
            (0, vec![0x00]),
            (127, vec![0x7f]),
            (128, vec![0x80, 0x01]),
            (16_384, vec![0x80, 0x80, 0x01]),
            (
                u64::MAX,
                vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01],
            ),
        ];
        for (value, expected) in cases {
            let mut encoded = Vec::new();
            encode_varint(value, &mut encoded);
            assert_eq!(encoded, expected);
            let mut offset = 0;
            assert_eq!(decode_varint(&encoded, &mut offset), Ok(value));
            assert_eq!(offset, encoded.len());
        }
    }

    #[test]
    fn frame_golden_has_varint_length_and_payload() {
        assert_eq!(encode_frame(&[0x08, 0x01]).unwrap(), vec![0x02, 0x08, 0x01]);
        assert_eq!(
            decode_frame(&[0x02, 0x08, 0x01]),
            Ok((Some(&[0x08, 0x01][..]), 3))
        );
        assert_eq!(decode_frame(&[0x00]), Ok((None, 1)));
    }

    #[test]
    fn frame_rejects_large_payload_before_allocation() {
        assert_eq!(
            encode_frame(&vec![0; MAX_FRAME_SIZE + 1]),
            Err(ProtocolError::FrameTooLarge(MAX_FRAME_SIZE + 1))
        );
        let mut oversized_length = Vec::new();
        encode_varint((MAX_FRAME_SIZE + 1) as u64, &mut oversized_length);
        assert_eq!(
            decode_frame(&oversized_length),
            Err(ProtocolError::FrameTooLarge(MAX_FRAME_SIZE + 1))
        );
    }

    #[test]
    fn fields_are_encoded_in_explicit_wire_types() {
        let mut output = Vec::new();
        write_varint_field(1, 7, &mut output).unwrap();
        write_string_field(2, "owl", &mut output).unwrap();
        assert_eq!(output, vec![0x08, 0x07, 0x12, 0x03, b'o', b'w', b'l']);
    }

    #[test]
    fn malformed_utf8_is_rejected() {
        let mut offset = 0;
        assert_eq!(
            read_string(&[0x01, 0xff], &mut offset),
            Err(ProtocolError::InvalidUtf8)
        );
    }
}
