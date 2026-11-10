//! The command line as a person meets it.

use std::process::Command;

#[cfg(unix)]
#[test]
fn an_argument_that_is_not_text_is_refused_not_a_panic() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let dir = std::env::temp_dir().join(format!("plumbline-cli-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let job = dir.join("job.json");
    assert!(Command::new(env!("CARGO_BIN_EXE_plumbline")).arg("sample").arg(&job).output().unwrap().status.success());
    let before = std::fs::read(&job).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_plumbline")).arg("set").arg(&job).args(["plane-0017", "pitch"]).arg(OsStr::from_bytes(b"4\xff")).args(["--status", "Known", "--source", "x"]).output().unwrap();
    let said = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{said}");
    assert!(said.contains("not readable text") && !said.contains("panicked"), "{said}");
    assert_eq!(std::fs::read(&job).unwrap(), before);
    std::fs::remove_dir_all(&dir).unwrap();
}
