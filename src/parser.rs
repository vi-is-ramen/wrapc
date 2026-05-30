use crate::*;
use std::path::PathBuf;

/// Internal representation of a parsed argument.
#[derive(Debug, Clone)]
#[allow(unused)]
enum Arg {
    // Flags
    Help,
    Version,
    Verbose,
    Test,
    OptLevel,
    DebugInfo,

    // Single values
    CrateName(String),
    Edition(String),
    OutDir(PathBuf),
    Out(PathBuf),
    Target(String),
    Sysroot(PathBuf),
    ErrorFormat(String),
    Color(String),
    Json(String),
    CapLints(String),
    Explain(String),
    Print(String),
    Unpretty(String),

    // Collections
    Cfg(String),
    CheckCfg(String),
    CrateType(String),
    Emit(String),
    Extern(String),
    RemapPathPrefix(String),

    LibPath(LibrarySearchPath),
    Link(LinkLib),

    Warn(String),
    Allow(String),
    Deny(String),
    Forbid(String),

    Codegen(String),
    Z(String),

    #[allow(clippy::enum_variant_names)]
    ArgFile(PathBuf),
    Input(PathBuf),
    Unknown(String),
}

fn split_eq(arg: &str) -> Option<(&str, &str)> {
    let (key, val) = arg.split_once('=')?;

    Some((key, val))
}

fn parse_libpath(val: &str) -> LSP {
    if let Some((kind, path)) = val.split_once('=') {
        let k = match kind {
            "dependency" => LSPK::Dep,
            "crate" => LSPK::Crate,
            "native" => LSPK::Native,
            "framework" => LSPK::Framework,
            _ => LSPK::All,
        };
        LSP {
            kind: k,
            path: path.into(),
        }
    } else {
        LSP {
            kind: LSPK::All,
            path: val.into(),
        }
    }
}

fn parse_link(val: &str) -> LinkLib {
    let (prefix, rest) = match val.split_once('=') {
        Some((p, r)) => (Some(p), r),
        None => (None, val),
    };

    let mut kind = None;
    let mut modifiers = Vec::new();

    if let Some(p) = prefix {
        let (k_str, mods_str) = match p.split_once(':') {
            Some((k, m)) => (k, Some(m)),
            None => (p, None),
        };

        kind = match k_str {
            "dylib" => Some(LLK::Dylib),
            "static" => Some(LLK::Static),
            "framework" => Some(LLK::Framework),
            _ => None,
        };

        if let Some(m) = mods_str {
            for mod_str in m.split(',') {
                modifiers.push(mod_str.to_string());
            }
        }
    }

    let (name, rename) = match rest.split_once(':') {
        Some((n, r)) => (n.to_string(), Some(r.to_string())),
        None => (rest.to_string(), None),
    };

    LL {
        kind,
        modifiers,
        name,
        rename,
    }
}

fn parse_emit(val: &str) -> Vec<Emit> {
    let mut emits = Vec::new();
    for part in val.split(',') {
        if let Some((kind, path)) = part.split_once('=') {
            emits.push(Emit {
                kind: kind.to_string(),
                path: Some(path.into()),
            });
        } else {
            emits.push(Emit {
                kind: part.to_string(),
                path: None,
            });
        }
    }
    emits
}

fn parse_extern(val: &str) -> Extern {
    if let Some((name, path)) = val.split_once('=') {
        Extern {
            name: name.to_string(),
            path: Some(path.into()),
        }
    } else {
        Extern {
            name: val.to_string(),
            path: None,
        }
    }
}

fn parse_remap(val: &str) -> Option<(String, String)> {
    let (from, to) = val.split_once('=')?;
    Some((from.to_string(), to.to_string()))
}

fn parse_arg(arg: &str, next_arg: Option<&str>) -> (Arg, usize) {
    if let Some(stripped) = arg.strip_prefix('@') {
        return (Arg::ArgFile(stripped.into()), 1);
    }

    let macro_match = |key: &str, val: &str, consumed: usize| -> (Arg, usize) {
        match key {
            "--crate-name" => (Arg::CrateName(val.to_string()), consumed),
            "--edition" => (Arg::Edition(val.to_string()), consumed),
            "--out-dir" => (Arg::OutDir(val.into()), consumed),
            "-o" | "--out" => (Arg::Out(val.into()), consumed),
            "--target" => (Arg::Target(val.to_string()), consumed),
            "--sysroot" => (Arg::Sysroot(val.into()), consumed),
            "--error-format" => (Arg::ErrorFormat(val.to_string()), consumed),
            "--color" => (Arg::Color(val.to_string()), consumed),
            "--json" => (Arg::Json(val.to_string()), consumed),
            "--cap-lints" => (Arg::CapLints(val.to_string()), consumed),
            "--explain" => (Arg::Explain(val.to_string()), consumed),
            "--print" => (Arg::Print(val.to_string()), consumed),
            "--unpretty" => (Arg::Unpretty(val.to_string()), consumed),
            "--cfg" => (Arg::Cfg(val.to_string()), consumed),
            "--check-cfg" => (Arg::CheckCfg(val.to_string()), consumed),
            "--crate-type" => (Arg::CrateType(val.to_string()), consumed),
            "--emit" => (Arg::Emit(val.to_string()), consumed),
            "--extern" => (Arg::Extern(val.to_string()), consumed),
            "--remap-path-prefix" => (Arg::RemapPathPrefix(val.to_string()), consumed),
            "-W" | "--warn" => (Arg::Warn(val.to_string()), consumed),
            "-A" | "--allow" => (Arg::Allow(val.to_string()), consumed),
            "-D" | "--deny" => (Arg::Deny(val.to_string()), consumed),
            "-F" | "--forbid" => (Arg::Forbid(val.to_string()), consumed),
            "-C" | "--codegen" => (Arg::Codegen(val.to_string()), consumed),
            "-Z" => (Arg::Z(val.to_string()), consumed),
            "-L" => (Arg::LibPath(parse_libpath(val)), consumed),
            "-l" => (Arg::Link(parse_link(val)), consumed),
            _ => (Arg::Unknown(arg.to_string()), 1),
        }
    };

    if let Some((key, val)) = split_eq(arg) {
        let res = macro_match(key, val, 1);
        if let Arg::Unknown(_) = res.0 {
            // Fall through to exact match or prefix logic
        } else {
            return res;
        }
    }

    match arg {
        "-h" | "--help" => (Arg::Help, 1),
        "-V" | "--version" => (Arg::Version, 1),
        "-v" | "--verbose" => (Arg::Verbose, 1),
        "--test" => (Arg::Test, 1),
        "-O" => (Arg::OptLevel, 1),
        "-g" => (Arg::DebugInfo, 1),

        "--crate-name"
        | "--edition"
        | "--out-dir"
        | "-o"
        | "--out"
        | "--target"
        | "--sysroot"
        | "--error-format"
        | "--color"
        | "--json"
        | "--cap-lints"
        | "--explain"
        | "--print"
        | "--unpretty"
        | "--cfg"
        | "--check-cfg"
        | "--crate-type"
        | "--emit"
        | "--extern"
        | "--remap-path-prefix"
        | "-W"
        | "--warn"
        | "-A"
        | "--allow"
        | "-D"
        | "--deny"
        | "-F"
        | "--forbid"
        | "-C"
        | "--codegen"
        | "-Z"
        | "-L"
        | "-l" => {
            if let Some(val) = next_arg {
                macro_match(arg, val, 2)
            } else {
                (Arg::Unknown(arg.to_string()), 1)
            }
        }

        _ => {
            if arg.starts_with('-') {
                if let Some(stripped) = arg.strip_prefix("-C") {
                    return (Arg::Codegen(stripped.to_string()), 1);
                }
                if let Some(stripped) = arg.strip_prefix("-Z") {
                    return (Arg::Z(stripped.to_string()), 1);
                }
                if let Some(stripped) = arg.strip_prefix("-W") {
                    return (Arg::Warn(stripped.to_string()), 1);
                }
                if let Some(stripped) = arg.strip_prefix("-A") {
                    return (Arg::Allow(stripped.to_string()), 1);
                }
                if let Some(stripped) = arg.strip_prefix("-D") {
                    return (Arg::Deny(stripped.to_string()), 1);
                }
                if let Some(stripped) = arg.strip_prefix("-F") {
                    return (Arg::Forbid(stripped.to_string()), 1);
                }
                if let Some(stripped) = arg.strip_prefix("-l") {
                    return (Arg::Link(parse_link(stripped)), 1);
                }
                if let Some(stripped) = arg.strip_prefix("-L") {
                    return (Arg::LibPath(parse_libpath(stripped)), 1);
                }

                (Arg::Unknown(arg.to_string()), 1)
            } else {
                (Arg::Input(arg.into()), 1)
            }
        }
    }
}

/// Parses an iterator of strings into an [`Info`] struct.
///
/// This function handles the standard `RUSTC_WRAPPER` invocation format:
/// `<wrapper> - <rustc> <args...>`.
#[allow(clippy::field_reassign_with_default)]
pub fn parse<I, S>(args: I) -> Result<Info, ParseError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut args: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();
    let mut info = Info::default();

    info.rustc = Some(args[2].clone());
    args.drain(0..3);

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        let next_arg = args.get(i + 1).map(|s| s.as_str());

        if arg == "--" {
            info.unknown.push(arg.clone());
            for rest in &args[i + 1..] {
                info.unknown.push(rest.clone());
            }
            break;
        }

        let (parsed, consumed) = parse_arg(arg, next_arg);

        match parsed {
            Arg::Help => info.help = true,
            Arg::Version => info.version = true,
            Arg::Verbose => info.verbose = true,
            Arg::Test => info.test = true,
            Arg::OptLevel => info.opt_level = true,
            Arg::DebugInfo => info.debug_info = true,

            Arg::CrateName(v) => info.crate_name = Some(v),
            Arg::Edition(v) => info.edition = Some(v),
            Arg::OutDir(v) => info.out_dir = Some(v),
            Arg::Out(v) => info.out = Some(v),
            Arg::Target(v) => info.target = Some(v),
            Arg::Sysroot(v) => info.sysroot = Some(v),
            Arg::ErrorFormat(v) => info.error_format = Some(v),
            Arg::Color(v) => info.color = Some(v),
            Arg::Json(v) => info.json = Some(v),
            Arg::CapLints(v) => info.cap_lints = Some(v),
            Arg::Explain(v) => info.explain = Some(v),
            Arg::Print(v) => info.print = Some(v),
            Arg::Unpretty(v) => info.unpretty = Some(v),

            Arg::Cfg(v) => info.configs.push(v),
            Arg::CheckCfg(v) => info.confchecks.push(v),
            Arg::CrateType(v) => info.crate_types.push(v),
            Arg::LibPath(v) => info.libpaths.push(v),
            Arg::Link(v) => info.links.push(v),
            Arg::Warn(v) => info.warns.push(v),
            Arg::Allow(v) => info.allows.push(v),
            Arg::Deny(v) => info.denies.push(v),
            Arg::Forbid(v) => info.forbids.push(v),
            Arg::Codegen(v) => info.codegen_opts.push(v),
            Arg::Z(v) => info.z_opts.push(v),
            Arg::Input(v) => info.inputs.push(v),
            Arg::Unknown(v) => info.unknown.push(v),
            Arg::Extern(v) => info.externs.push(parse_extern(&v)),
            Arg::ArgFile(_) => info.unknown.push(arg.clone()),
            Arg::Emit(v) => info.emits.extend(parse_emit(&v)),

            Arg::RemapPathPrefix(v) => {
                if let Some(remap) = parse_remap(&v) {
                    info.remap_path_prefixes.push(remap);
                }
            }
        }

        i += consumed;
    }

    Ok(info)
}

/// Fetches arguments from `std::env::args()` and parses them.
pub fn fetch() -> Result<Info, ParseError> {
    let args: Vec<String> = std::env::args().collect();
    parse(&args)
}
