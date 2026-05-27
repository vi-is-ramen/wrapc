use wrapc::{parse, Info};

fn main() {
    // Example invocation via Cargo's RUSTC_WRAPPER protocol
    let args = vec![
        "wrapc", "-", "rustc", 
        "--crate-name=my_crate", 
        "--edition=2021",
        "-L", "native=/usr/lib", 
        "-l", "static:+bundle,+whole-archive=mylib:renamed_lib", 
        "--emit=link,dep-info=/tmp/dep.d",
        "-C", "opt-level=3",
        "-Clink-arg=-fuse-ld=lld",
        "src/main.rs",
        "-o",
        "sth.exe",
    ];
    
    let info: Info = parse(&args).expect("Failed to parse arguments");
    
    println!("{:?}", info);

    // Reconstruct args accurately for rustc execution
    let reconstructed = info.to_args();
    println!("\nReconstructed arguments: {:?}", reconstructed);
}
