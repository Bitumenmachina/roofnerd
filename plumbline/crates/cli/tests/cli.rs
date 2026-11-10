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

#[test]
fn a_reader_that_stops_listening_is_not_a_panic() {
    use std::process::Stdio;
    let dir = std::env::temp_dir().join(format!("plumbline-pipe-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let job = dir.join("job.json");
    let bin = env!("CARGO_BIN_EXE_plumbline");
    assert!(Command::new(bin).arg("sample").arg(&job).output().unwrap().status.success());
    // Each command, with nobody reading what it prints (as under `| head -1`).
    let job_text = job.to_str().unwrap();
    for args in [
        vec!["show", job_text],
        vec!["lines", job_text],
        vec!["check", job_text],
        vec!["apply", job_text, "parapet-0024", "Parapet with copper coping", "--source", "x"],
        vec!["set", job_text, "parapet-0024", "stretch_out", "24 in", "--line", "Copper coping cap", "--status", "Known", "--source", "x"],
        vec!["set", job_text, "parapet-0024", "quantity", "plan_length", "--line", "Continuous cleat", "--status", "Known", "--source", "x"],
    ] {
        let mut child = Command::new(bin).args(&args).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
        drop(child.stdout.take());
        let out = child.wait_with_output().unwrap();
        let said = String::from_utf8_lossy(&out.stderr);
        assert!(out.status.success() && !said.contains("panicked"), "{args:?}: {said}");
    }
    std::fs::remove_dir_all(&dir).unwrap();
}
