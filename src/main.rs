use std::{
    env,
    ffi::OsString,
    fs, io,
    os::unix::prelude::OsStringExt,
    path::PathBuf,
    process::{self, Command},
};

use ansi_term::Colour;

const OUT_PATH: &str = "target/coverage";
const DEPS_PATH: &str = "target/debug/deps/";

// 🚧 I should return something, and at least allow for returning errors of some
// of my subprocesses and function calls and whatnot.
fn main() -> io::Result<()> {
    let current_dir = env::current_dir().unwrap();
    let mut root = find_package_dir(&None).unwrap();
    let mut deps = root.clone();
    deps.push(DEPS_PATH);
    root.push(OUT_PATH);

    match Command::new("grcov").arg("-h").output() {
        Ok(_) => {}
        Err(_) => {
            eprintln!(
                "🚧 {0} is not installed Please install {0} to continue. See {1}.",
                Colour::Yellow.italic().paint("grcov"),
                Colour::Blue
                    .italic()
                    .paint("https://github.com/mozilla/grcov")
            );
            std::process::exit(1);
        }
    };

    // Remove all existing profraw files
    for entry in fs::read_dir(&root)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(extension) = path.extension() {
            if extension == "profraw" {
                fs::remove_file(&path)?;
            }
        }
    }

    let child = Command::new("cargo")
        .arg("test")
        .env("CARGO_INCREMENTAL", "0")
        .env("RUSTFLAGS", "-Cinstrument-coverage")
        .env(
            "LLVM_PROFILE_FILE",
            format!("{}/cargo-test-%p-%m.profraw", OUT_PATH),
        )
        .spawn()?;
    let _ = child.wait_with_output().expect("failed to wait on child");

    env::set_current_dir(root)?;
    let child = Command::new("grcov")
        .arg(".")
        .arg("--binary-path")
        .arg(&deps)
        .arg("-s")
        .arg(".")
        .arg("-t")
        .arg("html")
        .arg("--branch")
        .arg("--ignore-not-existing")
        .arg("--ignore")
        .arg("'../*'")
        .arg("--ignore")
        .arg("'/*'")
        .arg("-o")
        .arg("./output/html/")
        .spawn()?;
    // 🚧 We should check that grcov is installed before we start running shit.
    // I wonder if we can actually just suck it in as a dependency, and run it
    // without spawning a shell?
    // .expect("failed to execute process");
    let _ = child.wait_with_output()?;

    let child = Command::new("grcov")
        .arg(".")
        .arg("--binary-path")
        .arg(deps)
        .arg("-s")
        .arg(".")
        .arg("-t")
        .arg("lcov")
        .arg("--branch")
        .arg("--ignore-not-existing")
        .arg("--ignore")
        .arg("'../*'")
        .arg("--ignore")
        .arg("'/*'")
        .arg("-o")
        .arg("./output/coverage.lcov")
        .spawn()?;
    // .expect("failed to execute process");
    let _ = child.wait_with_output()?;

    env::set_current_dir(current_dir)
}

fn find_package_dir(start_dir: &Option<PathBuf>) -> Result<PathBuf, std::io::Error> {
    if let Some(dir) = start_dir {
        std::env::set_current_dir(dir)?;
    }

    // Figure out where Cargo.toml is located.
    //
    let output = process::Command::new("cargo")
        .arg("locate-project")
        .arg("--message-format")
        .arg("plain")
        .output()?;
    // .context(
    //     "😱 Tried running `cargo locate-project to no avail. \
    //         Maybe you need to add cargo to you path?",
    // )?;

    // anyhow::ensure!(
    //     output.status.success(),
    //     format!(
    //         "😱 Unable to find package in directory: {:?}.",
    //         std::env::current_dir()?
    //     )
    // );

    let mut stdout = output.stdout;

    // I don't know if it's kosher, but this does nicely to get rid of
    // that newline character.
    stdout.pop();
    let os_string = OsString::from_vec(stdout);
    let mut package_root = PathBuf::from(os_string);
    // Get rid of Cargo.toml
    package_root.pop();

    // debug!("Found root 🦀 at {:?}!", package_root);

    Ok(package_root)
}
