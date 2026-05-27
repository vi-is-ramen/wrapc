use std::path::PathBuf;
use wrapc::{parse, LibrarySearchPathKind, LinkLibKind};

fn args<const N: usize>(a: [&str; N]) -> Vec<String> {
    a.iter().map(|s| s.to_string()).collect()
}

#[test]
fn cargo_wrapper_protocol() {
    let a = args(["sccache", "-", "rustc", "--crate-name", "my_crate", "src/main.rs"]);
    let info = parse(&a).unwrap();
    
    assert_eq!(info.rustc, Some("rustc".to_string()));
    assert_eq!(info.crate_name, Some("my_crate".to_string()));
    assert_eq!(info.inputs, vec![PathBuf::from("src/main.rs")]);
}

#[test]
fn boolean_flags() {
    let a = args(["wrapc", "-", "rustc", "-h", "-V", "-v", "--test", "-O", "-g"]);
    let info = parse(&a).unwrap();
    
    assert!(info.help);
    assert!(info.version);
    assert!(info.verbose);
    assert!(info.test);
    assert!(info.opt_level);
    assert!(info.debug_info);
}

#[test]
fn equals_vs_space_syntax() {
    let a1 = args(["wrapc", "-", "rustc", "--edition=2021", "--out-dir=target"]);
    let a2 = args(["wrapc", "-", "rustc", "--edition", "2021", "--out-dir", "target"]);
    
    let info1 = parse(&a1).unwrap();
    let info2 = parse(&a2).unwrap();
    
    assert_eq!(info1, info2);
    assert_eq!(info1.edition, Some("2021".to_string()));
    assert_eq!(info1.out_dir, Some(PathBuf::from("target")));
}

#[test]
fn lints_slugged_and_long() {
    let a = args(["wrapc", "-", "rustc", "-Wunused", "--allow", "dead_code", "-Dwarnings", "--forbid=unsafe_code"]);
    let info = parse(&a).unwrap();
    
    assert_eq!(info.warns, vec!["unused"]);
    assert_eq!(info.allows, vec!["dead_code"]);
    assert_eq!(info.denies, vec!["warnings"]);
    assert_eq!(info.forbids, vec!["unsafe_code"]);
}

#[test]
fn codegen_and_z_flags() {
    let a = args(["wrapc", "-", "rustc", "-Copt-level=3", "-C", "link-arg=-fuse-ld=lld", "-Zmir-opt-level=3"]);
    let info = parse(&a).unwrap();
    
    assert_eq!(info.codegen_opts, vec!["opt-level=3", "link-arg=-fuse-ld=lld"]);
    assert_eq!(info.z_opts, vec!["mir-opt-level=3"]);
}

#[test]
fn library_search_paths() {
    let a = args(["wrapc", "-", "rustc", "-L", "/usr/lib", "-Lnative=/opt/lib", "-Ldependency=target/deps"]);
    let info = parse(&a).unwrap();
    
    assert_eq!(info.libpaths.len(), 3);
    
    assert_eq!(info.libpaths[0].kind, LibrarySearchPathKind::All);
    assert_eq!(info.libpaths[0].path, PathBuf::from("/usr/lib"));
    
    assert_eq!(info.libpaths[1].kind, LibrarySearchPathKind::Native);
    assert_eq!(info.libpaths[1].path, PathBuf::from("/opt/lib"));
    
    assert_eq!(info.libpaths[2].kind, LibrarySearchPathKind::Dep);
    assert_eq!(info.libpaths[2].path, PathBuf::from("target/deps"));
}

#[test]
fn link_libraries_complex() {
    let a = args(["wrapc", "-", "rustc", "-lssl", "-lstatic=crypto", "-lstatic:+bundle,+whole-archive=mylib:renamed"]);
    let info = parse(&a).unwrap();
    
    assert_eq!(info.links.len(), 3);
    
    assert_eq!(info.links[0].name, "ssl");
    assert_eq!(info.links[0].kind, None);
    
    assert_eq!(info.links[1].name, "crypto");
    assert_eq!(info.links[1].kind, Some(LinkLibKind::Static));
    assert!(info.links[1].modifiers.is_empty());
    
    let complex = &info.links[2];
    assert_eq!(complex.name, "mylib");
    assert_eq!(complex.rename, Some("renamed".to_string()));
    assert_eq!(complex.kind, Some(LinkLibKind::Static));
    assert_eq!(complex.modifiers, vec!["+bundle", "+whole-archive"]);
}

#[test]
fn emits_and_externs() {
    let a = args([
        "wrapc",
        "-",
        "rustc", 
        "--emit=link,dep-info=a.d", 
        "--extern", "foo=libfoo.rlib", 
        "--extern=bar"
    ]);
    let info = parse(&a).unwrap();
    
    assert_eq!(info.emits.len(), 2);
    assert_eq!(info.emits[0].kind, "link");
    assert_eq!(info.emits[0].path, None);
    assert_eq!(info.emits[1].kind, "dep-info");
    assert_eq!(info.emits[1].path, Some(PathBuf::from("a.d")));
    
    assert_eq!(info.externs.len(), 2);
    assert_eq!(info.externs[0].name, "foo");
    assert_eq!(info.externs[0].path, Some(PathBuf::from("libfoo.rlib")));
    assert_eq!(info.externs[1].name, "bar");
    assert_eq!(info.externs[1].path, None);
}

#[test]
fn double_dash_separator() {
    let a = args(["wrapc", "-", "rustc", "--crate-name", "foo", "--", "--ignored-flag", "-some-thing"]);
    let info = parse(&a).unwrap();
    
    assert_eq!(info.crate_name, Some("foo".to_string()));
    assert_eq!(info.unknown, vec!["--", "--ignored-flag", "-some-thing"]);
}

#[test]
fn missing_value_fallback() {
    let a = args(["wrapc", "-", "rustc", "--crate-name"]);
    let info = parse(&a).unwrap();
    
    assert_eq!(info.crate_name, None);
    assert_eq!(info.unknown, vec!["--crate-name"]);
}

#[test]
fn stdin_as_input_with_wrapper() {
    let a = args(["wrapc", "-", "rustc"]);
    let info = parse(&a).unwrap();
    
    assert_eq!(info.rustc, Some("rustc".to_string()));
    assert_eq!(info.inputs, Vec::<PathBuf>::new());
}

#[test]
fn response_files() {
    let a = args(["wrapc", "-", "rustc", "@args.txt", "src/main.rs"]);
    let info = parse(&a).unwrap();
    
    assert_eq!(info.unknown, vec!["@args.txt"]);
    assert_eq!(info.inputs, vec![PathBuf::from("src/main.rs")]);
}

#[test]
fn roundtrip_reconstruction() {
    let original_args = args([
        "wrapc", "-", "rustc", "--crate-name=my_crate", "--edition=2021", 
        "-L", "native=/usr/lib", "-l", "static:+bundle=mylib:renamed", 
        "--emit=link,dep-info=/tmp/dep.d", "-Copt-level=3", "-Wunused",
        "src/main.rs"
    ]);
    
    let info1 = parse(&original_args).unwrap();
    let reconstructed_without_wrapper = info1.to_args();
    let reconstructed = vec![
        // to match Cargo's compiler wrapper protocol
        vec!["wrapc".to_string(), "-".to_string(),
        // to_args produces only arguments, not binary name itself
        info1.rustc.clone().unwrap()],
        reconstructed_without_wrapper
    ].concat();
    let info2 = parse(&reconstructed).unwrap();
    
    assert_eq!(info1, info2);
}
