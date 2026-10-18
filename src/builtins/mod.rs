use std::collections::HashMap;

use crate::parser::ast::DataType;

pub fn default_builtin_returns() -> HashMap<String, DataType> {
    let mut builtins = HashMap::new();

    for name in [
        "dasu",
        "push",
        "append",
        "remove",
        "time_sleep_ms",
        "time_sleep_ns",
        "fs_write",
        "fs_append",
        "fs_copy",
        "fs_move",
        "fs_drop",
        "fs_mkdir",
        "fs_rmdir",
        "env_set",
        "env_chdir",
        "proc_kill",
        "proc_write",
        "proc_on",
        "proc.on",
        "proc_exit",
    ] {
        builtins.insert(name.to_string(), DataType::None);
    }

    for name in [
        "len",
        "time.now.ms",
        "time.now.ns",
        "time_since_ms",
        "time_since_ns",
        "time_mark",
        "time_elapsed_ms",
        "time_elapsed_ns",
        "time.mark",
        "time.elapsed_ns",
        "mem.used",
        "mem_total",
        "mem_free",
        "mem_available",
        "mem_process",
        "mem.process",
        "cpu_time_ns",
        "cpu_time_ms",
        "cpu_mark",
        "cpu_elapsed_ns",
        "cpu.count",
        "cpu_cycles_est",
        "cpu.cycles_est",
        "cpu.mark",
        "sum",
        "min",
        "max",
        "abs",
        "round",
        "floor",
        "ceil",
        "clamp",
        "fs_size",
        "proc_wait",
        "lists.len",
        "lists.get",
        "strings.len",
    ] {
        builtins.insert(name.to_string(), DataType::I64);
    }

    for name in ["lists.push", "lists.set", "lists.slice"] {
        builtins.insert(
            name.to_string(),
            DataType::Vector {
                element_type: Box::new(DataType::Anything),
                dynamic: true,
            },
        );
    }

    // Vector with capacity
    {
        let name = "vec.with.capacity.i64";
        builtins.insert(
            name.to_string(),
            DataType::Vector {
                element_type: Box::new(DataType::I64),
                dynamic: true,
            },
        );
    }

    for name in ["lists.fold", "lists.map", "lists.filter"] {
        builtins.insert(name.to_string(), DataType::Unknown);
    }

    for name in [
        "strings.replace",
        "strings.join",
        "strings.to_upper",
        "strings.to_lower",
        "strings.trim",
        "strings.concat",
        "strings.to_string",
        "strings.replace_first",
        "mem.format",
        "gpu.snapshot",
        "time.elapsed_ms",
        "cpu.elapsed_ms",
        "cpu_elapsed_ms",
    ] {
        builtins.insert(name.to_string(), DataType::Str);
    }

    builtins.insert(
        "strings.split".to_string(),
        DataType::Vector {
            element_type: Box::new(DataType::Str),
            dynamic: true,
        },
    );

    // `env.get` used to be registered here with nothing behind it: there is no
    // `getenv` in the runtime and no `env` module in the standard library, so
    // type checking accepted the call and no phase ever lowered it. The result
    // was a call to an undefined `@env.get` and an llc failure naming neither
    // the missing builtin nor the reason. Registering a name is a promise that
    // codegen keeps; the rest of the `env_*` entries below are the real,
    // working spellings.
    for name in [
        "ireru",
        "__mire_fmt",
        "mem_format_bytes",
        "fs.read",
        "fs_join",
        "fs_dir",
        "fs_name",
        "fs_ext",
        "env_cwd",
        "proc.run",
        "proc_exec",
        "proc_shell",
        "proc_exec_pipe",
        "proc_pipe",
        "proc_read",
        "strings.to_upper",
        "strings.to_lower",
        "strings.trim",
        "strings.concat",
    ] {
        builtins.insert(name.to_string(), DataType::Str);
    }

    for name in [
        "fs_exists",
        "fs_is_dir",
        "fs_is_file",
        "proc_exists",
        "gpu_available",
        "strings.starts_with",
        "strings.ends_with",
    ] {
        builtins.insert(name.to_string(), DataType::Bool);
    }

    for name in ["lists.keys", "lists.values", "lists.slice", "range"] {
        builtins.insert(
            name.to_string(),
            DataType::Vector {
                element_type: Box::new(DataType::Anything),
                dynamic: true,
            },
        );
    }

    builtins.insert(
        "fs_list".to_string(),
        DataType::Vector {
            element_type: Box::new(DataType::Str),
            dynamic: true,
        },
    );

    builtins.insert(
        "env_args".to_string(),
        DataType::Vector {
            element_type: Box::new(DataType::Str),
            dynamic: true,
        },
    );

    for name in [
        "env_all",
        "mem_snapshot",
        "mem.snapshot",
        "cpu_loadavg",
        "cpu_snapshot",
        "cpu.snapshot",
        "gpu_snapshot",
        "dicts.set",
        "dicts.keys",
        "dicts.values",
        "dicts.to_string",
    ] {
        builtins.insert(
            name.to_string(),
            DataType::Map {
                key_type: Box::new(DataType::Anything),
                value_type: Box::new(DataType::Anything),
            },
        );
    }
    builtins.insert("dicts.get".to_string(), DataType::Anything);

    for name in [
        "int",
        "float",
        "bool",
        "type",
        "sort",
        "reverse",
        "unique",
        "trim",
        "ltrim",
        "rtrim",
        "pad_left",
        "pad_right",
        "first",
        "last",
        "slice",
        "flatten",
        "is_int",
        "is_float",
        "is_bool",
        "is_str",
        "is_list",
        "is_dict",
        "is_none",
        "contains",
        "index_of",
        "ram_usage",
        "mem_percent",
        "cpu_freq_mhz",
        "proc_spawn",
        "proc_exec_bg",
    ] {
        builtins.insert(name.to_string(), DataType::Anything);
    }

    builtins.insert("substr".to_string(), DataType::Str);
    builtins.insert("concat".to_string(), DataType::Str);

    builtins.insert("str".to_string(), DataType::Str);
    builtins.insert("range".to_string(), DataType::List);
    builtins.insert("call".to_string(), DataType::Unknown);
    builtins.insert("__if_expr".to_string(), DataType::Unknown);
    builtins.insert("__do_while".to_string(), DataType::None);
    builtins.insert("__type_matches".to_string(), DataType::Bool);
    builtins.insert("__is".to_string(), DataType::Bool);
    builtins.insert("thread.spawn".to_string(), DataType::I64);
    builtins.insert("thread.join".to_string(), DataType::I64);
    builtins.insert("new::".to_string(), DataType::Unknown);
    builtins.insert("own::".to_string(), DataType::Box);
    builtins.insert("move::".to_string(), DataType::Unknown);
    builtins.insert("drop::".to_string(), DataType::None);

    // Pointer intrinsics: lowered directly to LLVM GEP/Load/Store instead of
    // runtime calls. rt_ptr_add(p :&i64, delta :i64) :&i64 advances a byte
    // pointer; rt_read_u32(p :&i64) :i64 loads a u32 as i64;
    // rt_write_u32(p :&i64, v :i64) stores an i64 as u32.
    builtins.insert(
        "rt_ptr_add".to_string(),
        DataType::Ref {
            inner: Box::new(DataType::I64),
        },
    );
    builtins.insert("rt_read_u32".to_string(), DataType::I64);
    builtins.insert("rt_write_u32".to_string(), DataType::None);

    builtins
}

/// Builtins whose call path has a compiler lowering or a declared PAL/runtime
/// target. Keep this inventory explicit so registry changes require review.
#[cfg(test)]
const LOWERED_BUILTINS: &[&str] = &[
    "__do_while",
    "__if_expr",
    "__type_matches",
    "abs",
    "call",
    "ceil",
    "contains",
    "cpu.count",
    "dasu",
    "env_args",
    "floor",
    "ireru",
    "len",
    "lists.filter",
    "lists.fold",
    "lists.map",
    "lists.push",
    "mem.process",
    "vec.with.capacity.i64",
    "range",
    "round",
    "rt_ptr_add",
    "rt_read_u32",
    "rt_write_u32",
    "str",
    "time.now.ms",
    "time.now.ns",
];

/// Registered return types with no direct lowering. An explicitly loaded
/// function with one of these names still takes precedence in type checking;
/// this list only prevents the builtin fallback from pretending an
/// implementation exists.
const UNIMPLEMENTED_BUILTINS: &[&str] = &[
    "__is",
    "__mire_fmt",
    "append",
    "bool",
    "clamp",
    "concat",
    "cpu.cycles_est",
    "cpu.elapsed_ms",
    "cpu.mark",
    "cpu.snapshot",
    "cpu_cycles_est",
    "cpu_elapsed_ms",
    "cpu_elapsed_ns",
    "cpu_freq_mhz",
    "cpu_loadavg",
    "cpu_mark",
    "cpu_snapshot",
    "cpu_time_ms",
    "cpu_time_ns",
    "dicts.get",
    "dicts.keys",
    "dicts.set",
    "dicts.to_string",
    "dicts.values",
    "env_all",
    "env_chdir",
    "env_cwd",
    "env_set",
    "first",
    "flatten",
    "float",
    "fs.read",
    "fs_append",
    "fs_copy",
    "fs_dir",
    "fs_drop",
    "fs_exists",
    "fs_ext",
    "fs_is_dir",
    "fs_is_file",
    "fs_join",
    "fs_list",
    "fs_mkdir",
    "fs_move",
    "fs_name",
    "fs_rmdir",
    "fs_size",
    "fs_write",
    "gpu.snapshot",
    "gpu_available",
    "gpu_snapshot",
    "index_of",
    "int",
    "is_bool",
    "is_dict",
    "is_float",
    "is_int",
    "is_list",
    "is_none",
    "is_str",
    "last",
    "lists.get",
    "lists.keys",
    "lists.len",
    "lists.set",
    "lists.slice",
    "lists.values",
    "ltrim",
    "max",
    "mem.format",
    "mem.snapshot",
    "mem.used",
    "mem_available",
    "mem_format_bytes",
    "mem_free",
    "mem_percent",
    "mem_process",
    "mem_snapshot",
    "mem_total",
    "min",
    "pad_left",
    "pad_right",
    "proc.on",
    "proc.run",
    "proc_exec",
    "proc_exec_bg",
    "proc_exec_pipe",
    "proc_exists",
    "proc_exit",
    "proc_kill",
    "proc_on",
    "proc_pipe",
    "proc_read",
    "proc_shell",
    "proc_spawn",
    "proc_wait",
    "proc_write",
    "push",
    "ram_usage",
    "remove",
    "reverse",
    "rtrim",
    "slice",
    "sort",
    "strings.concat",
    "strings.ends_with",
    "strings.join",
    "strings.len",
    "strings.replace",
    "strings.replace_first",
    "strings.split",
    "strings.starts_with",
    "strings.to_lower",
    "strings.to_string",
    "strings.to_upper",
    "strings.trim",
    "substr",
    "sum",
    "thread.join",
    "thread.spawn",
    "time.elapsed_ms",
    "time.elapsed_ns",
    "time.mark",
    "time_elapsed_ms",
    "time_elapsed_ns",
    "time_mark",
    "time_since_ms",
    "time_since_ns",
    "time_sleep_ms",
    "time_sleep_ns",
    "trim",
    "type",
    "unique",
];

/// These names are compiler syntax or ownership intrinsics, not callable
/// function symbols, so they do not participate in call lowering.
#[cfg(test)]
const LANGUAGE_INTRINSICS: &[&str] = &["drop::", "move::", "new::", "own::"];

pub fn is_unimplemented_builtin(name: &str) -> bool {
    UNIMPLEMENTED_BUILTINS.contains(&name)
}

#[cfg(test)]
mod tests {
    use super::{
        LANGUAGE_INTRINSICS, LOWERED_BUILTINS, UNIMPLEMENTED_BUILTINS, default_builtin_returns,
        is_unimplemented_builtin,
    };
    use crate::{compiler::typeck::check_program_types, parse};
    use std::collections::HashSet;

    #[test]
    fn every_registered_builtin_has_an_explicit_lowering_disposition() {
        let registered = default_builtin_returns();
        let mut classified = HashSet::new();

        for name in LOWERED_BUILTINS
            .iter()
            .chain(UNIMPLEMENTED_BUILTINS)
            .chain(LANGUAGE_INTRINSICS)
        {
            assert!(
                classified.insert(*name),
                "builtin `{name}` is classified twice"
            );
            assert!(registered.contains_key(*name), "`{name}` is not registered");
        }

        let unclassified: Vec<_> = registered
            .keys()
            .filter(|name| !classified.contains(name.as_str()))
            .collect();
        assert!(
            unclassified.is_empty(),
            "registered builtins need an explicit lowering disposition: {unclassified:?}"
        );
    }

    #[test]
    fn unsupported_builtin_fallback_is_explicitly_identified() {
        for name in UNIMPLEMENTED_BUILTINS {
            assert!(is_unimplemented_builtin(name));
        }
        for name in LOWERED_BUILTINS.iter().chain(LANGUAGE_INTRINSICS) {
            assert!(!is_unimplemented_builtin(name));
        }
    }

    #[test]
    fn every_unimplemented_builtin_call_fails_before_mir_lowering() {
        for name in UNIMPLEMENTED_BUILTINS {
            let source = format!("pub fn main: () {{ {name}() }}\n");
            let mut program = match parse(&source) {
                Ok(program) => program,
                Err(_) if *name == "type" => continue,
                Err(error) => panic!("registered call syntax for `{name}` failed: {error}"),
            };

            let error = check_program_types(&mut program, &source).expect_err(&format!(
                "unimplemented builtin `{name}` passed type checking"
            ));
            assert!(
                error.to_string().contains("no compiler lowering"),
                "`{name}` produced an unexpected diagnostic: {error}"
            );
        }
    }
}
