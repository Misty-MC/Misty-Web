// build.rs — compile cubiomes (MIT, vendor/cubiomes) pour le seed viewer (`src/seedmap.rs`).

fn main() {
    println!("cargo:rerun-if-changed=vendor");
    let mut cc = cc::Build::new();
    cc.include("vendor").opt_level(3).warnings(false);
    for f in ["biomenoise", "biomes", "generator", "layers", "noise", "util", "finders"] {
        cc.file(format!("vendor/cubiomes/{f}.c"));
    }
    cc.file("vendor/misty_shim.c");
    cc.compile("cubiomes");
    // fonctions mathématiques de cubiomes (sqrt, floor…)
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        println!("cargo:rustc-link-lib=m");
    }
}
