// ── The structure ──────────────────────────────────────────────────────────
// This file holds the one open job and hands the same copy to every window.
//
// A window does not own the job. It asks for it once when it opens, then it
// subscribes, and every change arrives as an event. That is the only reason two
// windows can never disagree: there is only ever one document, and it is here.
//
// No estimating logic lives in this file, and none ever will. Rust holds the
// document, opens windows, and touches the disk. Every number in this program
// is computed in packages/engine, which is plain arithmetic with no window
// anywhere near it.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};

/// The parts of a job, and the file each one lives in.
///
/// The document is keyed by the PART NAME, not by the file path, and that is
/// load-bearing: a JSON Pointer splits on "/", so a key like "pages/pages.json"
/// would read as two steps and never find anything. Windows address the job as
/// "/pages/0/feetPerUnit", which is what an estimator would expect it to look
/// like anyway. `packages/engine/src/job.ts` owns this list.
const JOB_PARTS: [(&str, &str); 4] = [
    ("job", "job.json"),
    ("conditions", "conditions.json"),
    ("pages", "pages/pages.json"),
    ("costCodes", "cost-codes.json"),
];

/// The open job: where it came from, and what it currently says.
struct OpenJob {
    folder: Option<PathBuf>,
    doc: Value,
}

struct Document(Mutex<OpenJob>);

/// Canonical JSON: sorted keys (serde_json's BTreeMap does this for us), two
/// spaces, one trailing newline. The same bytes the engine writes, so a job
/// saved here and a job saved from the command line diff to nothing.
fn canonical(value: &Value) -> Result<String, String> {
    let mut s = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    s.push('\n');
    Ok(s)
}

fn read_folder(folder: &Path) -> Result<Value, String> {
    let mut doc = json!({});
    for (part, rel) in JOB_PARTS {
        let path = folder.join(rel);
        let raw = match fs::read_to_string(&path) {
            Ok(raw) => raw,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(format!("{}: {e}", path.display())),
        };
        let parsed: Value =
            serde_json::from_str(&raw).map_err(|e| format!("{}: {e}", path.display()))?;
        doc[part] = parsed;
    }
    Ok(doc)
}

/// Hand a window the document as it stands.
#[tauri::command]
fn doc_get(state: State<'_, Document>) -> Result<Value, String> {
    let guard = state.0.lock().map_err(|_| "the document lock is poisoned".to_string())?;
    Ok(guard.doc.clone())
}

/// Where the open job came from, so a window can show it. None until one opens.
#[tauri::command]
fn doc_folder(state: State<'_, Document>) -> Result<Option<String>, String> {
    let guard = state.0.lock().map_err(|_| "the document lock is poisoned".to_string())?;
    Ok(guard.folder.as_ref().map(|p| p.display().to_string()))
}

/// Where the demo job sits, for the Open button to prefill.
///
/// A relative path would resolve against wherever the program was launched from,
/// which is not the repository root. This resolves against the source tree at
/// build time, so it answers in a checkout and answers `None` once installed —
/// at which point the estimator types the path to their own job, which is the
/// normal case anyway.
#[tauri::command]
fn demo_folder() -> Option<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../jobs/demo-job");
    fs::canonicalize(path).ok().map(|p| p.display().to_string())
}

/// Change one value in the document and tell every window.
///
/// `pointer` is an RFC 6901 JSON Pointer — "/job.json/name", or
/// "/conditions.json/0/properties/height". One general mechanism instead of a
/// command per field, so the shell never grows estimating knowledge.
#[tauri::command]
fn doc_set(app: AppHandle, state: State<'_, Document>, pointer: String, value: Value) -> Result<(), String> {
    // Take the lock, make the change, take a copy, and then LET GO. The lock is
    // never held while the document is being serialized out to every window: one
    // slow or hung window must not be able to stall the job for the others.
    let snapshot = {
        let mut guard = state.0.lock().map_err(|_| "the document lock is poisoned".to_string())?;
        let slot = guard
            .doc
            .pointer_mut(&pointer)
            .ok_or_else(|| format!("nothing at {pointer} in this job"))?;
        *slot = value;
        guard.doc.clone()
    };

    app.emit("doc:changed", snapshot).map_err(|e| e.to_string())
}

/// Open a job folder. Replaces whatever was open; every window follows.
#[tauri::command]
fn doc_open(app: AppHandle, state: State<'_, Document>, folder: String) -> Result<(), String> {
    let path = PathBuf::from(&folder);
    let doc = read_folder(&path)?;

    let snapshot = {
        let mut guard = state.0.lock().map_err(|_| "the document lock is poisoned".to_string())?;
        guard.folder = Some(path);
        guard.doc = doc;
        guard.doc.clone()
    };

    app.emit("doc:changed", snapshot).map_err(|e| e.to_string())
}

/// Write the open job back to its folder, from the one document.
///
/// The save comes from here rather than from a window, so it cannot pick up a
/// copy that one window happens to be holding.
#[tauri::command]
fn doc_save(state: State<'_, Document>) -> Result<String, String> {
    let (folder, doc) = {
        let guard = state.0.lock().map_err(|_| "the document lock is poisoned".to_string())?;
        let folder = guard.folder.clone().ok_or("no job is open")?;
        (folder, guard.doc.clone())
    };

    for (part, rel) in JOB_PARTS {
        let Some(value) = doc.get(part) else { continue };
        let path = folder.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        fs::write(&path, canonical(value)?).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(folder.display().to_string())
}

/// Copy a drawing the estimator picked into the job folder, and say where it
/// landed. The job keeps a path, never the drawing's bytes: a job file stays
/// small and readable, and the PDF stays the PDF.
#[tauri::command]
fn add_page_source(state: State<'_, Document>, source: String) -> Result<String, String> {
    let folder = {
        let guard = state.0.lock().map_err(|_| "the document lock is poisoned".to_string())?;
        guard.folder.clone().ok_or("no job is open")?
    };

    let from = PathBuf::from(&source);
    let name = from
        .file_name()
        .ok_or_else(|| format!("{source} is not a file"))?
        .to_string_lossy()
        .to_string();

    let pages = folder.join("pages");
    fs::create_dir_all(&pages).map_err(|e| format!("{}: {e}", pages.display()))?;
    let to = pages.join(&name);
    if from != to {
        fs::copy(&from, &to).map_err(|e| format!("{}: {e}", from.display()))?;
    }
    Ok(format!("pages/{name}"))
}

/// Read a drawing back out of the job folder.
///
/// The path is relative to the open job and is checked to stay inside it. A
/// window asks for drawings by name; it does not get to name a file anywhere on
/// the machine and have this program read it out.
#[tauri::command]
fn read_page_source(state: State<'_, Document>, relative: String) -> Result<Vec<u8>, String> {
    let folder = {
        let guard = state.0.lock().map_err(|_| "the document lock is poisoned".to_string())?;
        guard.folder.clone().ok_or("no job is open")?
    };

    let root = fs::canonicalize(&folder).map_err(|e| format!("{}: {e}", folder.display()))?;
    let path = fs::canonicalize(root.join(&relative))
        .map_err(|e| format!("{relative}: {e}"))?;
    if !path.starts_with(&root) {
        return Err(format!("{relative} is outside this job"));
    }
    fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))
}

/// Tear an editor off into its own window, which can go to the other monitor.
///
/// Every window loads the same page and reads which editor it is from its own
/// URL. An editor is not a different program; it is a different view of the one
/// document, and that is why any of them can live in any window.
#[tauri::command]
fn open_editor(app: AppHandle, editor: String) -> Result<(), String> {
    if let Some(existing) = app.get_webview_window(&editor) {
        existing.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    let title = format!("{} — {editor}", app.package_info().name);
    WebviewWindowBuilder::new(
        &app,
        &editor,
        WebviewUrl::App(format!("index.html?editor={editor}").into()),
    )
    .title(title)
    .inner_size(760.0, 800.0)
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// WebKitGTK renders through DMABUF by default, and that path is broken against
/// the proprietary NVIDIA driver — the window dies on startup with a Wayland
/// protocol error, or fills the log with GBM buffer failures under XWayland.
/// Turning the fast path off fixes it. This is narrowed to machines that
/// actually have an NVIDIA device so everyone else keeps the fast path, and it
/// never overrides the variable if someone has already set it themselves.
#[cfg(target_os = "linux")]
fn work_around_nvidia_webkit() {
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_some() {
        return;
    }
    if Path::new("/dev/nvidiactl").exists() {
        // Safe here: this runs before any thread is spawned.
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "linux")]
    work_around_nvidia_webkit();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Document(Mutex::new(OpenJob {
            folder: None,
            doc: json!({}),
        })))
        .invoke_handler(tauri::generate_handler![
            doc_get,
            doc_folder,
            demo_folder,
            doc_set,
            doc_open,
            doc_save,
            add_page_source,
            read_page_source,
            open_editor
        ])
        .run(tauri::generate_context!())
        .expect("roofnerd failed to start");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../jobs/demo-job")
    }

    /// The claim in Cargo.toml, tested rather than asserted: a job saved by the
    /// application is byte-for-byte a job saved by the command line. If these
    /// two ever drift, a save from the window would show up as a diff against a
    /// save from a script, and nobody would know which one was right.
    #[test]
    fn the_shell_writes_the_same_bytes_the_engine_writes() {
        for (_part, rel) in JOB_PARTS {
            let path = demo().join(rel);
            let on_disk = fs::read_to_string(&path).expect("the demo job is in the repository");
            let parsed: Value = serde_json::from_str(&on_disk).unwrap();
            assert_eq!(
                canonical(&parsed).unwrap(),
                on_disk,
                "{rel} is not what this shell would write"
            );
        }
    }

    #[test]
    fn a_folder_reads_into_one_document_keyed_by_file() {
        let doc = read_folder(&demo()).unwrap();
        assert_eq!(doc["job"]["name"], "Demo Warehouse Reroof");
        assert!(doc["costCodes"].as_array().unwrap().len() >= 6);
        assert_eq!(doc["conditions"][0]["name"], "Parapet Wall Flashing");
    }

    #[test]
    fn a_missing_file_is_absent_rather_than_fatal() {
        let dir = std::env::temp_dir().join("roofnerd-empty-job");
        fs::create_dir_all(&dir).unwrap();
        let _ = fs::remove_file(dir.join("job.json"));
        assert!(read_folder(&dir).is_ok());
    }

    #[test]
    fn a_pointer_reaches_a_nested_property() {
        let mut doc = read_folder(&demo()).unwrap();
        let slot = doc
            .pointer_mut("/conditions/0/properties/H")
            .expect("the demo condition has a height");
        *slot = json!(2.5);
        assert_eq!(doc["conditions"][0]["properties"]["H"], 2.5);
    }

    #[test]
    fn a_pointer_at_nothing_is_refused_rather_than_created() {
        let mut doc = read_folder(&demo()).unwrap();
        assert!(doc.pointer_mut("/conditions/0/properties/nonsense").is_none());
    }
}
