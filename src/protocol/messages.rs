//! Version 1 message payloads for the Owl/Avenys protocol.
//!
//! These are protobuf payloads without transport framing. Encoders emit fields
//! in ascending field-number order and omit scalar defaults. Repeated fields
//! preserve caller order.

use super::{
    decode_varint, read_length_delimited, write_bytes_field, write_string_field,
    write_varint_field, ProtocolError,
};

pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFlags {
    Entry,
    MacroFile,
    PackageRoot,
}

impl SourceFlags {
    pub const fn bit(self) -> u32 {
        match self {
            Self::Entry => 1,
            Self::MacroFile => 2,
            Self::PackageRoot => 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourceEntry {
    pub id: String,
    pub path: String,
    pub sha256: String,
    pub size: u64,
    pub flags: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CArtifact {
    pub path: String,
    pub sha256: String,
    pub kind: String,
    pub symbol: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SecurityPolicy {
    pub mode: String,
    pub externs: Vec<String>,
    pub extern_libs: Vec<String>,
    pub macros: Vec<String>,
    pub allowed_builtins: Vec<String>,
    pub allow_unsafe: bool,
    pub allow_asm: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LoadPolicy {
    pub project_root: String,
    pub descend_only: bool,
    pub reject_symlink_escape: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompileRequest {
    pub protocol_version: u32,
    pub request_id: String,
    pub entry: String,
    pub load_policy: LoadPolicy,
    pub manifest: Vec<SourceEntry>,
    pub c_artifacts: Vec<CArtifact>,
    pub security: SecurityPolicy,
    pub flags: Vec<String>,
    pub artifact: String,
    pub target: String,
    pub cache_dir: String,
    pub output_dir: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileStatus {
    Done,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileResponse {
    pub request_id: String,
    pub status: CompileStatus,
    pub output_path: String,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FieldValue<'a> {
    Varint(u64),
    Bytes(&'a [u8]),
}

fn parse_fields(input: &[u8]) -> Result<Vec<(u32, FieldValue<'_>)>, ProtocolError> {
    let mut fields = Vec::new();
    let mut offset = 0;
    while offset < input.len() {
        let tag = decode_varint(input, &mut offset)?;
        let number =
            u32::try_from(tag >> 3).map_err(|_| ProtocolError::InvalidFieldNumber(u32::MAX))?;
        if number == 0 || number > 0x1fff_ffff {
            return Err(ProtocolError::InvalidFieldNumber(number));
        }
        let wire_type = (tag & 7) as u8;
        let value = match wire_type {
            0 => FieldValue::Varint(decode_varint(input, &mut offset)?),
            2 => FieldValue::Bytes(read_length_delimited(input, &mut offset)?),
            other => return Err(ProtocolError::InvalidWireType(other)),
        };
        fields.push((number, value));
    }
    Ok(fields)
}

fn expect_varint(value: &FieldValue<'_>) -> Result<u64, ProtocolError> {
    match value {
        FieldValue::Varint(value) => Ok(*value),
        FieldValue::Bytes(_) => Err(ProtocolError::InvalidWireType(2)),
    }
}

fn expect_bytes<'a>(value: &FieldValue<'a>) -> Result<&'a [u8], ProtocolError> {
    match value {
        FieldValue::Bytes(value) => Ok(value),
        FieldValue::Varint(_) => Err(ProtocolError::InvalidWireType(0)),
    }
}

fn decode_string(value: &FieldValue<'_>) -> Result<String, ProtocolError> {
    String::from_utf8(expect_bytes(value)?.to_vec()).map_err(|_| ProtocolError::InvalidUtf8)
}

fn encode_nested(
    field_number: u32,
    nested: Vec<u8>,
    output: &mut Vec<u8>,
) -> Result<(), ProtocolError> {
    write_bytes_field(field_number, &nested, output)
}

fn push_repeated_strings(
    field_number: u32,
    values: &[String],
    output: &mut Vec<u8>,
) -> Result<(), ProtocolError> {
    for value in values {
        write_string_field(field_number, value, output)?;
    }
    Ok(())
}

impl SourceEntry {
    pub fn encode_payload(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut output = Vec::new();
        if !self.id.is_empty() {
            write_string_field(1, &self.id, &mut output)?;
        }
        if !self.path.is_empty() {
            write_string_field(2, &self.path, &mut output)?;
        }
        if !self.sha256.is_empty() {
            write_string_field(3, &self.sha256, &mut output)?;
        }
        if self.size != 0 {
            write_varint_field(4, self.size, &mut output)?;
        }
        if self.flags != 0 {
            write_varint_field(5, u64::from(self.flags), &mut output)?;
        }
        Ok(output)
    }

    pub fn decode_payload(input: &[u8]) -> Result<Self, ProtocolError> {
        let mut result = Self::default();
        for (number, value) in parse_fields(input)? {
            match number {
                1 => result.id = decode_string(&value)?,
                2 => result.path = decode_string(&value)?,
                3 => result.sha256 = decode_string(&value)?,
                4 => result.size = expect_varint(&value)?,
                5 => {
                    result.flags = u32::try_from(expect_varint(&value)?)
                        .map_err(|_| ProtocolError::IntegerOverflow)?
                }
                _ => {}
            }
        }
        Ok(result)
    }
}

impl CArtifact {
    pub fn encode_payload(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut output = Vec::new();
        if !self.path.is_empty() {
            write_string_field(1, &self.path, &mut output)?;
        }
        if !self.sha256.is_empty() {
            write_string_field(2, &self.sha256, &mut output)?;
        }
        if !self.kind.is_empty() {
            write_string_field(3, &self.kind, &mut output)?;
        }
        if !self.symbol.is_empty() {
            write_string_field(4, &self.symbol, &mut output)?;
        }
        Ok(output)
    }

    pub fn decode_payload(input: &[u8]) -> Result<Self, ProtocolError> {
        let mut result = Self::default();
        for (number, value) in parse_fields(input)? {
            match number {
                1 => result.path = decode_string(&value)?,
                2 => result.sha256 = decode_string(&value)?,
                3 => result.kind = decode_string(&value)?,
                4 => result.symbol = decode_string(&value)?,
                _ => {}
            }
        }
        Ok(result)
    }
}

impl LoadPolicy {
    pub fn encode_payload(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut output = Vec::new();
        if !self.project_root.is_empty() {
            write_string_field(1, &self.project_root, &mut output)?;
        }
        if self.descend_only {
            write_varint_field(2, 1, &mut output)?;
        }
        if self.reject_symlink_escape {
            write_varint_field(3, 1, &mut output)?;
        }
        Ok(output)
    }

    pub fn decode_payload(input: &[u8]) -> Result<Self, ProtocolError> {
        let mut result = Self::default();
        for (number, value) in parse_fields(input)? {
            match number {
                1 => result.project_root = decode_string(&value)?,
                2 => result.descend_only = expect_varint(&value)? != 0,
                3 => result.reject_symlink_escape = expect_varint(&value)? != 0,
                _ => {}
            }
        }
        Ok(result)
    }
}

impl SecurityPolicy {
    pub fn encode_payload(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut output = Vec::new();
        if !self.mode.is_empty() {
            write_string_field(1, &self.mode, &mut output)?;
        }
        push_repeated_strings(2, &self.externs, &mut output)?;
        push_repeated_strings(3, &self.extern_libs, &mut output)?;
        push_repeated_strings(4, &self.macros, &mut output)?;
        push_repeated_strings(5, &self.allowed_builtins, &mut output)?;
        if self.allow_unsafe {
            write_varint_field(6, 1, &mut output)?;
        }
        if self.allow_asm {
            write_varint_field(7, 1, &mut output)?;
        }
        Ok(output)
    }

    pub fn decode_payload(input: &[u8]) -> Result<Self, ProtocolError> {
        let mut result = Self::default();
        for (number, value) in parse_fields(input)? {
            match number {
                1 => result.mode = decode_string(&value)?,
                2 => result.externs.push(decode_string(&value)?),
                3 => result.extern_libs.push(decode_string(&value)?),
                4 => result.macros.push(decode_string(&value)?),
                5 => result.allowed_builtins.push(decode_string(&value)?),
                6 => result.allow_unsafe = expect_varint(&value)? != 0,
                7 => result.allow_asm = expect_varint(&value)? != 0,
                _ => {}
            }
        }
        Ok(result)
    }
}

impl CompileRequest {
    pub fn encode_payload(&self) -> Result<Vec<u8>, ProtocolError> {
        if self.protocol_version != PROTOCOL_VERSION {
            return Err(ProtocolError::InvalidProtocolVersion(self.protocol_version));
        }
        let mut output = Vec::new();
        write_varint_field(1, u64::from(self.protocol_version), &mut output)?;
        if !self.request_id.is_empty() {
            write_string_field(2, &self.request_id, &mut output)?;
        }
        if !self.entry.is_empty() {
            write_string_field(3, &self.entry, &mut output)?;
        }
        encode_nested(4, self.load_policy.encode_payload()?, &mut output)?;
        for entry in &self.manifest {
            encode_nested(5, entry.encode_payload()?, &mut output)?;
        }
        for artifact in &self.c_artifacts {
            encode_nested(6, artifact.encode_payload()?, &mut output)?;
        }
        encode_nested(7, self.security.encode_payload()?, &mut output)?;
        push_repeated_strings(8, &self.flags, &mut output)?;
        if !self.artifact.is_empty() {
            write_string_field(9, &self.artifact, &mut output)?;
        }
        if !self.target.is_empty() {
            write_string_field(10, &self.target, &mut output)?;
        }
        if !self.cache_dir.is_empty() {
            write_string_field(11, &self.cache_dir, &mut output)?;
        }
        if !self.output_dir.is_empty() {
            write_string_field(12, &self.output_dir, &mut output)?;
        }
        Ok(output)
    }

    pub fn decode_payload(input: &[u8]) -> Result<Self, ProtocolError> {
        let mut result = Self::default();
        for (number, value) in parse_fields(input)? {
            match number {
                1 => {
                    result.protocol_version = u32::try_from(expect_varint(&value)?)
                        .map_err(|_| ProtocolError::IntegerOverflow)?
                }
                2 => result.request_id = decode_string(&value)?,
                3 => result.entry = decode_string(&value)?,
                4 => result.load_policy = LoadPolicy::decode_payload(expect_bytes(&value)?)?,
                5 => result
                    .manifest
                    .push(SourceEntry::decode_payload(expect_bytes(&value)?)?),
                6 => result
                    .c_artifacts
                    .push(CArtifact::decode_payload(expect_bytes(&value)?)?),
                7 => result.security = SecurityPolicy::decode_payload(expect_bytes(&value)?)?,
                8 => result.flags.push(decode_string(&value)?),
                9 => result.artifact = decode_string(&value)?,
                10 => result.target = decode_string(&value)?,
                11 => result.cache_dir = decode_string(&value)?,
                12 => result.output_dir = decode_string(&value)?,
                _ => {}
            }
        }
        if result.protocol_version != PROTOCOL_VERSION {
            return Err(ProtocolError::InvalidProtocolVersion(
                result.protocol_version,
            ));
        }
        Ok(result)
    }
}

impl CompileResponse {
    pub fn encode_payload(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut output = Vec::new();
        if !self.request_id.is_empty() {
            write_string_field(1, &self.request_id, &mut output)?;
        }
        let status = match self.status {
            CompileStatus::Done => 1,
            CompileStatus::Error => 2,
        };
        write_varint_field(2, status, &mut output)?;
        if !self.output_path.is_empty() {
            write_string_field(3, &self.output_path, &mut output)?;
        }
        push_repeated_strings(4, &self.errors, &mut output)?;
        push_repeated_strings(5, &self.warnings, &mut output)?;
        Ok(output)
    }

    pub fn decode_payload(input: &[u8]) -> Result<Self, ProtocolError> {
        let mut request_id = String::new();
        let mut status = None;
        let mut output_path = String::new();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        for (number, value) in parse_fields(input)? {
            match number {
                1 => request_id = decode_string(&value)?,
                2 => {
                    status = Some(match expect_varint(&value)? {
                        1 => CompileStatus::Done,
                        2 => CompileStatus::Error,
                        value => return Err(ProtocolError::InvalidMessageStatus(value)),
                    })
                }
                3 => output_path = decode_string(&value)?,
                4 => errors.push(decode_string(&value)?),
                5 => warnings.push(decode_string(&value)?),
                _ => {}
            }
        }
        Ok(Self {
            request_id,
            status: status.ok_or(ProtocolError::MissingMessageStatus)?,
            output_path,
            errors,
            warnings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request_fixture() -> CompileRequest {
        CompileRequest {
            protocol_version: PROTOCOL_VERSION,
            request_id: "r1".to_string(),
            entry: "src/main.mire".to_string(),
            load_policy: LoadPolicy {
                project_root: "/work/p".to_string(),
                descend_only: true,
                reject_symlink_escape: true,
            },
            manifest: vec![SourceEntry {
                id: "main".to_string(),
                path: "/work/p/main.mire".to_string(),
                sha256: "abcd".to_string(),
                size: 3,
                flags: SourceFlags::Entry.bit(),
            }],
            c_artifacts: vec![CArtifact {
                path: "lib.o".to_string(),
                sha256: "beef".to_string(),
                kind: "object".to_string(),
                symbol: "entry".to_string(),
            }],
            security: SecurityPolicy {
                mode: "strict".to_string(),
                externs: vec!["rt_*".to_string()],
                extern_libs: vec!["c".to_string()],
                macros: vec!["test".to_string()],
                allowed_builtins: vec!["abs".to_string()],
                allow_unsafe: true,
                allow_asm: false,
            },
            flags: vec!["-O3".to_string(), "--release".to_string()],
            artifact: "bin".to_string(),
            target: "x86_64-unknown-linux-gnu".to_string(),
            cache_dir: "cache".to_string(),
            output_dir: "out".to_string(),
        }
    }

    fn response_fixture() -> CompileResponse {
        CompileResponse {
            request_id: "r1".to_string(),
            status: CompileStatus::Done,
            output_path: "out/main".to_string(),
            errors: Vec::new(),
            warnings: vec!["unused value".to_string()],
        }
    }

    fn encode_hex(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    #[test]
    fn version_one_message_goldens_are_canonical_and_round_trip() {
        let request_golden = include_str!("golden/v1_compile_req.hex").trim();
        let request_payload = request_fixture().encode_payload().unwrap();
        assert_eq!(encode_hex(&request_payload), request_golden);
        assert_eq!(
            CompileRequest::decode_payload(&request_payload).unwrap(),
            request_fixture()
        );

        let response_golden = include_str!("golden/v1_compile_resp.hex").trim();
        let response_payload = response_fixture().encode_payload().unwrap();
        assert_eq!(encode_hex(&response_payload), response_golden);
        assert_eq!(
            CompileResponse::decode_payload(&response_payload).unwrap(),
            response_fixture()
        );

        let default_load_policy_golden = include_str!("golden/v1_default_load_policy.hex").trim();
        assert_eq!(
            encode_hex(&LoadPolicy::default().encode_payload().unwrap()),
            default_load_policy_golden
        );
        assert_eq!(
            LoadPolicy::decode_payload(&[]).unwrap(),
            LoadPolicy::default()
        );
    }

    #[test]
    fn protocol_codec_rejects_unknown_versions_and_invalid_statuses() {
        let mut request = request_fixture();
        request.protocol_version = PROTOCOL_VERSION + 1;
        assert_eq!(
            request.encode_payload(),
            Err(ProtocolError::InvalidProtocolVersion(PROTOCOL_VERSION + 1))
        );
        assert_eq!(
            CompileResponse::decode_payload(&[0x10, 0x7f]),
            Err(ProtocolError::InvalidMessageStatus(127))
        );
    }

    #[test]
    fn codec_ignores_unknown_supported_wire_fields() {
        let mut payload = response_fixture().encode_payload().unwrap();
        super::super::write_varint_field(99, 7, &mut payload).unwrap();
        assert_eq!(
            CompileResponse::decode_payload(&payload).unwrap(),
            response_fixture()
        );
    }
}
