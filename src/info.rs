use std::path::PathBuf;

/// Kind of library search path passed via `-L`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum LibrarySearchPathKind {
    /// This value means "all except Framework".
    #[default]
    All,
    /// Dependency.
    Dep,
    /// Crate.
    Crate,
    /// Native.
    Native,
    /// MacOS Framework.
    Framework,
}

/// Just a convenient shortcut to [`LibrarySearchPathKind`].
pub type LSPK = LibrarySearchPathKind;

/// Represents a parsed `-L [KIND=]PATH` argument.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LibrarySearchPath {
    /// Library search path kind.
    pub kind: LibrarySearchPathKind,
    /// Path value.
    pub path: PathBuf,
}

/// Just a convenient shortcut to [`LibrarySearchPath`].
pub type LSP = LibrarySearchPath;

/// Kind of native library passed via `-l`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum LinkLibKind {
    /// Dynamic-linking library.
    #[default]
    Dylib,
    /// Static-linking library.
    Static,
    /// MacOS Framework.
    Framework,
}

/// Just a convenient shortcut to [`LinkLibKind`].
pub type LLK = LinkLibKind;

/// Represents a parsed `-l [KIND[:MODIFIERS]=]NAME[:RENAME]` argument.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LinkLib {
    /// Library linkage kind.
    pub kind: Option<LinkLibKind>,
    /// Linrary linkage modifiers.
    pub modifiers: Vec<String>,
    /// Name of library link to.
    pub name: String,
    /// Re-name of library link to.
    pub rename: Option<String>,
}

/// Just a convenient shortcut to [`LinkLib`].
pub type LL = LinkLib;

/// Represents a parsed `--emit` directive.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Emit {
    /// Emission kind.
    pub kind: String,
    /// Path.
    pub path: Option<PathBuf>,
}

/// Represents a parsed `--extern` directive.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Extern {
    /// Extern kind.
    pub name: String,
    /// Path.
    pub path: Option<PathBuf>,
}

/// The core structure containing all parsed `rustc` arguments.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Info {
    /// Original rustc path (extracted according to Cargo's
    /// `RUSTC_WRAPPER` protocol: `<wrapper> - <rustc> <args>`)
    pub rustc: Option<String>,
    
    /// Positional input files
    pub inputs: Vec<PathBuf>,

    // --- Flags ---
    /// `--help` present.
    pub help: bool,
    /// `-V` present.
    pub version: bool,
    /// `-v` present.
    pub verbose: bool,
    /// `--test` present.
    pub test: bool,
    /// `-O` present.
    pub opt_level: bool,
    /// `-g` present.
    pub debug_info: bool,
    
    // --- Simple Values ---
    pub crate_name: Option<String>,
    pub edition: Option<String>,
    pub out_dir: Option<PathBuf>,
    pub out: Option<PathBuf>,
    pub target: Option<String>,
    pub sysroot: Option<PathBuf>,
    pub error_format: Option<String>,
    pub color: Option<String>,
    pub json: Option<String>,
    pub cap_lints: Option<String>,
    pub explain: Option<String>,
    pub print: Option<String>,
    pub unpretty: Option<String>,
    
    // --- Collections ---
    pub configs: Vec<String>,         // --cfg
    pub confchecks: Vec<String>,      // --check-cfg
    pub crate_types: Vec<String>,     // --crate-type
    pub emits: Vec<Emit>,             // --emit
    pub externs: Vec<Extern>,         // --extern
    pub remap_path_prefixes: Vec<(String, String)>, // --remap-path-prefix
    
    pub libpaths: Vec<LibrarySearchPath>, // -L
    pub links: Vec<LinkLib>,              // -l
    
    pub warns: Vec<String>,
    pub allows: Vec<String>,
    pub denies: Vec<String>,
    pub forbids: Vec<String>,
    
    pub codegen_opts: Vec<String>,    // -C / --codegen
    pub z_opts: Vec<String>,          // -Z
    
    pub unknown: Vec<String>,
}

impl Info {
    /// Reconstructs the parsed arguments back into a `Vec<String>` suitable for passing to the actual `rustc`.
    pub fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        
        if self.help        { args.push("--help"    .into()); }
        if self.version     { args.push("--version" .into()); }
        if self.verbose     { args.push("--verbose" .into()); }
        if self.test        { args.push("--test"    .into()); }
        if self.opt_level   { args.push("-O"        .into()); }
        if self.debug_info  { args.push("-g"        .into()); }
        
        if let Some(v) = &self.crate_name   { args.push(format!("--crate-name={}"   , v             )); }
        if let Some(v) = &self.edition      { args.push(format!("--edition={}"      , v             )); }
        if let Some(v) = &self.out_dir      { args.push(format!("--out-dir={}"      , v.display()   )); }
        if let Some(v) = &self.out          { args.push(format!("-o={}"             , v.display()   )); }
        if let Some(v) = &self.target       { args.push(format!("--target={}"       , v             )); }
        if let Some(v) = &self.sysroot      { args.push(format!("--sysroot={}"      , v.display()   )); }
        if let Some(v) = &self.error_format { args.push(format!("--error-format={}" , v             )); }
        if let Some(v) = &self.color        { args.push(format!("--color={}"        , v             )); }
        if let Some(v) = &self.json         { args.push(format!("--json={}"         , v             )); }
        if let Some(v) = &self.cap_lints    { args.push(format!("--cap-lints={}"    , v             )); }
        if let Some(v) = &self.explain      { args.push(format!("--explain={}"      , v             )); }
        if let Some(v) = &self.print        { args.push(format!("--print={}"        , v             )); }
        if let Some(v) = &self.unpretty     { args.push(format!("--unpretty={}"     , v             )); }
        
        for v in &self.configs      { args.push(format!("--cfg={}"          , v)); }
        for v in &self.confchecks   { args.push(format!("--check-cfg={}"    , v)); }
        for v in &self.crate_types  { args.push(format!("--crate-type={}"   , v)); }
        
        if !self.emits.is_empty() {
            let mut emit_strs = Vec::new();
            for e in &self.emits {
                if let Some(p) = &e.path {
                    emit_strs.push(format!("{}={}", e.kind, p.display()));
                } else {
                    emit_strs.push(e.kind.clone());
                }
            }
            args.push(format!("--emit={}", emit_strs.join(",")));
        }
        
        for e in &self.externs {
            if let Some(p) = &e.path {
                args.push(format!("--extern={}={}", e.name, p.display()));
            } else {
                args.push(format!("--extern={}", e.name));
            }
        }
        
        for (from, to) in &self.remap_path_prefixes {
            args.push(format!("--remap-path-prefix={}={}", from, to));
        }
        
        for lp in &self.libpaths {
            let kind = match lp.kind {
                LSPK::All       => ""           ,
                LSPK::Dep       => "dependency=",
                LSPK::Crate     => "crate="     ,
                LSPK::Native    => "native="    ,
                LSPK::Framework => "framework=" ,
            };
            args.push(format!("-L{}{}", kind, lp.path.display()));
        }
        
        for l in &self.links {
            let mut s = String::from("-l");
            if let Some(k) = &l.kind {
                let k_str = match k {
                    LLK::Dylib      => "dylib"      ,
                    LLK::Static     => "static"     ,
                    LLK::Framework  => "framework"  ,
                };
                s.push_str(k_str);
                if !l.modifiers.is_empty() {
                    s.push(':');
                    s.push_str(&l.modifiers.join(","));
                }
                s.push('=');
            }
            s.push_str(&l.name);
            if let Some(r) = &l.rename {
                s.push(':');
                s.push_str(r);
            }
            args.push(s);
        }
        
        for v in &self.warns        { args.push(format!("-W{}", v)); }
        for v in &self.allows       { args.push(format!("-A{}", v)); }
        for v in &self.denies       { args.push(format!("-D{}", v)); }
        for v in &self.forbids      { args.push(format!("-F{}", v)); }
        for v in &self.codegen_opts { args.push(format!("-C{}", v)); }
        for v in &self.z_opts       { args.push(format!("-Z{}", v)); }
        
        args.extend(self.unknown.clone());
        
        for inp in &self.inputs {
            args.push(inp.to_string_lossy().into_owned());
        }
        
        args
    }
}
