# Window behaviours (read-only review; paths under crates/app/src unless noted; world = crates/world/src)

PLACING
1 Ghost before click: NO for all five. tools.rs:2204 `preview` draws only the points picked, a line from the last pick to the pointer, and a mark (cross if free, square if snapped). Nothing of a wall's thickness, door, fixture, run section or cabinet is drawn. An item "in hand" (panels.rs:1508 `doc.placing`) is a message only: "X in hand: click where one stands." scene.rs:1976 sets it down on click. Wall tool over a scan draws faint face lines (tools.rs:2214).
2 Allowed/clash before click: NO. Clash words come only after: scene.rs:2010 "Set one down on ... It is in the same space as ..."
3 Turn: before set down NO. After: PARTLY. Q or "Turn a quarter" (main.rs:1107, panels.rs:2524) turns a selected fixture/equipment/furnishing 90 degrees only; says if it clashes. No free angle in the window (world editing::turn takes degrees).
4 Typed size/length: YES for typing, PARTLY for display. Digit with a start picked opens "Length" (main.rs:943; hint "Toward the pointer, like 24 ft or 12' 6\". Enter sets it." panels.rs:1805). Sizes (thickness, height AFF) are typed in the tool's fields before drawing. The length of the rubber-band as the mouse moves is NOT shown; the status line shows only E/N coordinates (panels.rs:3522).
5 Angle/slope at pointer: NO text. 15-degree angle snap exists (snap.rs Snaps::default angle 15, name "On the angle"); roof pitch is typed, not read off the pointer.

SNAPPING
6 Kinds (world/editing/snap.rs:49): Endpoint "On an end", Midpoint "On a middle", Intersection "Where two cross", Centre "On a center", Perpendicular "Square to it", Nearest "On a line", Angle "On the angle", Free. Shown: square mark vs cross at the pointer (tools.rs:2228) and the name beside the coordinates in the status line (panels.rs:3525). Switch off: NO, no key or switch for any kind (`Snaps` is built in tools.rs:2099-2107 per tool; Door/Window use Nearest only; Floor/Room/Cabinets none). Only Shift holds a point square to the last (tools.rs:2095). Reach is 10 px to a point, 8 px to a line.
7 Grid: PARTLY. A ground grid every 10 ft, heavier every 50 (scene.rs:2388), fixed and decorative; no grid snap, step not changeable.

MOVING AND COPYING
8 Move: YES, two ways. (a) Press and carry a fixture, equipment or furnishing in 3D or plan (scene.rs:1968 `carry`, `stands` = Equipment|Fixture|Furnishing): clashes recomputed each frame ("in the same space as ... Keep going" scene.rs:2085), clash marks switched on while carried, one undo step. (b) Move tool (tools.rs:972): click point then destination, or press-drag-release; wall (takes connected walls, floor edge, footing), run, fitting, room, room line. Clashes are said only after the drop for (b). Doors/windows: NO (world editing.rs `move_by` has no Opening branch: "not moved this way").
9 Copy/duplicate/array: NO (grep duplicate|copy|array|paste in app: none about things). Eyedropper: NO. (Only "Keep as a new shape" copies an item's shape, panels.rs:2924.)
10 Multi-select: PARTLY. Shift-click adds/removes (scene.rs:490, model.rs:611); "select all walls on this level" (tools.rs:1950). Only WORK status and Delete act on all (panels.rs:3282). No box drag; Move/Turn act on one.

REUSE
11 Save a group/blueprint: NO. "Layouts" means alternative whole jobs (job A vs B) set side by side on the sheet (tools.rs:1516, Tab::Layouts), not reusable groups. Only single items (Items list, library/items.csv) are reusable.
12 Tool chest: no search box, no filter, no favourites/recent (grep search|filter|recent in panels.rs: none relevant). Items are a list grouped by trade division (panels.rs:1485).

FINDING
13 Command palette: NO. Shortcut list in window: NO (no Help/F1). Keys ARE shown on buttons: tool buttons show 1-0 and Del (panels.rs:1835), Undo "Ctrl+Z", Save "Ctrl+S", Plan "T", Walk "V", Frame "F". Many keys (Q, L, B, U, N, O, R, P, M, G, X, C, Tab, PageUp/Down) are on panel buttons only partly.
14 Search job by name and fly: NO search. Fly-to exists only by clicking a row in the Verify list (sheet.rs:739 `look_at`) and double-click on the model.

CAMERA (scene.rs)
15 Wheel zooms toward point under pointer YES (2347-2370; plan uses pointer-pinned zoom too). Drag with left (no tool) or middle slides ground, place grabbed stays under pointer YES. Orbit: right button about the place under pointer (2207); on plan right slides. Momentum YES: slide and spin fade `exp(-dt*2.5)` (2318). Double-click flies and closes to 0.45x distance YES (2232). Frame whole house: F (main.rs:1197). Plan view/north-up: T toggles; plan is always north-up (place_camera yaw 0). PageUp/Down cycles levels. Walk: V; W/A/S/D, Shift faster, right-drag looks, E/Q up/down. Arrow keys turn/tilt. No key frames the selection.
16 Easing: by clock. place_camera:1640 `ease = 1 - exp(-dt*9)` (about 0.11 s time constant). No fixed-duration flights; a double-click sets the target and the ease follows.

UNDO AND SAFETY
17 Undo/redo: YES, Ctrl+Z/Ctrl+Y/Z/Y and buttons; one whole-job snapshot stack (tools.rs:2188 `record`), UNDO_LIMIT 200 (world/editing/history.rs:30). Steps are labelled, but no visible history list. Closing with unsaved work asks once ("Not closed: the job has changes that are not saved..." main.rs:425); a "Not saved" chip shows.
18 Autosave/recovery: NO (grep autosave|recover: none). Save is F2/Ctrl+S only.

TEACHING
19 YES, in-window: left panel "NEXT STEPS", "N of 10 done", the current step highlighted with how (panels.rs:975: e.g. "Click Wall in the bar at the bottom, or press 1. Type its thickness and height, then click each corner."); empty job: "Nothing is drawn yet. Next steps, on the left, says where to start." (3070); build strip "Pick a tool below to draw, or click anything on the model to select it." (1727); per-tool `how` line (tools.rs:121); a status message on nearly every action. Tooltips on hover: NO.

SCREENS AND INPUT
20 Fixed pixels: left 252, right 344, top 44, build bar 104 (scene.rs:37-41), middle bar between them clips (overflow clip, flex-shrink). Max heights are vh (83/88). Each panel can be put away by its switch (panels.rs:614), which is the narrow-screen answer. Below 1300 px cannot tell without running; the model gets 704 px at 1300.
21 Touch/pen/pinch: NO (grep Touch|Pinch: none). UI scale/text size: NO.
22 Settings screen: NO. Per-session state only (sun hour/month, switches); nothing saved about the program itself. Tool field values live in memory (`draw.settings`).

MARKUPS
23 Notes, clouds, arrows: NO. Count symbols: NO. Compare two drawing versions: NO (only "its file has changed since it was added" warning, panels.rs:1357; Layouts compares estimates). Plain measuring tool: NO. "Set scale" is calibration, and any dimension comes from drawing a wall.

Five a first-timer feels soonest: (1) no ghost/length-at-pointer while drawing; (2) no copy/array; (3) no search/command list/shortcut list; (4) no autosave; (5) no way to move doors/windows or rotate before placing.
Better than assumed: Next-steps teaching list; laser-in-keyboard burst handling (main.rs:462); carry-with-live-clash; snap to the lines of a PDF; typed refusal messages that name what to do; every panel has a switch; one-step undo of a whole carry.
