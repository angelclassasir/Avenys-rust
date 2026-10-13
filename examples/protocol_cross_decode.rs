use std::env;
use std::fs;
use std::path::Path;

use mire::protocol::{
    CArtifact, CompileRequest, CompileResponse, CompileStatus, LoadPolicy, PROTOCOL_VERSION,
    SecurityPolicy, SourceEntry, SourceFlags,
};

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

fn exchange_path(directory: &Path, file_name: &str) -> std::path::PathBuf {
    directory.join(file_name)
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = env::args().skip(1);
    let operation = arguments.next().ok_or("expected emit or verify")?;
    let directory = arguments.next().ok_or("expected exchange directory")?;
    let directory = Path::new(&directory);
    fs::create_dir_all(directory)?;

    match operation.as_str() {
        "emit" => {
            fs::write(
                exchange_path(directory, "rust-compile-req.bin"),
                request_fixture().encode_payload()?,
            )?;
            fs::write(
                exchange_path(directory, "rust-compile-resp.bin"),
                response_fixture().encode_payload()?,
            )?;
            println!("Rust encoded CompileReq and CompileResp payloads");
        }
        "verify" => {
            let owl_request = fs::read(exchange_path(directory, "owl-compile-req.bin"))?;
            let owl_response = fs::read(exchange_path(directory, "owl-compile-resp.bin"))?;
            assert_eq!(
                CompileRequest::decode_payload(&owl_request)?,
                request_fixture()
            );
            assert_eq!(
                CompileResponse::decode_payload(&owl_response)?,
                response_fixture()
            );
            println!("Rust decoded Owl CompileReq and CompileResp payloads successfully");
        }
        other => return Err(format!("unknown operation: {other}").into()),
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("protocol cross-decode failed: {error}");
        std::process::exit(1);
    }
}
