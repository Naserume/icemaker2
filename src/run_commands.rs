use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use tempdir::TempDir;

fn get_cmd_string(cmd: &std::process::Command) -> String {
    let envs: String = cmd
        .get_envs()
        .filter(|(_, y)| y.is_some())
        .map(|(x, y)| format!("{}={}", x.to_string_lossy(), y.unwrap().to_string_lossy()))
        .collect::<Vec<String>>()
        .join(" ");
    let command = format!("{:?}", cmd);
    format!("\"{}\" {}", envs, command).replace('"', "")
}

pub(crate) fn run_rustc(
    executable: &str,
    file: &Path,
    incremental: bool,
    rustc_flags: &[&str],
) -> (Output, String) {
    if incremental {
        // only run incremental compilation tests
        return run_rustc_incremental(executable, file);
    }
    // if the file contains no "main", run with "--crate-type lib"
    let has_main = std::fs::read_to_string(&file)
        .unwrap_or_default()
        .contains("pub(crate) fn main(");

    //let tempdir = TempDir::new("rustc_testrunner_tmpdir").unwrap();
    //let tempdir_path = tempdir.path();
    let output_file = String::from("-o/dev/null");
    let dump_mir_dir = String::from("-Zdump-mir-dir=/dev/null");

    let mut output = Command::new(executable);
    output
        .arg(&file)
        .args(rustc_flags)
        // always keep these:
        .arg(&output_file)
        .arg(&dump_mir_dir);
    if !has_main {
        output.args(&["--crate-type", "lib"]);
    }
    //dbg!(&output);
    // run the command
    (
        output
            .output()
            .unwrap_or_else(|_| panic!("Error: {:?}, executable: {:?}", output, executable)),
        get_cmd_string(&output),
    )
    // remove tempdir
    //tempdir.close().unwrap();
}

pub(crate) fn run_rustc_incremental(executable: &str, file: &Path) -> (Output, String) {
    let tempdir = TempDir::new("rustc_testrunner_tmpdir").unwrap();
    let tempdir_path = tempdir.path();

    let has_main = std::fs::read_to_string(&file)
        .unwrap_or_default()
        .contains("pub(crate) fn main(");

    let mut cmd = Command::new("DUMMY");
    let mut output = None;
    for i in &[0, 1] {
        let mut command = Command::new(executable);
        if !has_main {
            command.args(&["--crate-type", "lib"]);
        }
        command
            .arg(&file)
            // avoid error: the generated executable for the input file  .. onflicts with the existing directory..
            .arg(format!("-o{}/{}", tempdir_path.display(), i))
            .arg(format!("-Cincremental={}", tempdir_path.display()))
            .arg("-Zincremental-verify-ich=yes");

        output = Some(command.output());
        cmd = command;
    }

    let output = output.map(|output| output.unwrap()).unwrap();

    tempdir.close().unwrap();
    //dbg!(&output);
    (output, get_cmd_string(&cmd))
}

pub(crate) fn run_clippy(executable: &str, file: &Path) -> (Output, String) {
    let has_main = std::fs::read_to_string(&file)
        .unwrap_or_default()
        .contains("pub(crate) fn main(");
    let mut cmd = Command::new(executable);

    if !has_main {
        cmd.args(&["--crate-type", "lib"]);
    }
    cmd.env("RUSTFLAGS", "-Z force-unstable-if-unmarked")
        .env("SYSROOT", "/home/matthias/.rustup/toolchains/master")
        .arg(&file)
        .arg("-Aclippy::cargo") // allow cargo lints
        //.arg("-Wclippy::internal")
        .arg("-Wclippy::pedantic")
        .arg("-Wclippy::nursery")
        .arg("-Wmissing-doc-code-examples")
        .arg("-Wabsolute-paths-not-starting-with-crate")
        .arg("-Wbare-trait-objects")
        .arg("-Wbox-pointers")
        .arg("-Welided-lifetimes-in-paths")
        .arg("-Wellipsis-inclusive-range-patterns")
        .arg("-Wkeyword-idents")
        .arg("-Wmacro-use-extern-crate")
        .arg("-Wmissing-copy-implementations")
        .arg("-Wmissing-debug-implementations")
        .arg("-Wmissing-docs")
        .arg("-Wsingle-use-lifetimes")
        .arg("-Wtrivial-casts")
        .arg("-Wtrivial-numeric-casts")
        .arg("-Wunreachable-pub")
        .arg("-Wunsafe-code")
        .arg("-Wunstable-features")
        .arg("-Wunused-extern-crates")
        .arg("-Wunused-import-braces")
        .arg("-Wunused-labels")
        .arg("-Wunused-lifetimes")
        .arg("-Wunused-qualifications")
        .arg("-Wunused-results")
        .arg("-Wvariant-size-differences")
        .args(&["--cap-lints", "warn"])
        .args(&["-o", "/dev/null"]);
    (cmd.output().unwrap(), get_cmd_string(&cmd))
}

pub(crate) fn run_rustdoc(executable: &str, file: &Path) -> (Output, String) {
    let mut cmd = Command::new(executable);
    cmd.env("RUSTFLAGS", "-Z force-unstable-if-unmarked")
        .env("SYSROOT", "/home/matthias/.rustup/toolchains/master")
        .arg(&file)
        .arg("-Zunstable-options")
        .arg("--document-private-items")
        .arg("--document-hidden-items")
        .args(&["--cap-lints", "warn"])
        .args(&["-o", "/dev/null"]);
    let output = cmd.output().unwrap();
    (output, get_cmd_string(&cmd))
}

pub(crate) fn run_rust_analyzer(executable: &str, file: &Path) -> (Output, String) {
    let file_content = std::fs::read_to_string(&file).expect("failed to read file ");

    let mut cmd = Command::new(executable)
        .arg("symbols")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdin = &mut cmd.stdin.as_mut().unwrap();
    stdin.write_all(file_content.as_bytes()).unwrap();
    (
        cmd.wait_with_output().unwrap(),
        get_cmd_string(Command::new("rust-analyer").arg("symbols")),
    )

    /*
    let output = process.wait_with_output().unwrap();
    println!("\n\n{:?}\n\n", output);
    output
    */
}
pub(crate) fn run_rustfmt(executable: &str, file: &Path) -> (Output, String) {
    let mut cmd = Command::new(executable);
    cmd.env("SYSROOT", "/home/matthias/.rustup/toolchains/master")
        .arg(&file)
        .arg("--check")
        .args(&["--edition", "2018"]);
    let output = cmd.output().unwrap();
    (output, get_cmd_string(&cmd))
}