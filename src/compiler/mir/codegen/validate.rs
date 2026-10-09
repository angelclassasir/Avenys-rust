use crate::compiler::mir::{MirOp, MirProgram, MirValue};

use super::builtins::{builtin_to_pal, pal_extern_decls};
use std::collections::HashSet;

const SPECIAL_BARE_CALLS: &[&str] = &[
    "str",
    "dasu",
    "print",
    "ireru",
    "env_args",
    "call",
    "__if_expr",
    "__do_while",
    "range",
    "len",
    "contains",
];

fn pal_decl_names() -> HashSet<String> {
    let mut names = HashSet::new();
    for decl in pal_extern_decls() {
        if let Some(rest) = decl.strip_prefix("declare ")
            && let Some(start) = rest.find('@') {
                let after = &rest[start + 1..];
                if let Some(end) = after.find('(') {
                    names.insert(after[..end].trim().to_string());
                }
            }
    }
    names
}

pub(crate) fn find_first_undefined_call(program: &MirProgram) -> Option<(String, (usize, usize))> {
    let defined: HashSet<String> = program.functions.iter().map(|f| f.name.clone()).collect();
    let extern_names: HashSet<String> = program
        .extern_functions
        .iter()
        .map(|e| e.name.clone())
        .collect();
    let struct_names: HashSet<String> = program.struct_types.keys().cloned().collect();
    let pal_decls = pal_decl_names();
    for func in &program.functions {
        for block in &func.blocks {
            for inst in &block.insts {
                let name = match &inst.op {
                    MirOp::Call(callee, _, _) => match callee {
                        MirValue::Global(n) => Some(n.as_str()),
                        MirValue::FunctionRef { name, .. } => Some(name.as_str()),
                        _ => None,
                    },
                    _ => None,
                };
                if let Some(n) = name {
                    if n.starts_with("rt_")
                        || n.starts_with("pal_")
                        || n.starts_with("fn_")
                        || n.starts_with("alloca_")
                    {
                        continue;
                    }
                    if SPECIAL_BARE_CALLS.contains(&n) {
                        continue;
                    }
                    if builtin_to_pal(n).is_some() {
                        continue;
                    }
                    if defined.contains(n)
                        || extern_names.contains(n)
                        || pal_decls.contains(n)
                        || struct_names.contains(n)
                    {
                        continue;
                    }
                    return Some((n.to_string(), inst.loc));
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::find_first_undefined_call;
    use crate::compiler::mir::{
        DataType, MirBlock, MirFunction, MirInst, MirOp, MirProgram, MirTerminator, MirType,
        MirValue,
    };
    use std::collections::HashMap;

    #[test]
    fn unresolved_qualified_builtin_is_not_hidden_from_validation() {
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "main".to_string(),
                params: Vec::new(),
                ret_type: DataType::None,
                body_hash: 0,
                noinline: false,
                blocks: vec![MirBlock {
                    id: 0,
                    label: "entry".to_string(),
                    insts: vec![MirInst {
                        result: Some(0),
                        op: MirOp::Call(
                            MirValue::Global("env.get".to_string()),
                            Vec::new(),
                            MirType {
                                data_type: DataType::Str,
                            },
                        ),
                        loc: (1, 1),
                    }],
                    terminator: MirTerminator::Ret(None),
                }],
            }],
            entry_point: Some("main".to_string()),
            extern_functions: Vec::new(),
            extern_libs: Vec::new(),
            struct_types: HashMap::new(),
            globals: HashMap::new(),
        };

        assert_eq!(
            find_first_undefined_call(&program),
            Some(("env.get".to_string(), (1, 1)))
        );
    }
}
