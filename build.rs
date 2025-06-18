use std::{fs, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=src/palette.rs");
    println!("cargo:rerun-if-changed=scripts/load-palette.py");

    let dest = PathBuf::from("src/palette.rs");
    fs::create_dir_all(dest.parent().unwrap()).unwrap();

    let status = Command::new("python3")
        .arg("scripts/load-palette.py")
        .arg("palettes/smooth.pal")
        .stdout(fs::File::create(&dest).unwrap())
        .status()
        .expect("failed to run Python generator");

    assert!(status.success(), "palette generation failed");
}
