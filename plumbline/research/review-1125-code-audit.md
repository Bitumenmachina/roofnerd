# Code audit, read only, tree wtA/plumbline (nothing was built or run)
Paths below are relative to plumbline/. Counts come from grep/awk over non-test code (text before the first `#[cfg(test)]`, files not named *test*, not under tests/) unless said.

## 1. Shape
Lines of .rs: world 61,103 (6,712 of them one test file, crates/world/tests/world.rs) / app 17,670 / plans 13,024 / roofing 11,594 / ifc 6,340 / cli 6,306 / sheets 562; total 116,599. 7 crates. 46 src files carry a `#[cfg(test)]` block.
Largest files: world/src/building.rs 3,943; cli/src/main.rs 3,918; app/src/panels.rs 3,704; plans/src/found.rs 2,930; ifc/src/lib.rs 2,843; app/src/scene.rs 2,694; world/src/editing.rs 2,627; app/src/tools.rs 2,317; plans/src/drawn.rs 2,330; world/src/tables.rs 2,146.
Twenty longest functions (awk brace-match, ±1 line):
1 ifc/src/lib.rs:1132 `bring` 1,676 lines (59% of the file)
2 plans/src/drawn.rs:1126 `find_core` 1,015
3 plans/src/found.rs:1431 `find_core` 966
4 app/src/scene.rs:579 `rebuild` 918 (the whole 3D scene in one system)
5 cli/src/main.rs:1928 `run` 861 (verb dispatch)
6 app/src/main.rs:492 `keys` 826 (all keyboard handling)
7 world/src/building.rs:965 `measures` 708
8 ifc/src/write.rs:329 `export` 688
9 cli/src/main.rs:3176 `draw` 566
10 app/src/sheet.rs:945 `estimate` 525
11 roofing/src/drawing.rs:936 `section_of` 448
12 cli/src/main.rs:1236 `tables` 421
13 world/src/building.rs:2489 `room_measures` 413
14 world/src/file.rs:265 `load` 411
15 world/src/plan_set.rs:1303 `plan_prims` 402
16 world/src/tables.rs:244 `prices_in` 362
17 world/src/derive.rs:72 `derive` 361
18 world/src/building/clash.rs:830 `find` 360
19 world/src/sample.rs:226 `house` 344
20 world/src/vif.rs:129 `vif` 312
(then editing/measure.rs:341 `put` 310; trim/lay.rs:327 `lay` 307; app/model.rs:849 `refresh` 301; app/tools.rs:733 `pick` 284; app/map.rs:34 `write` 277.) 29 functions are over 250 lines.
Files doing more than one job: cli/src/main.rs (argument parsing, ~50 verbs, tables, drawing, printing, 3,918 lines in one file); app/src/panels.rs (every panel, file picker at :705-760, sun card, scroll physics at :3587-3660, label placement :3476-3500); app/src/main.rs (setup, keys, laser detection, save, close); plans/src/found*.rs (file names are call numbers: found_179/183/188/189, tests as found_183_tests.rs); world/tests has 38 files named callNNN.rs (call numbers mean nothing to a newcomer); 1,251 "call NNN" references in .rs comments.
Model/window split: mostly clean. The window formats model figures (`view::so_many`, `view::figure`, `panels.rs:122-137 total`) and says "nothing is added up here" (sheet.rs:693). Places where the window works out something itself:
- app/src/plans.rs:582-615 `bare_length` (geometry: length of a stretch not covered by plan faces), called from app/src/tools.rs:1281 to decide a wall's gap. Model-level geometry in the window crate.
- app/src/panels.rs:437 `floor_note`: `(f - r).abs() > 0.05` compares floor SF to room SF itself and decides the note "out to the walls' center lines".
- app/src/sheet.rs:615 `by * 12.0` feet to inches by hand (units.rs has the converters); sheet.rs:1507 chooses "over/under/even" from sign.
- app/src/panels.rs:2039-2090 groups runs and picks the longest to tag (label choice, not a figure).
- app/src/main.rs:578, panels.rs:314, scene.rs:1413 centroids by hand (three copies of mean of points).
- app/src/panels.rs:1030 "{done} of {} done", sheet.rs:319/336/403/1204 row counts shown as table counts, panels.rs:3173 `totals.surface.missing.len()` ("N of M planes have no area"), sheet.rs:376 same: counts of list lengths, taken from model lists, not recomputed totals.
- No dollar, area or length total is summed in app (grep for sum/fold/+= on cost/price/area found only the centroid and `lowest` base folds at main.rs:502, scene.rs:1578).
Verdict: the "one arithmetic" rule holds for money and quantities. The exceptions are small geometry/threshold decisions above.

## 2. Hygiene (non-test code)
unsafe: 0 in all crates (grep 'unsafe' crates, 0 hits).
unwrap(): 10 total, all plans/src/drawn.rs (lines 1268,1287,1383,1384,1466,1594 on slices known non-empty). 0 in world, app, cli, ifc, roofing, sheets.
expect(): world 111 (95 in world/src/sample.rs, the made-up sample house builder, 9 in file.rs on serialising our own types, 7 invariants: view.rs:129,130 "every element has a node/kind", derive.rs:191, spec.rs:278, tables.rs:290,318, export.rs:432), plans 11 (all in found_179.rs, a test file by name), ifc 8 (step.rs: a method called `expect`, not Option::expect, so 0 panics), roofing 2, app 3 (scene.rs:240 and :271 mesh builds, main.rs:235 `camera.single().expect("the 3D camera is there")`).
panic!: world 4 (building/forms.rs:207, shape.rs:451,457,469: a starter shape table that cannot be read), plans 6 (found_179.rs, tests), app 1 (app/src/bench.rs:407 `other => panic!("not a box")`). todo!/unimplemented!: 0. TODO/FIXME/XXX/HACK in .rs: 0.
Panics a user could reach (worst, none confirmed by running):
1 app/src/panels.rs:2819 `name[..1].to_uppercase()` on `s.name`, a job shape name the person typed (called at :2822 `cap(&s.name)`): a name starting with a multi-byte character, or empty, panics when the item panel draws.
2 app/src/tools.rs:1992, :2021 and app/src/main.rs:369 `went[..1]` on a sentence from the model: panics if it is empty or starts with non-ASCII.
3 world/src/building/shape.rs:451,457,469 and forms.rs:207: panic if a starter shape row is wrong (embedded data, fixed at build).
4 app/src/main.rs:235 `.single().expect` on the camera.
5 plans/src/drawn.rs:1268,1287,1383,1384,1594 `.unwrap()` / `.last().unwrap()` on slices assumed non-empty after a PDF is read (a hostile or odd PDF is the input).
6 app/src/scene.rs:240 `.expect` on mesh generation.
7 app/src/tools.rs:1656 `doc.beside.remove(which)` index from UI state.
8 app/src/tools.rs:1720 `words.remove(0)`.
9 world/src/file.rs:71 "every element has a node" expect on loading a job (loader of a hand-edited file).
10 world/src/tables.rs:290, :318 `how.as_ref().expect(...)`.
(That is ten, not twenty: I found no more on a file read or a typed parse. File reads are `map_err`'d everywhere I looked: app/main.rs:134, tools.rs:1519, plans.rs:280,335, bring.rs:119,406. Index `[0]`/`[1]` appear 356 times in app/src; I did not audit each.)
`#[allow(`: 29 in non-test code, 28 are `clippy::too_many_arguments` (app: scene.rs:414,575,1967,2125; sheet.rs:179; main.rs:491; map.rs:33; dress.rs:643,669; world: building.rs:2297,2450,2461,2472, editing.rs:562,1194, job.rs:524,618, plans.rs:325, plan_set.rs:963, runs.rs:42, lay.rs:310, measure.rs:340; roofing/sump.rs:55; ifc/lib.rs:1131; plans/found_scenes.rs:121) and `type_complexity` x4 (map.rs:33, building.rs:3835, derive.rs:256). Functions of 8+ arguments are a sign the Bevy systems and ifc `bring(job, r, name, how, unit_name, k, only_runs, extras)` want a struct.
Commented-out code: 0 found by pattern (`// let`, `// foo(...);`).
Duplicated helpers: the same path-from-folder function in two crates: world/src/plans.rs:302 `path_from` and app/src/plans.rs:363 `from_folder` (identical body, comment at world says "one copy, for the command line and the window", call 153). Vector helpers `sub` written 7 times (app/plans.rs:493, plans/found.rs:2607, found_183.rs:36, drawn.rs:268, world/editing/stretch.rs:307, world/src/building/connections.rs:500 (as listed by grep: editing/connections.rs:500), skeleton.rs:46) and `dot` 8 times (app/plans.rs:496, ifc/lib.rs:257, plans/found_188.rs:18, found.rs:578, drawn.rs:271, stretch.rs:313, skeleton.rs:58). Sample of unused public items: none found among world/trim/lay.rs, carries.rs, derive.rs (every pub fn was named elsewhere); not exhaustive.
Dependencies: 563 packages in Cargo.lock; Cargo.lock committed; rust-toolchain.toml pins 1.97.1; workspace rust-version 1.95, app 1.97.1. Duplicates at two versions (`cargo tree -d`): 22 crates, e.g. hashbrown 0.15/0.16/0.17, itertools 0.11/0.14/0.15, rustix 0.38/1.1, thiserror 1/2, syn 2/3, rstar 0.12/0.13, read-fonts/skrifa two each, fearless_simd 0.7/1.2, rustc-hash 1/2, miniz_oxide, memchr.
Bevy: `=0.20.1` pinned exactly, default-features = false, features (app/Cargo.toml:17-28): default_app, std, multi_threaded, bevy_winit, x11, common_api, bevy_light, bevy_render, bevy_core_pipeline, bevy_pbr, bevy_gizmos_render, bevy_picking, mesh_picking, ui_picking, bevy_ui, bevy_ui_render, bevy_input_focus, default_font, ktx2, zstd_rust, tonemapping_luts, bevy_anti_alias, bevy_post_process, hdr, dfg_lut, bluenoise_texture. No wayland, no audio, no gamepad, no gl/webgl.
Native libraries: no -sys crate needing a system library on Linux. Present in the lock but platform-specific: windows-sys, core-foundation-sys, objc-sys, ndk-sys, android_log-sys, jni-sys, renderdoc-sys, js-sys/web-sys. Linux libraries are loaded at run time by name (libloading, x11-dl, xkbcommon-dl, ash): libX11, libxkbcommon-x11, libXcursor, libXi, a Vulkan loader (README :19-23). world tree: 209 lines, `bevy_ecs`, `bevy_tasks`, `bevy_platform`, `parry3d-f64`, `rayon`, no -sys.
Tokio is in Cargo.lock but `cargo tree -i tokio` finds it in no host-target tree. No http/net crate for the host target; no `std::net` in crates (grep).

## 3. Other machines
cfg(target_os/unix/windows): none in non-test code (grep). Only crates/cli/tests/cli.rs:5 `#[cfg(unix)]` + `std::os::unix::ffi::OsStrExt` (test).
Command::new: only in tests (cli/tests/*.rs). The program starts no outside program: no PDF tool, no font tool, no browser. Good.
Environment variables read: PLUMBLINE_GRAPHICS (main.rs:87; "plain" = no AO/TAA, 4x MSAA, scene.rs:283-286), PLUMBLINE_FOG (scene.rs:277), PLUMBLINE_RUN_DOTS (scene.rs:531), PLUMBLINE_TRACE (scene.rs:2270,2320), PLUMBLINE_SHOTS (main.rs:519), PLUMBLINE_MAP (map.rs:46). No HOME, no TEMP. No hard-coded /tmp, /home, /usr in non-test code (grep). Demonstration scripts use PLUMBLINE_BIN.
Fonts: bundled. app/src/theme.rs:209-211 `include_bytes!("../assets/fonts/Inter-{Regular,Medium,SemiBold}.otf")`; nothing is expected on the machine. (Bevy `default_font` is also on.)
File picking: app/src/panels.rs:705-760 `pick_list` lists `std::fs::read_dir` of the job's folder, folders first, "Up one" via `here.parent()`; hides names starting with '.'; filters by lowercase extension. Starts at the job's folder, not at a home or documents folder, and the top row goes up to the file-system root with no drive list. Works on Windows/macOS in principle (read_dir/parent), not tested; Windows hidden attribute and drive letters are not handled. app/src/tools.rs:1538 `jobs_here` lists *.json in the job's folder (any .json is opened and parsed to see if it is a job). Drag and drop of a file is also taken: plans.rs:381.
Keys: F2 saves (main.rs:602,1314) and Ctrl+S (main.rs:602); the Save button's own label says "Ctrl+S" (panels.rs:511). Only ControlLeft/ControlRight are read (main.rs:535,618; scene.rs:2360); no Super/Command anywhere (grep Super: 0). On a Mac Cmd+S and Cmd+Z do nothing. F12 takes a picture (main.rs:518) into PLUMBLINE_SHOTS; it is a test aid, with no menu route; on a Mac laptop F-keys need fn. Right mouse button orbits in 3D (scene.rs:2207) so a one-button trackpad needs ctrl-click.
Line endings/path separators: job JSON is written with "\n" only (file.rs:769-805, hand-written pretty printer); CSV terminator is `\n` (sheets/src/lib.rs:308), not RFC 4180's CRLF although the code comment says RFC 4180 (Cargo.toml comment on csv). The saved job keeps a drawing file path built with `PathBuf::push` and `to_str()` (app/src/plans.rs:370-376, world/src/plans.rs:316-322): on Windows that writes `..\plans\a.pdf` with backslashes into the job; the same job on Linux/macOS would not find it. Plans are read relative to the job's folder (plans.rs:334). `canonicalize` on Windows returns `\\?\` paths: components compare fine but the result is untested.
Clock: world/src/job.rs:101 `SystemTime::now()` for the job heading (UTC; no locale). Panics on wasm32-unknown-unknown.
Graphics: wgpu 30 with features vulkan, dx12, metal, wgsl (cargo tree -e features -i wgpu); no `gl` so no OpenGL fallback for an old card; no WgpuSettings / backend choice / power preference in code (grep `WgpuSettings|backends|RenderCreation` = 0). Fallback when no GPU: none coded; it relies on Mesa software Vulkan (README says so). Weak-card settings: one switch, env PLUMBLINE_GRAPHICS=plain (main.rs:87). Not changeable: shadow map fixed 4096 (scene.rs:241 `DirectionalLightShadowMap { size: 4096 }`), cascades, SSAO + temporal AA always on in full mode (scene.rs:283), HDR on, no render-scale, no vsync/present mode setting, no in-window quality control.
Frame rate: motion is by time. `time.delta_secs()` in scene.rs:1640 (ease 1-exp(-dt*9)), 2191 (dt clamped to 0.1 s), 2301, 2364; panels.rs:3634 scroll ease 18/s. Double-click uses elapsed time with a window `(delta*1.5).max(0.5)` (scene.rs:2225-2228). Left-button count per frame handled for slow frames (scene.rs:2130). So not frame-count driven.
High DPI: UI sizes are `px()` logical (Bevy scales by the window's scale factor); no UiScale and no explicit scale_factor read in app code (grep scale_factor: only `inverse_scale_factor` for map/labels, and `ImageRenderTarget { scale_factor: 1.0 }` at glass.rs:73). Frost blur target is sized once at startup: glass.rs:69 `window.physical_width()/SMALLER` (SMALLER = 6) with no resize handler (grep WindowResized/resize in app: none). Whether it looks right after a window resize or a move between a 1x and a 2x monitor is not verified.
Absolute pixels: theme.rs and panels.rs use fixed logical sizes throughout (LEFT_W 252, RIGHT_W 344 at scene.rs:39-40, GAP 10, buttons 52x44 at panels.rs:1657, cells 90-170 wide at sheet.rs:1498-1520). Window sizes are fixed in code: model 1600x960 at (0,0) (main.rs:91), sheet 880x960 at (1610,0) (main.rs:238). On a 1920x1080 screen the sheet window opens partly off screen (1610+880=2490); on 1366x768 the model window is larger than the screen. No min-size constraint (resize_constraints: 0). The two side panels take 596 + gaps of a 1600 window; panels use min_width 0 + overflow clip (panels.rs:502,600,754) so narrow text clips, not wraps. Smallest size the layout survives: not measured; every demo runs at 2500x980 (README, STATUS).
Two windows: model window is Primary; the estimate sheet is a second OS `Window` entity with its own Camera2d and layer (main.rs:238-239), sharing one `Doc` resource. On a tablet or in a browser there is one surface; second window cannot be opened (winit on wasm/iOS/Android has one).
README install line (README.md:14-23): Ubuntu 24.04 only: build-essential pkg-config curl git libx11-dev libxcursor1 libxi6 libxkbcommon-x11-0 mesa-vulkan-drivers. Missing: anything for Windows (MSVC build tools, a Vulkan/DX12 driver, no libs to install) and macOS (Xcode command line tools; Metal is used), Fedora/Arch names, Wayland note (window is X11 via XWayland, README says so in Cargo.toml comment only), the rustup install on Windows. The README's clone line names a branch `claude/nifty-tesla-qmn44n`.

## 4. Remote desktop
Pointer: absolute. The code reads `window.cursor_position()` each frame and derives motion from the last position (scene.rs:2136-2149, comment: "a pointer that is put somewhere (a tablet, a remote desktop, a test rig) sends no 'moved by'"). No MouseMotion / raw motion reads (grep 0). No cursor grab, no CursorGrabMode, no cursor hide (grep `CursorGrab|CursorOptions` = 0). Good for remote desktop. Cost: on foot (Walk) the right-drag look uses `moved` without grab (scene.rs:2259-2262), so the pointer hits the screen edge and stops turning.
Wheel: handled both ways: scene.rs:2326 `if scroll.unit == Pixel { delta.y / 100 } else { delta.y }` clamped to +-3 notches; panels.rs:3631 Pixel as is, Line x48. Remote desktops that send many small pixel deltas are handled by AccumulatedMouseScroll; the clamp +-3 per frame limits a fast flick.
Double-click: time-based, half a second at least (scene.rs:2225-2231); tolerant of a slow frame. Drag threshold: 5 logical px (scene.rs:2250 `feel.gone > 5.0`; :1764 `drag.distance.length() > 5.0`); move tool 8 px (tools.rs:2122). Fixed pixels, not scaled by DPI.
Hover-only: `doc.hover` from PointerOver sets the highlighted/pointed thing (scene.rs:504-507) and click picking uses the hover map (scene.rs:2154); no controls exist only on hover that I found, but a tap with no prior move gives no hover. Clipboard: grep clipboard = 0 (no copy/paste of a value into a field from the system clipboard; a laser typed as keys is detected by timing, main.rs:~523-535). Second monitor: the sheet window defaults to x=1610 so it wants a second monitor or a large screen; with it off-screen the estimate is unreachable unless the window is dragged.

## 5. Field app later
bevy_ecs only in world (Cargo.toml: `bevy_ecs` default-features off, "std"), parry3d-f64, serde, serde_json, flate2, plumbline-roofing. No -sys crate. bevy_tasks and rayon are in the tree. Not built for wasm/iOS/Android here: I cannot say it compiles, only that no windowing, graphics, or native-library crate is in its tree.
Non-portable spots in world: job.rs:101 SystemTime::now (fails at run time on wasm32-unknown-unknown); plans.rs:305-329 writes a PNG with std::fs and canonicalize (the only file write in world, a layer mixing model and disk); plans/src/found.rs:2925 `Instant::now` (timing, a plans crate). No home folder, no temp folder, no locale use found.
Job read/write: text in/out is one module (file.rs `save(&mut Job) -> String`, `load(&str)`), but the file system call is made in 8 places in app: main.rs:134 (read), :436 (write), tools.rs:1519,1544,1610, bring.rs:278,281, plans.rs:280,335, panels.rs:704,721, map.rs:306, plus cli. Imports (CSV/XLSX/IFC/PDF) take bytes or text, not paths, in the crates (good), the app reads the bytes.
Desktop assumptions in app: mouse only. `Touch` handled nowhere (grep Touch/touch: only `doc.touch()` method). Right button orbit, middle button pan, wheel zoom, hover highlight, Ctrl, F2/F12, Shift. No on-screen keyboard path: values typed by key events (tools.rs entry); a tablet needs a soft keyboard (Bevy text input absent). UI toolkit: bevy_ui (Node, flex) with bevy_text/cosmic-text/swash; text drawn by bevy_ui_render; a custom frost shader at crates/app/src/frost.wesl. Layout is mixed: flex columns for panels with fixed widths (252/344), floating labels placed by absolute coordinates each frame (panels.rs:2139-2150 `left: px(-2000)` + UiTransform), a hand-written control map for tests (map.rs, 310 lines) that depends on node positions.
One surface with tabs: possible in principle because both windows read the same `Doc` and have separate camera/layer (main.rs:238-240) and sheet.rs builds its own flex root; it would need the sheet root moved under the main camera's UI target and the 880 px assumptions in sheet.rs relaxed. No code does this today.

## 6. Tests and gates
#[test] counts (crates, including tests/ dirs): world 667, plans 172, roofing 122, ifc 73, app 48, cli 45, sheets 6 = 1,133 (STATUS says 1,043 passed at a40b968, later commits added more). Demonstration scripts: 101 in demos/*.sh; 42 end in `w.sh` (window); 43 use the window kit (xdotool/open_window); the rest (about 58) drive the command line. tools/: committed.sh, installs.sh, licenses.sh, portmap.sh, fetch-plans.sh, landscape.sh, made-up-*.py. Repository-root tools/: gate-language.sh (words; covers plumbline/crates/app/src at :77), gate-egress.sh and gate-clientdata.sh.
Gate coverage: tools/gate-egress.sh scans only `packages/engine/src packages/engine/bin packages/app/src packages/app/index.html` (line 17 WHERE): the earlier TypeScript program. It does not look at plumbline/crates, so "the program never opens a connection" is not enforced by that gate for the Rust code (my own grep for std::net/TcpStream/reqwest/ureq found 0 in crates, and no http crate in the lock for the host target). gate-clientdata.sh --all reads every tracked file (covers plumbline). committed.sh builds the last commit in a scratch folder.
In-process input: none that I found. The window is driven only by xdotool on Xvfb (README :55). In-process hooks that exist: the control map (map.rs: names, positions of every control written to a file when PLUMBLINE_MAP is set), F12 screenshots, `doc.press(KeyCode)` used by buttons (panels.rs:511), `Doc` is a plain resource with `say/edit/press` methods, and pure functions (tools.rs entry parsing, panels `step_hour`, `floor_note`) with 48 unit tests in app. Bevy supports sending `KeyboardInput`/`CursorMoved`/`MouseButtonInput` messages into the App from a test, which is how a headless, cross-platform driver could be written; nothing like it exists in the repo.
Pixel/threshold checks in window scripts (fixed numbers, renderer-dependent): glassw wheel ratio 1.74 to 2.02 (STATUS "Start here": has read 1.88, 2.02, 2.04); trimw door region 80 dots (read 129 once, 64 alone); benchw.sh:195 `<= 2.0` dots; benchw.sh:239 mean difference and `> 24` of 255; cabinetsw.sh:236-362 luminance cuts (110, 235, 215, 180) with counts (>= 6 pixels); carryw.sh:106-107 `< 0.6` colour fraction; buildw.sh:37 corner positions (539,653),(1061,653) at 2500x980. Screen coordinates assume fonts, scale and the software renderer of this one machine.

## 7. The record
DECISIONS.md 9,062 lines; STATUS.md 11,726 lines; README.md 444. The last section of STATUS.md ("Start here", :11453) is the entry point and it is itself a long bulleted status page, and is dated 2026-11-25 06:00 with "older below". No single short current page: README.md:1-12 says what it is but its Run it list (README :80-170) is 60 demonstration lines, and "Install on Linux" is the only install. CLAUDE.md is the real map. TRY-IT.md exists at root of plumbline/ (not read).
README vs code: ten verbs the README uses (sample, house, sixteen, takeoff, clashes, price, show, set, new, estimate, vif) all exist in crates/cli/src/main.rs (matched at :2004,2021,2090,2115,2162,2355,2366,2571,3220). All 90 demonstration/tool scripts the README names exist (loop over names found none missing). One README line is stale: `cargo test # the world-model checks` (it runs the whole workspace). Two README entries listed twice (findwalls.sh, planwalls.sh).
Old name: 105 string literals in app/cli sources contain "plumbline"; 48 lines in app+cli .rs use the word outside a crate path (`use plumbline_world`). A person sees: CLI error prefix `plumbline: ...` (cli/main.rs:48,56), `plumbline-app: {why}` (app/main.rs:83), the job's author field written as "plumbline" (app/main.rs:132, cli/main.rs:3226, world/plans.rs:673), the two commands `plumbline`/`plumbline-app`, the folder `plumbline/`, 137 occurrences in cli/src/main.rs of "plumbline" in help/usage lines (grep -c). Window titles say "BuildOS: model" / "BuildOS: sheet" (main.rs:91,238). Env vars are PLUMBLINE_*.

## Lists
(a) Before it runs on another machine
1 app/src/main.rs:91,238 window 1600x960 at (0,0) and 880x960 at (1610,0): size/position by fixed pixels; sheet falls off screens narrower than 2490.
2 app/src/plans.rs:370-376 + world/src/plans.rs:316-322 backslash paths written into the job on Windows (job not portable between OSes).
3 app/src/main.rs:535,602,618 + scene.rs:2360: only Ctrl; Cmd+S/Z/Y do nothing on a Mac.
4 app/Cargo.toml:17-28 / wgpu features: no GL fallback; no weak-GPU path beyond PLUMBLINE_GRAPHICS=plain (main.rs:87); shadow map 4096 fixed (scene.rs:241).
5 app/src/glass.rs:69 frost texture sized once at startup, no resize handler.
6 README.md:14-23 only Ubuntu steps; Windows/macOS need are unwritten.
7 app/src/panels.rs:2819 `name[..1]` panic on a typed non-ASCII or empty shape name; tools.rs:1992,2021, main.rs:369 same pattern.
8 app/src/scene.rs:2259 walk-mode look with no cursor grab (pointer stops at the edge, bad on remote desktop).
(b) For clean code
1 ifc/src/lib.rs:1132 `bring` 1,676 lines; plans/drawn.rs:1126 and found.rs:1431 `find_core` ~1,000 lines each; app/scene.rs:579 `rebuild` 918; app/main.rs:492 `keys` 826; cli/main.rs:1928 `run` 861.
2 cli/src/main.rs 3,918 lines, one file for ~50 verbs; panels.rs 3,704; building.rs 3,943.
3 world/src/plans.rs:302 vs app/src/plans.rs:363 duplicate; `sub` x7, `dot` x8 vector helpers.
4 app/src/plans.rs:582 `bare_length`, panels.rs:437 `floor_note`, sheet.rs:615: model arithmetic in the window.
5 28 `too_many_arguments` allows (scene.rs:414,575,1967,2125; ifc/lib.rs:1131; building.rs:2297-2472).
6 File names by call number (plans/src/found_179.rs; world/tests/call157.rs ... 38 files); 1,251 "call NNN" comments.
7 tools/gate-egress.sh:17 scans the old program, not crates/.
8 world/src/sample.rs 95 `.expect(OK)` in a 344-line function used by the CLI `house`.
(c) Sound; leave alone
- 0 unsafe, 0 todo!/unimplemented!, 0 TODO/FIXME, 0 unwrap in world/app/cli; file errors are returned, not panicked.
- Program starts no outside process (no Command::new outside tests); reads no env except PLUMBLINE_*; no network crate.
- Fonts bundled (theme.rs:209-211); Cargo.lock and toolchain pinned (rust-toolchain.toml 1.97.1; bevy `=0.20.1`).
- Pointer read as absolute position, wheel in pixel and line units, time-based easing (scene.rs:2136-2149, 2326, 1640).
- No audio/gamepad/wayland/-sys libraries needed to build; Linux runtime libs loaded by name and listed by tools/installs.sh.
- Job text in/out is one module (world/src/file.rs); imports take bytes/text not paths in the crates.
- Money/quantity totals come from the model (`view::Sum`, `vif`, sheet.rs:693 note).
- 1,133 tests, portmap/licenses/committed gates.

## Five changes for a field app, smallest first
1 Replace `SystemTime::now` (world/src/job.rs:101) and the PNG write in world/src/plans.rs:329 with an injected clock and a returned byte buffer: world then touches neither clock nor disk.
2 One file-system seam: a small trait or module in app for read/write/list used by main.rs:134,436, tools.rs:1519-1610, bring.rs, plans.rs, panels.rs:721, and store drawing paths with '/' always.
3 Make the layout read the window size (min size, panels collapse to tabs below a width), drop the fixed 1600x960 / 1610,0 positions, and move the estimate sheet under the main window's UI as a tab (sheet.rs root, main.rs:238).
4 Add a pointer/gesture layer: one input module that maps mouse, touch (one finger drag = slide, two-finger = orbit/zoom, tap = pick/double-tap = fly) and keyboard onto the same camera commands, replacing the direct button reads in scene.rs:2195-2330 and tools.rs:2115-2170; add Cmd as an alternate to Ctrl.
5 Split the window crate so the decisions sit in testable functions (keys at main.rs:492 and rebuild at scene.rs:579) and feed events in-process (Bevy messages) so the window demonstrations stop depending on xdotool/Xvfb and pixel counts.
