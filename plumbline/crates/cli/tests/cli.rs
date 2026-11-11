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

/// A system's total length is no surer than the runs in it, and the takeoff marks it so
/// (ninth judgment): one piece only assumed to be on a system makes its heading read `~`.
#[test]
fn a_takeoffs_heading_carries_the_flag_of_its_total() {
    let dir = std::env::temp_dir().join(format!("plumbline-takeoff-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let job = dir.join("house.json");
    let bin = env!("CARGO_BIN_EXE_plumbline");
    assert!(Command::new(bin).arg("house").arg(&job).output().unwrap().status.success());
    let takeoff = |job: &std::path::Path| String::from_utf8(Command::new(bin).arg("takeoff").arg(job).output().unwrap().stdout).unwrap();
    // As written, every run on the vent is known to be on it: 12.10 LF, no mark, one piece.
    let before = takeoff(&job);
    assert!(before.contains("Vent  (Vent; 1 straight piece, 12.10 LF)"), "{before}");
    assert!(before.contains(" 1 piece "), "one piece is not \"1 pieces\": {before}");
    // Mark that run as only assumed to be on the vent system.
    let mut doc: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&job).unwrap()).unwrap();
    let vent = doc["data"].as_array().unwrap().iter().find(|e| e["attributes"]["pl::system"]["classification"] == "Vent").unwrap()["path"].as_str().unwrap().to_string();
    for entry in doc["data"].as_array_mut().unwrap() {
        if entry["attributes"]["pl::class"] == "Pipe" && entry["attributes"]["pl::in_system"]["value"] == vent.as_str() {
            entry["attributes"]["pl::in_system"]["status"] = "Assumed".into();
        }
    }
    std::fs::write(&job, serde_json::to_string_pretty(&doc).unwrap()).unwrap();
    let after = takeoff(&job);
    assert!(after.contains("Vent  (Vent; 1 straight piece, ~12.10 LF)"), "{after}");
    std::fs::remove_dir_all(&dir).unwrap();
}
