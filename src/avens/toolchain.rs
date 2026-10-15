use super::*;

#[allow(dead_code)]
pub(super) fn collect_c_files(
    dir: &std::path::Path,
    files: &mut Vec<String>,
) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_c_files(&path, files)?;
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("c") {
            files.push(path.to_string_lossy().into_owned());
        }
    }
    Ok(())
}

/// Tags every function definition with a size-preference attribute.
///
/// LLVM no longer has `-Os`/`-Oz` pipelines; the equivalent is O2 plus an
/// attribute on each function, so the flag alone was not enough and both
/// documented options failed on every build.
fn apply_size_attribute(ir: &str, attribute: &str) -> String {
    let mut out = String::with_capacity(ir.len() + 64);
    for line in ir.lines() {
        // Insert just before the body, which is the position LLVM reads
        // function attributes from. An existing attribute group (`#0`) stays
        // where it is; this just adds to the list.
        if line.starts_with("define ")
            && let Some(body) = line.rfind('{')
        {
            out.push_str(&line[..body]);
            out.push(' ');
            out.push_str(attribute);
            out.push(' ');
            out.push_str(&line[body..]);
            out.push('\n');
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

pub(super) fn optimize_ir(ir: &str, opt_level: OptLevel, source_filename: &str) -> Result<String> {
    let ir = match opt_level.size_attribute() {
        Some(attribute) => apply_size_attribute(ir, attribute),
        None => ir.to_string(),
    };
    let mut command = Command::new("opt");
    command
        .arg("-S")
        .arg(opt_level.as_opt_flag())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());
    let mut child = command.spawn().map_err(|err| {
        MireError::new(ErrorKind::Runtime {
            span: crate::error::Span::unknown(),
            message: format!("Failed to run opt: {}", err),
        })
        .with_filename(source_filename.to_string())
    })?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(ir.as_bytes()).map_err(|err| {
            MireError::new(ErrorKind::Runtime {
                span: crate::error::Span::unknown(),
                message: format!("Failed to stream IR into opt: {}", err),
            })
            .with_filename(source_filename.to_string())
        })?;
    }
    let output = child.wait_with_output().map_err(|err| {
        MireError::new(ErrorKind::Runtime {
            span: crate::error::Span::unknown(),
            message: format!("Failed to wait for opt: {}", err),
        })
        .with_filename(source_filename.to_string())
    })?;
    if !output.status.success() {
        return Err(MireError::new(ErrorKind::Runtime {
            span: crate::error::Span::unknown(),
            message: format!(
                "opt failed for `{}` with status {}.\nstderr:\n{}",
                source_filename,
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        })
        .with_filename(source_filename.to_string()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub(super) fn compile_binary_from_ir(
    ir: &str,
    c_object_files: &[String],
    binary_path: &Path,
    extern_libs: &[(String, String)],
    opt_level: OptLevel,
    source_filename: &str,
    link_crypto_libs: bool,
) -> Result<()> {
    let c_defs = super::build_support::c_defs();
    if matches!(c_defs.artifact, super::config::LibType::Static) {
        // A static library is an archive of objects, not a linked executable
        // with an `.a` suffix. Compile the generated LLVM IR separately and
        // package it together with the runtime/PAL objects using llvm-ar.
        let ir_object = binary_path.with_extension("mire.ir.o");
        // LLVM owns IR-to-object lowering here. Clang is intentionally not
        // involved in producing the Mire object; PAL/runtime C objects are
        // still compiled separately by the C compiler below.
        let mut compile = Command::new("llc");
        compile
            .arg("-filetype=obj")
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        compile.arg("-o").arg(&ir_object);
        // `as_opt_flag` is the single source of truth: llc rejects `-Os`/`-Oz`
        // outright ("invalid optimization level"), so both sites had to agree.
        compile.arg(opt_level.as_opt_flag());
        if let Some(target) = &c_defs.target {
            compile.arg("--target").arg(target);
        }
        let mut child = compile.spawn().map_err(|err| {
            MireError::new(ErrorKind::Runtime {
                span: crate::error::Span::unknown(),
                message: format!("Failed to run llc for static library: {}", err),
            })
            .with_filename(source_filename.to_string())
        })?;
        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(ir.as_bytes()).map_err(|err| {
                MireError::new(ErrorKind::Runtime {
                    span: crate::error::Span::unknown(),
                    message: format!("Failed to stream IR into llc: {}", err),
                })
                .with_filename(source_filename.to_string())
            })?;
        }
        let status = child.wait().map_err(|err| {
            MireError::new(ErrorKind::Runtime {
                span: crate::error::Span::unknown(),
                message: format!("Failed to wait for llc static library: {}", err),
            })
            .with_filename(source_filename.to_string())
        })?;
        if !status.success() {
            return Err(MireError::new(ErrorKind::Runtime {
                span: crate::error::Span::unknown(),
                message: format!("llc failed while compiling static library ({status})"),
            })
            .with_filename(source_filename.to_string()));
        }

        // The destination may be a stale executable from an older compiler
        // version; llvm-ar otherwise tries to update it as an archive.
        let _ = std::fs::remove_file(binary_path);
        let mut archive = Command::new("llvm-ar");
        archive.arg("rcs").arg(binary_path).arg(&ir_object);
        for object in c_object_files {
            archive.arg(object);
        }
        let status = archive.status().map_err(|err| {
            MireError::new(ErrorKind::Runtime {
                span: crate::error::Span::unknown(),
                message: format!("Failed to run llvm-ar: {}", err),
            })
            .with_filename(source_filename.to_string())
        })?;
        let _ = std::fs::remove_file(&ir_object);
        if !status.success() {
            return Err(MireError::new(ErrorKind::Runtime {
                span: crate::error::Span::unknown(),
                message: format!(
                    "llvm-ar failed while creating {} ({status})",
                    binary_path.display()
                ),
            })
            .with_filename(source_filename.to_string()));
        }
        return Ok(());
    }

    if matches!(c_defs.artifact, super::config::LibType::Shared) {
        // A shared library has no process startup objects. Lower Mire IR with
        // LLVM and link the resulting object with the C PAL/runtime objects.
        let is_none_tier = matches!(c_defs.runtime, super::config::RuntimeTier::None);
        let ir_object = binary_path.with_extension("mire.ir.o");
        lower_ir_to_object(
            ir,
            &ir_object,
            c_defs.target.as_deref(),
            opt_level,
            source_filename,
            true,
        )?;

        let mut linker = Command::new("ld.lld");
        linker.arg("-shared").arg("-o").arg(binary_path);
        let tcc_path = build_support::find_tcc().unwrap_or_else(|| "tcc".to_string());
        add_tcc_library_search_paths(&tcc_path, &mut linker);
        linker.arg(&ir_object);
        for object in c_object_files {
            linker.arg(object);
        }
        if !is_none_tier {
            linker.arg("-lm");
            if link_crypto_libs {
                linker.args(["-lssl", "-lcrypto", "-lsodium"]);
            }
        }
        for (lib_name, lib_path) in extern_libs {
            let native = std::path::Path::new(lib_path);
            if native.exists() {
                linker.arg(native);
                if let Some(parent) = native.parent() {
                    linker.arg(format!("-L{}", parent.display()));
                }
            } else if !lib_path.is_empty() {
                let clean_name = if lib_name.contains('.') {
                    lib_name.rsplit('.').next().unwrap_or(lib_name)
                } else {
                    lib_name
                };
                linker.arg(format!("-l{}", clean_name));
            }
        }
        for lib in &c_defs.libs {
            linker.arg(format!("-l{}", lib.trim_start_matches("lib")));
        }
        let output = linker.output().map_err(|err| {
            MireError::runtime(format!("Failed to run ld.lld: {err}"))
                .with_filename(source_filename.to_string())
        })?;
        let _ = std::fs::remove_file(&ir_object);
        if !output.status.success() {
            return Err(MireError::runtime(format!(
                "ld.lld failed while building shared library:\n{}",
                String::from_utf8_lossy(&output.stderr).trim()
            ))
            .with_filename(source_filename.to_string()));
        }
        return Ok(());
    }

    // Lower Mire IR independently of the native link driver. Tcc is
    // responsible for target-aware CRT/libc/native-library linking.
    let ir_object = binary_path.with_extension("mire.ir.o");
    lower_ir_to_object(
        ir,
        &ir_object,
        c_defs.target.as_deref(),
        opt_level,
        source_filename,
        true,
    )?;

    let tcc_path = build_support::find_tcc().ok_or_else(|| {
        MireError::new(ErrorKind::Runtime {
            span: crate::error::Span::unknown(),
            message: "libtcc (tcc) not found. Install tcc or set the TCC_PATH environment variable. Native linking requires libtcc."
                .to_string(),
        })
    })?;
    let mut linker = Command::new(tcc_path);
    let mut seen_objects = std::collections::HashSet::new();

    // Pass the LLVM object and native objects to the target-aware linker.
    linker.arg(&ir_object);
    seen_objects.insert(ir_object.to_string_lossy().into_owned());
    for obj in c_object_files {
        if seen_objects.insert(obj.clone()) {
            linker.arg(obj);
        }
    }
    linker.stdout(Stdio::piped()).stderr(Stdio::piped());

    linker.arg("-o").arg(binary_path);
    linker.arg(opt_level.as_opt_flag());

    let is_none_tier = matches!(c_defs.runtime, super::config::RuntimeTier::None);
    let is_library = matches!(
        c_defs.artifact,
        super::config::LibType::Static | super::config::LibType::Shared
    );
    let is_shared = matches!(c_defs.artifact, super::config::LibType::Shared);

    // For shared libraries, pass -shared and -fPIC to tcc
    if is_shared {
        linker.arg("-shared");
        linker.arg("-fPIC");
    }

    // In freestanding mode (none tier), skip runtime libraries — the program
    // provides its own implementations.  In full/minimal tiers, link the
    // standard runtime dependencies.  Crypto libs (libssl/libcrypto/libsodium)
    // are only linked when PAL files are actually compiled.
    if !is_none_tier {
        linker.arg("-lm");
        if link_crypto_libs {
            linker.arg("-lssl");
            linker.arg("-lcrypto");
            linker.arg("-lsodium");
        }
    }

    // Skip C runtime startup files for libraries (shared/static) and when
    // explicitly requested via nostartfiles.  Libraries don't need a main entry point.
    let needs_nostartfiles = c_defs.nostartfiles || (is_library && !is_shared);
    if needs_nostartfiles {
        // Allow nostartfiles for libraries regardless of tier; for binaries it requires none tier.
        if c_defs.nostartfiles && !is_none_tier && !is_library {
            return Err(MireError::new(ErrorKind::Runtime {
                span: crate::error::Span::unknown(),
                message: "`nostartfiles = true` requires `runtime = \"none\"` or library type (static/shared)".to_string(),
            }).with_filename(source_filename.to_string()));
        }
        linker.arg("-nostartfiles");
    }

    // True no-libc: skip standard library (libc, libgcc, etc.)
    if c_defs.nostdlib {
        if !is_none_tier {
            return Err(MireError::new(ErrorKind::Runtime {
                span: crate::error::Span::unknown(),
                message: "`nostdlib = true` requires `runtime = \"none\"`".to_string(),
            })
            .with_filename(source_filename.to_string()));
        }
        if !c_defs.nostartfiles {
            return Err(MireError::new(ErrorKind::Runtime {
                span: crate::error::Span::unknown(),
                message: "`nostdlib = true` requires `nostartfiles = true`".to_string(),
            })
            .with_filename(source_filename.to_string()));
        }
        linker.arg("-nostdlib");
    }

    // Pass user-defined cflags
    for flag in &c_defs.cflags {
        linker.arg(flag);
    }

    // Add -ffreestanding for none tier or when nostartfiles is set
    if is_none_tier || c_defs.nostartfiles {
        linker.arg("-ffreestanding");
    }

    linker.arg("-pthread");

    // `-x ir -` applies to subsequent inputs in clang. Reset the language
    // before passing native .so/.a files, otherwise clang parses archives as
    // LLVM text and reports a misleading IR syntax error.
    linker.arg("-x").arg("none");
    for (lib_name, lib_path) in extern_libs {
        let clean_name = if lib_name.contains('.') {
            lib_name.rsplit('.').next().unwrap_or(lib_name)
        } else {
            lib_name
        };
        let explicit_native =
            lib_path.ends_with(".so") || lib_path.ends_with(".dylib") || lib_path.ends_with(".a");
        if explicit_native {
            let native_path = std::path::Path::new(&lib_path);
            if let Some(parent) = native_path.parent() {
                linker.arg(format!("-L{}", parent.display()));
                if lib_path.ends_with(".so") || lib_path.ends_with(".dylib") {
                    // Keep explicit fixture/shared-library dependencies
                    // runnable without LD_LIBRARY_PATH. Relative rpaths are
                    // intentional for project-local test libraries.
                    linker.arg(format!("-Wl,-rpath,{}", parent.display()));
                }
            }
            if let Some(file_name) = native_path.file_name().and_then(|name| name.to_str()) {
                // Exact filenames may intentionally omit the conventional
                // `lib` prefix (for example `kioto.so` in test fixtures).
                linker.arg(format!("-l:{file_name}"));
            }
        } else if lib_path.as_str() != clean_name && !lib_path.is_empty() {
            linker.arg(format!("-l:{}", lib_path));
        }
        if !explicit_native {
            linker.arg("-l");
            linker.arg(clean_name);
        }
    }

    for lib in super::build_support::c_defs().libs.iter() {
        let clean_name = if lib.contains('.') {
            lib.rsplit('.').next().unwrap_or(lib)
        } else {
            lib
        };
        linker.arg("-l");
        linker.arg(clean_name);
    }

    let child = linker.spawn().map_err(|err| {
        let _ = std::fs::remove_file(&ir_object);
        MireError::new(ErrorKind::Runtime {
            span: crate::error::Span::new(1, 1),
            message: format!("Failed to run tcc: {}", err),
        })
        .with_filename(source_filename.to_string())
    })?;
    let output = child.wait_with_output().map_err(|err| {
        let _ = std::fs::remove_file(&ir_object);
        MireError::new(ErrorKind::Runtime {
            span: crate::error::Span::new(1, 1),
            message: format!("Failed to wait for tcc: {}", err),
        })
        .with_filename(source_filename.to_string())
    })?;
    let _ = std::fs::remove_file(&ir_object);
    if !output.status.success() {
        return Err(MireError::new(ErrorKind::Runtime {
            span: crate::error::Span::new(1, 1),
            message: format!(
                "tcc failed building `{}` (source: {}) with status {}.\nstdout:\n{}\nstderr:\n{}",
                binary_path.display(),
                source_filename,
                output.status,
                String::from_utf8_lossy(&output.stdout).trim(),
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        })
        .with_filename(source_filename.to_string()));
    }

    Ok(())
}

fn lower_ir_to_object(
    ir: &str,
    output: &Path,
    target: Option<&str>,
    opt_level: OptLevel,
    source_filename: &str,
    position_independent: bool,
) -> Result<()> {
    let mut command = Command::new("llc");
    command
        .arg("-filetype=obj")
        .arg("-o")
        .arg(output)
        .arg(opt_level.as_opt_flag())
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    if position_independent {
        command.arg("-relocation-model=pic");
    }
    if let Some(target) = target {
        command.arg("-mtriple").arg(target);
    }
    let mut child = command.spawn().map_err(|err| {
        MireError::runtime(format!("Failed to run llc: {err}"))
            .with_filename(source_filename.to_string())
    })?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(ir.as_bytes()).map_err(|err| {
            MireError::runtime(format!("Failed to stream LLVM IR into llc: {err}"))
                .with_filename(source_filename.to_string())
        })?;
    }
    drop(child.stdin.take());
    let output_result = child.wait_with_output().map_err(|err| {
        MireError::runtime(format!("Failed to wait for llc: {err}"))
            .with_filename(source_filename.to_string())
    })?;
    if !output_result.status.success() {
        return Err(MireError::runtime(format!(
            "llc failed for {}:\n{}",
            source_filename,
            String::from_utf8_lossy(&output_result.stderr).trim()
        ))
        .with_filename(source_filename.to_string()));
    }
    Ok(())
}

/// Adds the host toolchain's library directories when `ld.lld` is used
/// directly. Tcc normally supplies these implicitly; bypassing Tcc means
/// that common libraries such as sqlite3 would otherwise be invisible to the
/// linker even though they are installed and available to normal C builds.
fn add_tcc_library_search_paths(tcc_path: &str, linker: &mut Command) {
    let output = match Command::new(tcc_path).arg("-print-search-dirs").output() {
        Ok(output) if output.status.success() => output,
        _ => return,
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let mut in_libs = false;
    for line in text.lines() {
        if line.starts_with("libraries:") {
            in_libs = true;
            continue;
        }
        if in_libs {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("libtcc1:") || trimmed.starts_with("crt:") {
                in_libs = false;
                continue;
            }
            if !trimmed.is_empty() {
                linker.arg(format!("-L{trimmed}"));
            }
        }
    }
}

#[allow(dead_code)]
pub(super) fn llvm_version() -> Result<String> {
    let output = Command::new("llvm-config")
        .arg("--version")
        .output()
        .map_err(|err| {
            MireError::new(ErrorKind::Runtime {
                span: crate::error::Span::unknown(),
                message: format!("Failed to run llvm-config: {}", err),
            })
        })?;
    if !output.status.success() {
        return Err(MireError::new(ErrorKind::Runtime {
            span: crate::error::Span::unknown(),
            message: "llvm-config --version failed".to_string(),
        }));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
