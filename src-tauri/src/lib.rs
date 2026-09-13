// ── The structure ──────────────────────────────────────────────────────────
// This file holds the one open job and hands the same copy to every window.
//
// A window does not own the job. It asks for it once when it opens, then it
// subscribes, and every change arrives as an event. That is the only reason two
// windows can never disagree: there is only ever one document, and it is here.
//
// Rust owns the SOLVERS and the engine owns the MONEY. That line moved on
// 2026-09-12, on Patrick's ruling that the 3D model is the function rather than
// a view: what has to get built — the tapered package, the walls and curbs with
// their flashing heights, the fastening pattern by zone — is solved once in
// `core/` (`roofnerd-core`), and the drawings, the section and the estimate are
// produced from that one result. It is the handoff README's own architecture
// sentence, adopted (D146).
//
// So: no estimating logic in this file, and none ever will be. The solvers are
// next door in `core/`, with no window and no disk anywhere near them. What money
// does with a solved number — formulas, lines, waste, prices, the recap — is
// packages/engine, which is plain arithmetic. This file holds the one open job,
// solves it when it changes, hands the same answer to every window, opens those
// windows, and touches the disk.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, LogicalSize, Manager, State, WebviewUrl, WebviewWindowBuilder, Window};

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

/// The open job: where it came from, what it currently says, and what that
/// means the roof is.
///
/// The solve lives beside the document under the one lock, so a window can never
/// read a document and a roof that disagree. `solve_id` moves on every change:
/// the windows are told it, and the ones that draw geometry come and fetch the
/// geometry. `solved` is a cache of the last answer, not a second truth — drop
/// it and the next pull works it out again.
struct OpenJob {
    folder: Option<PathBuf>,
    doc: Value,
    solve_id: u64,
    solved: Option<roofnerd_core::Solved>,
}

struct Document(Mutex<OpenJob>);

/// The inner size each window was last ASKED for, by label.
///
/// Not what it got — what it was asked for. `fit_window` corrects against this,
/// and the difference between the two is the whole reason both windows came up
/// 52 px too wide and stayed there.
struct Frames(Mutex<std::collections::HashMap<String, (f64, f64)>>);

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

/// Solve the job as it now stands, and say what to tell the windows.
///
/// The geometry is kept here; the named numbers go out with the event, because
/// they are hundreds of bytes and belong on a formula line the moment they
/// change. A solve that fails is not fatal to the estimator's edit: the change
/// stands, the windows hear about it, and the scopes go out empty — which reads
/// as pending on every line rather than as zero — while `solve_get` answers with
/// the reason so the window can say it.
fn solve_into(job: &mut OpenJob) -> Value {
    job.solve_id += 1;
    match roofnerd_core::solve(&job.doc, job.solve_id) {
        Ok(solved) => {
            let scopes = serde_json::to_value(&solved.scopes).unwrap_or_else(|_| json!({}));
            job.solved = Some(solved);
            scopes
        }
        Err(_) => {
            job.solved = None;
            json!({})
        }
    }
}

/// What every window is told when the job changes.
fn changed(document: Value, solve_id: u64, scopes: Value) -> Value {
    json!({ "document": document, "solveId": solve_id, "scopes": scopes })
}

/// Hand a window the document as it stands.
#[tauri::command]
fn doc_get(state: State<'_, Document>) -> Result<Value, String> {
    let guard = state.0.lock().map_err(|_| "the document lock is poisoned".to_string())?;
    Ok(guard.doc.clone())
}

/// The solved geometry of one part of the model.
///
/// Pulled by a window when the solve id it heard about is not the one it drew,
/// which is why this exists as a command rather than as more payload on the
/// event: the tapered boards on a real roof are hundreds of kilobytes, and a
/// keystroke must not carry them to three windows.
///
/// Every part answers with the one solve the shell is holding. A part this
/// program does not have would answer that it is not solved rather than with an
/// empty roof, which a window would draw.
#[tauri::command]
fn solve_get(state: State<'_, Document>, part: String) -> Result<Value, String> {
    let mut guard = state.0.lock().map_err(|_| "the document lock is poisoned".to_string())?;
    match part.as_str() {
        "taper" => {
            if guard.solved.is_none() {
                let solved = roofnerd_core::solve(&guard.doc, guard.solve_id)?;
                guard.solved = Some(solved);
            }
            let solved = guard.solved.as_ref().ok_or("this roof did not solve")?;
            serde_json::to_value(&solved.taper).map_err(|e| e.to_string())
        }
        // The named numbers, for a window that has just connected. They ride
        // `doc:changed` for every window that is already listening, but a window
        // opened AFTER the job was opened has heard no change yet — and before
        // this existed it priced the whole job with an empty scope, so a cost
        // panel torn off into its own window showed every derived line pending
        // and a selling price short by the whole tapered package. Money in two
        // windows disagreeing is the defect this shell exists to prevent.
        // Hundreds of bytes, so pulling them costs nothing.
        "scopes" => {
            if guard.solved.is_none() {
                let solved = roofnerd_core::solve(&guard.doc, guard.solve_id)?;
                guard.solved = Some(solved);
            }
            let solved = guard.solved.as_ref().ok_or("this roof did not solve")?;
            serde_json::to_value(&solved.scopes).map_err(|e| e.to_string())
        }
        // The drawings are built off the solved taper on the way out rather than
        // kept beside it. They are the same roof said a second way — the board
        // grid, the section through each drain — and nothing that rides a
        // keystroke needs them, so a window that opens one pays for it and a
        // window that never does pays nothing.
        "drawing" => {
            if guard.solved.is_none() {
                let solved = roofnerd_core::solve(&guard.doc, guard.solve_id)?;
                guard.solved = Some(solved);
            }
            let document = roofnerd_core::Document::read(&guard.doc)?;
            let solved = guard.solved.as_ref().ok_or("this roof did not solve")?;
            serde_json::to_value(roofnerd_core::drawing::drawings(
                &document,
                &solved.taper,
                &solved.walls,
                &solved.fastening,
            ))
                .map_err(|e| e.to_string())
        }
        // The walls and curbs: their solids, and how high the membrane may
        // climb them. C3.
        "walls" => {
            if guard.solved.is_none() {
                let solved = roofnerd_core::solve(&guard.doc, guard.solve_id)?;
                guard.solved = Some(solved);
            }
            let solved = guard.solved.as_ref().ok_or("this roof did not solve")?;
            serde_json::to_value(&solved.walls).map_err(|e| e.to_string())
        }
        // The wind zones, the count in each and the adhesive over them. C4.
        "fastening" => {
            if guard.solved.is_none() {
                let solved = roofnerd_core::solve(&guard.doc, guard.solve_id)?;
                guard.solved = Some(solved);
            }
            let solved = guard.solved.as_ref().ok_or("this roof did not solve")?;
            serde_json::to_value(&solved.fastening).map_err(|e| e.to_string())
        }
        other => Err(format!("there is no {other} in this model")),
    }
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
    let (snapshot, solve_id, scopes) = {
        let mut guard = state.0.lock().map_err(|_| "the document lock is poisoned".to_string())?;
        let slot = guard
            .doc
            .pointer_mut(&pointer)
            .ok_or_else(|| format!("nothing at {pointer} in this job"))?;
        *slot = value;
        let scopes = solve_into(&mut guard);
        (guard.doc.clone(), guard.solve_id, scopes)
    };

    app.emit("doc:changed", changed(snapshot, solve_id, scopes))
        .map_err(|e| e.to_string())
}

/// Open a job folder. Replaces whatever was open; every window follows.
#[tauri::command]
fn doc_open(app: AppHandle, state: State<'_, Document>, folder: String) -> Result<(), String> {
    let path = PathBuf::from(&folder);
    let doc = read_folder(&path)?;

    let (snapshot, solve_id, scopes) = {
        let mut guard = state.0.lock().map_err(|_| "the document lock is poisoned".to_string())?;
        guard.folder = Some(path);
        guard.doc = doc;
        let scopes = solve_into(&mut guard);
        (guard.doc.clone(), guard.solve_id, scopes)
    };

    app.emit("doc:changed", changed(snapshot, solve_id, scopes))
        .map_err(|e| e.to_string())
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

    // The cost panel is the one thing this program tears off now, and it has
    // its own size and its own name: 600 x 860, the handoff's frame for the
    // second monitor (D137). It is a second window over the same in-process
    // document, never a copy — `doc_get` and `doc:changed` are what it reads.
    let is_cost = editor == "cost";
    let title = if is_cost {
        format!("Cost — {}", app.package_info().name)
    } else {
        format!("{} — {editor}", app.package_info().name)
    };
    // An editor in a window of its own — the arrangement before the handoff's
    // anatomy — still needs room for what is in it, and the Estimate Sheet's
    // columns came to 1212px once Order, Priced and Extended were widened to
    // hold their own numbers on one line. Nothing in the program opens one of
    // these any more; the mechanism stays because it is the mechanism, and the
    // size stays with it so a label that is not "cost" is not given a window
    // too small for its own contents.
    let (width, height) = if is_cost { (600.0, 860.0) } else { (1560.0, 860.0) };
    WebviewWindowBuilder::new(
        &app,
        &editor,
        WebviewUrl::App(format!("index.html?editor={editor}").into()),
    )
    .title(title)
    .inner_size(width, height)
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Make the window's INNER size what the design asks for, from either side.
///
/// The sizes in `tauri.conf.json` and in `open_editor` are what the WINDOW is
/// asked for, and on this desktop they are not what the page gets. The page is
/// the only thing that can measure what it actually got, so it measures, says
/// how far off it is — signed: positive is too big — and this corrects it.
///
/// The correction is applied to WHAT THIS FUNCTION LAST ASKED FOR, and that is
/// the whole of this function's history. It used to add the difference to a
/// size it read back out of the toolkit, and a getter and a setter on this
/// toolkit do not count from the same edge. Write it out: let the page end up
/// `asked + c` for whatever constant `c` the toolkit takes or adds, and let the
/// getter report `page + d`. Then `getter + (want − page)` lands the page at
/// `want + d`, and asking again lands it at `want + d` for ever — a fixed point
/// 52 px wide of the frame, which is exactly what was on screen: the main
/// window's page 1492 against a 1440 design and the cost panel's 652 against
/// 600, both windows, both dimensions that were already right when the height
/// was corrected.
///
/// Against the last REQUEST there is no such fixed point. The first correction
/// has to seed itself from the getter, so it can still land wrong once; from
/// then on the base is a number this function chose, the offset cancels, and
/// the page converges on the design in one more step — whether it was too big
/// or too small. `the_window_converges_from_either_side` proves that against a
/// toolkit that adds a constant, with no 52 written down anywhere.
///
/// A dimension the page is right about is re-asked at the size it was already
/// asked for, which is a no-op rather than a nudge.
///
/// It is deliberately dumb: no target sizes live here. The anatomy is the front
/// end's, and it is the front end that knows a document bar is 44 px.
#[tauri::command]
fn fit_window(
    window: Window,
    state: State<'_, Frames>,
    over_width: f64,
    over_height: f64,
) -> Result<(), String> {
    if over_width.abs() < 1.0 && over_height.abs() < 1.0 {
        return Ok(());
    }
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let inner = window
        .inner_size()
        .map_err(|e| e.to_string())?
        .to_logical::<f64>(scale);
    let label = window.label().to_string();

    let next = {
        let mut asked = state.0.lock().map_err(|_| "the window sizes lock is poisoned".to_string())?;
        let last = asked.get(&label).copied().unwrap_or((inner.width, inner.height));
        let next = next_request(last, (over_width, over_height));
        asked.insert(label, next);
        next
    };

    window
        .set_size(LogicalSize::new(next.0, next.1))
        .map_err(|e| e.to_string())
}

/// What to ask for next: the last request, less how far the page overshot it.
///
/// A dimension the page is within a pixel of is left exactly as it was asked
/// for. Nothing in here knows what a decoration is, which is the point.
fn next_request(asked: (f64, f64), over: (f64, f64)) -> (f64, f64) {
    (
        if over.0.abs() >= 1.0 { asked.0 - over.0 } else { asked.0 },
        if over.1.abs() >= 1.0 { asked.1 - over.1 } else { asked.1 },
    )
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
            solve_id: 0,
            solved: None,
        })))
        .manage(Frames(Mutex::new(std::collections::HashMap::new())))
        .invoke_handler(tauri::generate_handler![
            doc_get,
            solve_get,
            doc_folder,
            demo_folder,
            doc_set,
            doc_open,
            doc_save,
            add_page_source,
            read_page_source,
            export_write,
            open_editor,
            fit_window
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
        assert!(doc["job"]["name"].as_str().unwrap().contains("Retrofit"));
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

    /// The window converges on the page the design asks for, from either side.
    ///
    /// A stand-in for the toolkit: it gives the page `taken` more than the size
    /// it is asked for, and its getter reports `counted` more than the page. Two
    /// separate constants on purpose — the whole defect was that they are not
    /// the same number on this desktop, and a test that assumed they were would
    /// have passed on the broken code.
    ///
    /// Returns how many corrections it took, or `None` if it never got there.
    fn corrections(taken: (f64, f64), counted: (f64, f64), opened_at: (f64, f64), want: (f64, f64)) -> Option<usize> {
        let mut page = (opened_at.0 + taken.0, opened_at.1 + taken.1);
        let mut remembered: Option<(f64, f64)> = None;
        // The page asks four times, which is what `fitTheWindow` does.
        for tries in 0..4 {
            let over = (page.0 - want.0, page.1 - want.1);
            if over.0.abs() < 1.0 && over.1.abs() < 1.0 {
                return Some(tries);
            }
            let reported = (page.0 + counted.0, page.1 + counted.1);
            let next = next_request(remembered.unwrap_or(reported), over);
            remembered = Some(next);
            page = (next.0 + taken.0, next.1 + taken.1);
        }
        None
    }

    #[test]
    fn the_window_converges_from_either_side() {
        let want = (1440.0, 900.0);
        // This desktop, as the probes found it: the height comes up 47 px short
        // and the width is counted 52 px wider than the page. Both windows had
        // stuck at want + 52 wide.
        assert_eq!(corrections((0.0, -47.0), (52.0, 0.0), (1440.0, 900.0), want), Some(2));
        // Too wide to begin with — the state the program was actually left in.
        assert_eq!(corrections((0.0, 0.0), (52.0, 0.0), (1492.0, 900.0), want), Some(2));
        // Too small in both dimensions, which is the case that used to work.
        // Two again: the first correction is spent learning what this toolkit
        // does with a request, because the only base it has to start from is a
        // number the toolkit gave it.
        assert_eq!(corrections((-30.0, -47.0), (0.0, 0.0), (1440.0, 900.0), want), Some(2));
        // A toolkit that hands over exactly what it was asked for needs nothing.
        assert_eq!(corrections((0.0, 0.0), (0.0, 0.0), (1440.0, 900.0), want), Some(0));
        // And the cost panel, whose frame is its own (D137).
        assert_eq!(corrections((0.0, -47.0), (52.0, 0.0), (600.0, 860.0), (600.0, 860.0)), Some(2));
        // The size it was stuck at is not a fixed point any more: 1492 wide is
        // corrected, and 1440 wide is then left there however often it is asked.
        assert_eq!(corrections((0.0, 0.0), (52.0, 0.0), (1440.0, 900.0), want), Some(0));
    }

    /// A dimension the page is right about is asked for at the size it was
    /// already asked for. Correcting a height must not move a width.
    #[test]
    fn a_dimension_that_is_right_is_left_alone() {
        assert_eq!(next_request((1440.0, 853.0), (0.0, -47.0)), (1440.0, 900.0));
        assert_eq!(next_request((1440.0, 900.0), (52.0, 0.0)), (1388.0, 900.0));
        assert_eq!(next_request((1440.0, 900.0), (0.4, -0.6)), (1440.0, 900.0));
    }
}
