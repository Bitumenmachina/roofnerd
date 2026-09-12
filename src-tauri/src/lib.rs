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
/// The files a job folder is made of.
///
/// `library.json` sits here for now, which makes the library per-job. The
/// proposal has it as a folder shared across jobs and that is where it ends up
/// — but a job that carries its own is the smallest thing that lets an assembly
/// be loaded onto a condition and priced, and the shape of the data does not
/// change when it moves. Recorded rather than pretended about.
const JOB_PARTS: [(&str, &str); 5] = [
    ("job", "job.json"),
    ("conditions", "conditions.json"),
    ("pages", "pages/pages.json"),
    ("costCodes", "cost-codes.json"),
    ("library", "library.json"),
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

/// Write a sheet the estimator asked for into the open job's own folder.
///
/// The same confinement `read_page_source` reads under, one direction along:
/// canonicalize, then `starts_with(root)`. Two differences, both deliberate.
///
/// The root here is the job's `exports` folder rather than the job folder, so
/// `../sheet.csv` is refused instead of quietly landing on `conditions.json`.
/// A sheet goes where sheets go.
///
/// And the file does not exist yet, so there is nothing to canonicalize. The
/// PARENT is canonicalized — which exists, because `exports` is made first —
/// and the name is taken off the end. A name is not a place: anything with a
/// folder in it, absolute or climbing, resolves to a parent that is not this
/// one and is refused. A name that is already a link is refused outright: a
/// sheet is a file, and following a link is how a write lands off the job.
fn write_export(folder: Option<&Path>, relative: &str, text: &str) -> Result<String, String> {
    let folder = folder.ok_or("no job is open")?;
    let root = fs::canonicalize(folder).map_err(|e| format!("{}: {e}", folder.display()))?;

    let exports = root.join("exports");
    fs::create_dir_all(&exports).map_err(|e| format!("{}: {e}", exports.display()))?;
    let exports = fs::canonicalize(&exports).map_err(|e| format!("{}: {e}", exports.display()))?;
    if !exports.starts_with(&root) {
        return Err("this job's exports folder is outside the job".to_string());
    }

    let asked = exports.join(relative);
    let parent = asked
        .parent()
        .ok_or_else(|| format!("{relative} does not name a sheet"))?;
    let parent = fs::canonicalize(parent).map_err(|e| format!("{relative}: {e}"))?;
    if !parent.starts_with(&exports) {
        return Err(format!("{relative} is outside this job"));
    }

    let name = asked
        .file_name()
        .ok_or_else(|| format!("{relative} does not name a sheet"))?;
    let path = parent.join(name);
    // A name that is a link is a name that points somewhere else, and where it
    // points is not this program's to follow — least of all when it points
    // nowhere yet, which is the case `canonicalize` cannot answer and a plain
    // write would happily create on the other side. A sheet is a file.
    if fs::symlink_metadata(&path).is_ok_and(|m| m.file_type().is_symlink()) {
        return Err(format!("{relative} is a link out of this job, not a sheet in it"));
    }

    fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path.display().to_string())
}

#[tauri::command]
fn export_write(state: State<'_, Document>, relative: String, text: String) -> Result<String, String> {
    let folder = {
        let guard = state.0.lock().map_err(|_| "the document lock is poisoned".to_string())?;
        guard.folder.clone()
    };
    write_export(folder.as_deref(), &relative, &text)
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
    // Wide enough for what is in it, which is the whole of the fix for v2 §1.9.
    //
    // A torn-off editor is the same anatomy as the main window: the editor, and
    // the Properties panel beside it. The Estimate Sheet's columns come to
    // 1136px, the actions column included, and it is torn off precisely so it
    // can have a monitor to itself. At 760 it had 440 of those 1136 once the
    // panel was beside it — and even before the panel, at 756, its pinned money
    // column sat on top of the Unit column and the word UNIT. Pinning cannot
    // fix that: a sticky cell reserves no room, it paints over whatever scrolls
    // under it, so at any width short of the table SOME column is half covered.
    // Measured against the real stylesheet: 1240 moves the wound to Order and
    // Priced (the heading read "O" and the cells "75."); 1480 leaves the
    // scroller 1160px for 1160px of table, nothing overlaps anything, and every
    // heading is its own element at its own centre.
    //
    // The main window keeps the pinned pair, because 1136 + 320 + the tree is
    // wider than a laptop screen. That is what pinning is for (D94) and it is
    // the trade this window no longer has to make.
    .inner_size(1480.0, 860.0)
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
            export_write,
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

    /// Find a condition by its id rather than its place in the array.
    ///
    /// These tests used to reach for `conditions[0]` and for a job name that had
    /// been changed, so they went red the day the demo job was edited and stayed
    /// red — through a whole section — because `pnpm -r test` runs the engine and
    /// the app and never ran these. Asserting on identity instead of on position
    /// is what stops that happening again; running them is the other half.
    fn condition<'a>(doc: &'a Value, id: &str) -> &'a Value {
        doc["conditions"]
            .as_array()
            .expect("the demo job has conditions")
            .iter()
            .find(|c| c["id"] == id)
            .unwrap_or_else(|| panic!("the demo job has a condition {id}"))
    }

    #[test]
    fn a_folder_reads_into_one_document_keyed_by_file() {
        let doc = read_folder(&demo()).unwrap();
        assert!(doc["job"]["name"].as_str().unwrap().contains("Reroof"));
        assert!(doc["costCodes"].as_array().unwrap().len() >= 6);
        assert_eq!(condition(&doc, "c-parapet")["name"], "Parapet Wall Flashing");
    }

    #[test]
    fn the_library_is_part_of_the_open_document() {
        let doc = read_folder(&demo()).unwrap();
        let assemblies = doc["library"]["assemblies"].as_array().expect("assemblies");
        assert!(!assemblies.is_empty(), "the demo library has an assembly to load");
        assert!(doc["library"]["profiles"].as_array().is_some_and(|p| !p.is_empty()));
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
        let at = doc["conditions"]
            .as_array()
            .unwrap()
            .iter()
            .position(|c| c["id"] == "c-parapet")
            .expect("the demo job has a parapet");
        let slot = doc
            .pointer_mut(&format!("/conditions/{at}/properties/H"))
            .expect("the demo condition has a height");
        *slot = json!(2.5);
        assert_eq!(doc["conditions"][at]["properties"]["H"], 2.5);
    }

    #[test]
    fn a_pointer_at_nothing_is_refused_rather_than_created() {
        let mut doc = read_folder(&demo()).unwrap();
        assert!(doc.pointer_mut("/conditions/0/properties/nonsense").is_none());
    }

    // ── writing a sheet out ────────────────────────────────────────────────
    // Every figure in these is made up. The demo job in the repository is never
    // written into by a test: a job folder a test can write to is a job folder
    // a test made, in the temporary directory, and thrown away by the next run.

    /// An empty job folder to write into, in a scratch place.
    fn somewhere_to_write(what: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("roofnerd-export-{what}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("a temporary folder");
        dir
    }

    #[test]
    fn a_sheet_lands_in_the_exports_folder_and_reads_back_unchanged() {
        let job = somewhere_to_write("lands");
        let sheet = "Item,Order,Unit\nCoping,4.50,LF\nCleat,9.00,LF\n";
        let path = write_export(Some(&job), "stocking-2026-01-02.csv", sheet).unwrap();

        assert!(
            path.ends_with("exports/stocking-2026-01-02.csv"),
            "it landed at {path}"
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), sheet, "the bytes changed on the way");
    }

    #[test]
    fn writing_the_same_sheet_twice_leaves_one_file() {
        let job = somewhere_to_write("twice");
        write_export(Some(&job), "recap-2026-01-02.csv", "Class,Cost\nMaterial,1.00\n").unwrap();
        let path = write_export(Some(&job), "recap-2026-01-02.csv", "Class,Cost\nMaterial,2.00\n").unwrap();

        let names: Vec<String> = fs::read_dir(job.join("exports"))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        assert_eq!(names, vec!["recap-2026-01-02.csv"], "the second one made a second file");
        assert!(fs::read_to_string(&path).unwrap().contains("2.00"), "the second one did not take");
    }

    #[test]
    fn a_name_that_climbs_out_of_the_exports_folder_is_refused() {
        let job = somewhere_to_write("climbs");
        let refused = write_export(Some(&job), "../sheet.csv", "Class,Cost\n");
        assert!(refused.is_err(), "it wrote to {refused:?}");
        assert!(!job.join("sheet.csv").exists(), "it landed in the job folder anyway");
    }

    #[test]
    fn a_name_that_is_a_place_of_its_own_is_refused() {
        let job = somewhere_to_write("absolute");
        let out = std::env::temp_dir().join("roofnerd-export-must-not-appear.csv");
        let _ = fs::remove_file(&out);

        let refused = write_export(Some(&job), out.to_str().unwrap(), "Class,Cost\n");
        assert!(refused.is_err(), "it wrote to {refused:?}");
        assert!(!out.exists(), "it wrote outside the job");
        assert!(write_export(Some(&job), "/etc/roofnerd-export.csv", "x").is_err());
    }

    /// A name already standing as a link is refused on being a link, not on
    /// where it leads — which is the only answer that also holds when it leads
    /// nowhere yet and a write would have created the far end.
    #[cfg(unix)]
    #[test]
    fn a_name_that_is_a_link_out_of_the_job_is_refused() {
        let job = somewhere_to_write("link");
        let out = std::env::temp_dir().join("roofnerd-export-link-target.csv");
        let _ = fs::remove_file(&out);
        fs::create_dir_all(job.join("exports")).unwrap();
        std::os::unix::fs::symlink(&out, job.join("exports/recap-2026-01-02.csv")).unwrap();

        let refused = write_export(Some(&job), "recap-2026-01-02.csv", "Class,Cost\n");
        assert!(refused.is_err(), "it wrote to {refused:?}");
        assert!(!out.exists(), "it followed the link and wrote off the job");
    }

    #[test]
    fn with_no_job_open_there_is_nowhere_to_write() {
        assert_eq!(
            write_export(None, "recap-2026-01-02.csv", "Class,Cost\n"),
            Err("no job is open".to_string())
        );
    }
}
