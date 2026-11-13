# Research report: Earth-style camera, glass panels, BIM-tool UI patterns

## How to read this
- **Sourced** means I fetched the primary file or page this session. The URL is given.
- **Inference** or **recalled** means my own derivation or memory, not fetched. Treat it as a starting value to tune by eye.
- **Not obtained.** Apple's HIG and developer pages render with JavaScript and gave me no numbers (the dimensions and rates for toggles, scroll bars and springs below are not from Apple). I fetched no Blender, Google Earth, Twinmotion, Enscape or Planner 5D pages with numbers, and no Fluent acrylic numeric table. Where I give figures for those, they are flagged.
- **Interface text.** You don't ship these terms to users; the words below are for you.

## Part 1. Camera and scroll

### Licences and sources
| Project | Licence | Source |
|---|---|---|
| CesiumJS ScreenSpaceCameraController | Apache-2.0 (your premise; I did not fetch the licence file) | https://raw.githubusercontent.com/CesiumGS/cesium/main/packages/engine/Source/Scene/ScreenSpaceCameraController.js |
| three.js OrbitControls, MapControls | MIT (your premise) | https://raw.githubusercontent.com/mrdoob/three.js/dev/examples/jsm/controls/OrbitControls.js |
| yomotsu/camera-controls | MIT (your premise) | https://raw.githubusercontent.com/yomotsu/camera-controls/dev/src/CameraControls.ts |
| bevy_panorbit_camera | **MIT or Apache-2.0** (README: dual licensed) | https://raw.githubusercontent.com/Plonq/bevy_panorbit_camera/master/README.md |
| Blender viewport | GPL, behaviour only | not fetched |
| Google Earth | proprietary, behaviour only | not fetched |

**bevy_panorbit_camera Bevy support** (README table): Bevy 0.18 uses 0.34, Bevy 0.19 uses 0.35, Bevy 0.20 uses 0.36.

### (a) Wheel zoom
| System | Zoom target | Formula and normalisation | Limits | Source |
|---|---|---|---|---|
| three OrbitControls | Centre by default; cursor if `zoomToCursor = true` | scale = 0.95^(zoomSpeed × \|deltaY × 0.01\|). Wheel units: line mode ×16, page mode ×100, Ctrl held (pinch) ×10. | min 0, max Infinity | OrbitControls.js (URL above) |
| three zoomToCursor (perspective) | Cursor | Unproject the pointer to a ray. Move the camera along it by (old radius − clamped new radius). With a 70° tilt limit, the target is not moved when within 20° of the horizon. | same | same |
| camera-controls | Centre; cursor if `dollyToCursor = true` (default false) | Builds a plane through the target facing the camera. Converts the cursor to a point on it using radius × tan(fov/2). Moves the target toward that point by lerpRatio = (prevRadius − newRadius) / newRadius. Wheel: line mode ÷ factor, pixel mode ÷ (factor × 10). Factor is −1 on Mac, −3 elsewhere. A Ctrl-wheel event (a Mac pinch) uses the two-finger action. | min distance = Number.EPSILON, max Infinity. `infinityDolly` pushes the target past the limit. | CameraControls.ts (URL above) |
| Cesium | Along the view toward the globe pick; screen centre in look-at mode | zoomRate = clamp(5.0 × (distance − minHeight), 20, 5.9e12). distance moved = zoomRate × min(diff / canvasHeight, 0.1). diff comes from the aggregator: wheel arcLength = 7.5 × toRadians(delta). | `minimumZoomDistance` 1.0, max Infinity; collision detection on | ScreenSpaceCameraController.js, CameraEventAggregator.js (https://raw.githubusercontent.com/CesiumGS/cesium/main/packages/engine/Source/Scene/CameraEventAggregator.js) |
| bevy_panorbit_camera | Centre only. No zoom-to-cursor in the source I read. | Line scroll moves the target radius by target_radius × 0.2. Pixel scroll moves both target and current radius, clamped. Pinch becomes pixel scroll ×0.015. | `zoom_lower_limit` 0.05 | https://raw.githubusercontent.com/Plonq/bevy_panorbit_camera/master/src/lib.rs |

- **Blender:** behaviour is zoom to mouse position and orbit around selection, both off by default. Recalled, unverified.
- **Google Earth:** the web version zooms with the scroll wheel (official help: https://support.google.com/earth/answer/148186). I found no numeric formula for it.

### (b) Inertia and damping
| System | Constants | Update rule | Frame-rate independent? | Source |
|---|---|---|---|---|
| three OrbitControls | dampingFactor 0.05 | Each frame, angles += delta × 0.05, target += panOffset × 0.05, then deltas and panOffset ×= 0.95. | No. It is per frame. Derived: half-life about 13.5 frames, so about 0.22 s at 60 fps. | OrbitControls.js |
| Cesium | inertiaSpin 0.9, inertiaTranslate 0.9, inertiaZoom 0.8. `maximumMovementRatio` 0.1. Inertia only if the click lasted under 0.4 s. | decay(t, c) = exp(−τ·t) with τ = (1 − c) × 25. Motion = (lastEnd − lastStart) × 0.5 per axis, applied as start + motion × decay(secondsSinceRelease). Skipped under 0.5 px. | Yes. It is a function of seconds. Derived: c = 0.9 gives τ = 2.5/s (time constant 0.4 s); c = 0.8 gives τ = 5/s (0.2 s). | ScreenSpaceCameraController.js |
| camera-controls | smoothTime 0.25 s, draggingSmoothTime 0.125 s, restThreshold 0.01 | smoothDamp(current, target, velocityRef, smoothTime, maxSpeed, dt). It is the Unity SmoothDamp from Game Programming Gems 4: omega = 2/smoothTime, x = omega·dt, exp = 1/(1 + x + 0.48x² + 0.235x³). | Yes (takes dt). | CameraControls.ts; https://raw.githubusercontent.com/yomotsu/camera-controls/dev/src/utils/math-utils.ts |
| bevy_panorbit_camera | orbit_smoothness 0.1, pan_smoothness 0.02, zoom_smoothness 0.1 (code values; the doc comments say 0.8, 0.6 and 0.8) | `lerp_and_snap` with dt. Exact form not seen. | Claims dt use | lib.rs |

### (c) Drag buttons, pan, pivot, pitch
| System | Left drag | Middle | Right | Pan method | Orbit pivot | Pitch limits |
|---|---|---|---|---|---|---|
| three MapControls | pan | dolly | rotate | Pan is by screen-space delta × distance × tan(fov/2) × 2 / clientHeight (OrbitControls `_pan`). Not a ray-plane grab. `screenSpacePanning` false. One finger pans, two fingers dolly and rotate. | target | min/max polar angle (defaults 0 and π) |
| three OrbitControls | rotate | dolly | pan | same screen-space form, `screenSpacePanning` true | target | same |
| camera-controls | rotate | dolly | truck (truckSpeed 2.0) | delta-based | target | min/max polar angle (defaults 0 and π) |
| Cesium 3D | pan on the globe, rotate | zoom | zoom, tilt | Not fetched. | Tilt pivots on the pick at the drag start (cached as `_tiltCenter`), falling back to the ellipsoid hit | rotateRate clamped to [1/5000, 1.77]; `maximumTiltAngle` optional, no limit if unset |
| bevy_panorbit_camera | orbit | none | pan | sensitivity-based | focus point | `pitch_upper_limit` and `pitch_lower_limit` default to None |

- **Cesium tilt rate.** rotateRate = rotateFactor × (rho − adjustment). deltaTheta = rotateRate × min(thetaWindowRatio, 0.1) × π.
- **Cesium ground pan.** I did not read the globe-pan code. Ray-plane grab is the standard technique and is my inference (see the scheme below).
- **Google Earth web.** Official help documents mouse and keyboard navigation (https://support.google.com/earth/answer/148186). A community thread reports the web compass is not draggable, and tilt and rotate work via middle button or Shift and arrow keys (https://support.google.com/earth/thread/237424547/where-are-the-tilt-and-compass-controls-on-the-new-google-earth-web?hl=en). That is user-reported, not official.

### (d) Fly-to and double-click
| System | Duration rule | Easing | Source |
|---|---|---|---|
| Cesium flyTo | If unset: ceil(distance / 1,000,000 m) + 2, capped at 3.0 s. So always 2 to 3 s. Zero or less jumps. | CUBIC_OUT if the start height is above the end height and above 11,500 m, else QUINTIC_IN_OUT. Height follows an arc: if the peak altitude exceeds both endpoint heights, altitude − x^8 / 1e6. Peak = min(0.2 × altitude, 1e9). | https://raw.githubusercontent.com/CesiumGS/cesium/main/packages/engine/Source/Scene/CameraFlightPath.js |
| Cesium QUADRATIC_OUT | n/a | t × (2 − t) (from the JSDoc example). It maps to Tween.js. | https://raw.githubusercontent.com/CesiumGS/cesium/main/packages/engine/Source/Core/EasingFunction.js |
| camera-controls | smoothTime 0.25 s, so it settles in roughly 4 × smoothTime | SmoothDamp, a spring-like critically damped response | CameraControls.ts |

### (e) Trackpad pinch and two-finger scroll
- **Browsers:** a pinch arrives as a wheel event with ctrlKey = true. three scales it ×10 (OrbitControls.js). camera-controls routes it to the two-finger action (CameraControls.ts).
- **Bevy:** panorbit supports touch (one finger orbits, two fingers pan, pinch zooms). Pinch is off by default for the trackpad (`trackpad_pinch_to_zoom_enabled` false). Its `TrackpadBehavior::BlenderLike` uses Shift to pan and Ctrl to zoom (lib.rs).
- **Convention** (inference): on a trackpad, two-finger scroll pans and pinch zooms; on a mouse the wheel zooms.

### Recommended scheme for a house about 60 ft across (feet)
This is my design, assembled from the sources above. Constants are starting values.

**Camera state:** target T (always on the ground plane, y = 0 for ground level), yaw ψ, pitch θ (degrees above the horizon), distance d. Camera position C = T + d·(cosθ·sinψ, sinθ, cosθ·cosψ).

**Buttons:**
- Left drag: grab-pan the ground.
- Right drag, or Shift + left drag: orbit.
- Middle drag: orbit as well.
- Wheel: zoom to cursor.
- Double-click: fly to the point or object.
- Ctrl + left drag: tilt only.

**Wheel normalisation:**
1. px = deltaY × (16 if deltaMode = line, 100 if page, 1 if pixel). This is three's rule.
2. Pinch (ctrlKey): ×10.
3. Clamp |px| to 300 per event.
4. s = 0.90^(px / 100). Positive px zooms out.
5. A notch of 100 px therefore moves 10%. three's 0.95 per notch is the more cautious value; try 0.90 first.

**Zoom about the cursor (exact, derived):**
1. Find P: the ray under the cursor hits the model, else the ground plane y = 0.
2. Clamp s so that d·s stays within [dmin, dmax]: s = clamp(d·s, 2 ft, 400 ft) / d.
3. Then C' = P + (C − P)·s and T' = P + (T − P)·s. Orientation is unchanged, so P stays under the cursor.
4. If the ray misses the ground (near the horizon), zoom about the screen centre.

**Grab-pan (ray-plane, derived):**
1. On press, intersect the ray with the plane y = 0 (or a plane through the picked point, parallel to the ground) to get G0. Freeze that plane.
2. On each move, intersect again to get G1. Set T += G0 − G1. The grabbed point stays under the cursor exactly.
3. If the ray is within 5° of parallel to the plane, skip that move.
4. At pitch near 90° (top-down) this is identical to the plain screen pan.

**Orbit:**
1. Pivot Q is the ground or mesh point under the cursor at press. Fall back to the ground point at screen centre.
2. Yaw change = 2π × dx / canvasHeight, and pitch change = 2π × dy / canvasHeight. This is three's rotate rule.
3. Apply yaw about the vertical axis through Q to both C and T. Pitch changes only θ about Q, then re-derive T on the ground.

**Clamps:**
| Quantity | Value |
|---|---|
| Distance | 2 ft to 400 ft |
| Pitch | 8° to 90° (never below the ground) |
| Ground bound | keep T within ±300 ft of the model centre |
| Collision | after any move, if C.y < 1.5 ft, raise C.y to 1.5 ft and recompute θ |

**Inertia (frame-rate independent, Cesium form):**
- On release, if the press lasted under 0.4 s, velocity v = last motion × 0.5 per axis.
- Each frame, position += v × decay(Δt), with decay(t) = exp(−τ·t).
- τ = 2.5 /s for pan and orbit (Cesium 0.9). τ = 5 /s for zoom (Cesium 0.8).
- Stop below 0.5 px of movement.
- While held, follow the pointer with SmoothDamp, smoothTime 0.125 s (camera-controls draggingSmoothTime).

**Transitions (plan to perspective, fly-to):**
- **Plan to perspective:** animate θ from 90° to 35° while d changes by a chosen factor (try ×1.2). Duration 0.8 s, easing QUINTIC_IN_OUT (Cesium's choice for most flights). Tween.js form: k = 2t; if k < 1 then 0.5·k⁵, else 0.5·((k − 2)⁵ + 2).
- **Double-click fly-to:** duration = clamp(0.4 + 0.15 × log2(1 + travel / 10 ft), 0.4, 1.2) s. This is my inference. Cesium's 2 to 3 s is for globe scale and too slow here.
- Keep one perspective projection throughout, and narrow the field of view to reach a near-orthographic plan view if you want a clean top-down. No source tested.
- Any new user input cancels the animation.

## Part 2. Glass and depth

### What the official sources actually say
| Source | Finding | URL |
|---|---|---|
| Apple HIG, Materials (via mirror) | Liquid Glass is a functional layer for controls and navigation floating above content; do not use it in the content layer. Use it sparingly on custom controls. Transient controls (sliders, toggles) take a glass look when activated. | https://unpkg.com/apple-hig-mcp@1.1.3/content/materials.md (a copy of Apple's page, not Apple's server) |
| Apple HIG, Materials | Thicker materials are more opaque and give better contrast for fine text. Thinner are more translucent. "Avoid using quaternary on top of the thin and ultraThin materials, because the contrast is too low." The page has no numeric ratios. | same |
| Microsoft Acrylic | Recipe layers: background, blur, exclusion blend, colour/tint overlay, noise. Avoid layering multiple acrylic surfaces and edge-to-edge panes (visible seam). It is GPU-intensive and falls back to a solid colour. Do not put accent-coloured text on acrylic. Optimised so text meets contrast on top. No numeric parameters given on the page. | https://learn.microsoft.com/en-us/windows/apps/design/style/acrylic |
| Microsoft backdrops | Customisable parameters are TintColor, TintOpacity, LuminosityOpacity and FallbackColor. Mica is opaque and captures the wallpaper once; acrylic is for transient surfaces. | https://learn.microsoft.com/en-us/windows/apps/develop/ui/system-backdrops |
| Fluent motion | Durations: Normal 250 ms, Fast 167 ms, Faster 83 ms. Easing: Fast Out Slow In (entering) cubic-bezier(0, 0, 0, 1). Slow Out Fast In (exiting) cubic-bezier(1, 0, 1, 1). | https://learn.microsoft.com/en-us/windows/apps/design/motion/timing-and-easing |
| MDN backdrop-filter | `backdrop-filter: blur(10px) saturate(150%)` and a 30%-white background is the example form. An element with opacity under 1 becomes a backdrop root and blurs only what is between it and the child. No recommended ranges. | https://developer.mozilla.org/en-US/docs/Web/CSS/backdrop-filter |

**Important for your case.** Both Apple and Microsoft restrict glass to floating or transient surfaces and keep large background areas opaque or standard. That fits floating panels over a 3D view. It argues against glass for the estimate sheet itself, which is dense text.

### Starting token table (inference, not from Apple or Microsoft pages)
Blur radius values are CSS px at 1× scale. In a Bevy shader, the same value is a Gaussian sigma of about radius/2 (my inference).

| Token | Light | Dark | Note |
|---|---|---|---|
| panel blur radius | 20 px (range 16 to 28) | 24 px (range 18 to 32) | thicker for text-heavy panels |
| backdrop saturation | 160% (range 140 to 180) | 140% | MDN example uses 150% |
| tint, opacity | white (#FFFFFF), 55 to 65% | near-black (#14161A), 55 to 65% | thick for sheet-like panels: 75 to 85% |
| noise / grain | 2 to 3% alpha | 3 to 4% alpha | hides banding |
| inner top highlight | 1 px, white 60% | 1 px, white 18% | top and left edge |
| border (outer) | 1 px, black 8% | 1 px, white 10% | |
| corner radius | 6 px small control, 10 px button or field, 14 px card, 18 to 22 px floating panel | same | rounded corners scale with element size |
| shadow level 1 (resting) | 0 1 2 0 black 10%; 0 4 12 -2 black 8% | 0 1 2 0 black 40%; 0 4 12 -2 black 30% | |
| shadow level 2 (raised panel) | 0 8 24 -4 black 14%; 0 2 6 0 black 8% | 0 8 24 -4 black 50%; 0 2 6 0 black 30% | |
| shadow level 3 (menu) | 0 16 48 -8 black 22%; 0 4 12 0 black 10% | 0 16 48 -8 black 60% | |
| text contrast | 4.5:1 body, 3:1 large text and icons (WCAG 2.x) | same | measure against the worst background under the panel |
| hover on glass | raise fill by +6% white (light: +4% black) | +8% white | |
| selection on glass | accent fill at 18 to 22% plus 1 px accent border | accent fill 28% plus 1 px accent border | accent text on glass is discouraged by Microsoft |
| press | fill +10%, scale 0.98 | same | |

### Controls and motion (mostly recalled, unverified)
| Item | Value | Status |
|---|---|---|
| Toggle (iOS) | 51 × 31 pt | A Stack Overflow comment says UISwitch is 51 × 31; Apple's UISwitch page confirms only that the size is enforced and the frame ignored (https://developer.apple.com/documentation/uikit/uiswitch/init(frame:).md). Not official. |
| Toggle animation | 200 to 250 ms with an ease-out curve | inference; Fluent's 167 and 250 ms are the sourced bracket |
| Segmented control | 28 to 32 px tall, radius at half height, thumb slides 200 ms | inference |
| Overlay scroll bar | 6 px resting, 10 to 12 px on hover; fade in 100 ms, fade out 300 ms after 800 ms idle; minimum thumb length 32 px | recalled, unverified |
| Scroll deceleration (UIKit) | normal 0.998, fast 0.99, applied per millisecond | recalled from UIKit; the page body did not load, so unverified |
| Rubber-band | stretch about 0.55 × overscroll, spring back in about 0.4 s | recalled, unverified |
| Spring (UI motion) | about 0.4 to 0.5 s response, damping ratio 0.8 to 1.0 | recalled, unverified |
| UI easing | enter cubic-bezier(0, 0, 0, 1); exit cubic-bezier(1, 0, 1, 1); durations 83 / 167 / 250 ms | **sourced** (Fluent, URL above) |

## Part 3. What building tools put on screen
Sourced evidence here is thin (listed first). The rest is from general knowledge of these products and should be confirmed by looking at them.

**Sourced:**
- **That Open Company Hider:** isolate by category, hide by category, reset all. Works across several loaded models. Its example filters by category only. https://docs.thatopen.com/Tutorials/Components/Core/Hider
- **Speckle viewer:** Apache-2.0. Features not confirmed from the page: https://github.com/specklesystems/speckle-server/tree/main/packages/viewer (docs are at speckle.guide/viewer/).
- **Online3DViewer:** MIT. Exports 3dm, bim, gltf, obj, off, stl, ply. https://github.com/kovacsv/Online3DViewer
- **Autodesk Docs/Forma viewer** (via search results; not read in full): a Levels control for display by level, a Section tool with a yellow arrow gizmo to move the cut plane, and a View Cube (Front click makes the cut orthogonal). The Properties tool queries metadata. Forma Site Design has a context-sensitive right panel (area metrics, element properties such as height, floors and function). Sources: https://www.autodesk.com/learn/ondemand/module/project-or-get-started-in-the-classroom-with-docs-and-build/unit/2cXOVEgA9f0Yee6gzfkv0u and https://blogs.autodesk.com/forma/2026/08/28/forma-site-design-tools-overview/
- **Autodesk FormIt:** palette bar of Properties, Layers, Scenes, Levels, Content Library, Collaboration. Levels are added with a "+" button. https://windows.help.formit.autodesk.com/tool-library/levels-and-area

**Recurring patterns (general knowledge, not fetched):**
| Pattern | Who does it | How |
|---|---|---|
| Visibility by storey and discipline | Autodesk Docs/Forma, FormIt, That Open, Speckle, Online3DViewer, Qonic, Rayon | Tree or list with eye toggles per level and per system. Isolate means show only the selection. Ghost means the rest is faded to low opacity. That Open Hider does isolate, hide and reset. |
| Selection feedback | Speckle, That Open, Online3DViewer, Twinmotion, Enscape | Outline or tint, plus a gizmo on the selected object. |
| Moving an object | Snaptrude, Arcol, Planner 5D, The Sims | Drag on the floor plane, snap to walls and grid, rotate handle, red or green feedback on collision. |
| Properties panel | Forma, Autodesk Docs | Right-hand, context-sensitive, with grouped fields. |
| Navigation widget | Autodesk Docs (View Cube), Google Earth (compass, 2D/3D toggle, zoom buttons) | Compass or cube top-right, home button, zoom buttons. |
| Level picker or minimap | FormIt, Planner 5D, The Sims | Floor selector at one edge; a plan thumbnail. |
| Measurement readouts | That Open, Speckle, Online3DViewer | Click-click distance with a live label. |
| Export | Online3DViewer (gltf, obj, 3dm, stl and others), That Open and Speckle (IFC and their own formats) | Single export menu offering several formats. |

## The ten changes that would most make a dated flat docked UI feel like these tools
1. **Make the 3D view the whole window.** Float every panel over it instead of docking beside it (Apple: glass is for the floating control layer, not content).
2. **Zoom to the cursor and grab-pan the ground** exactly as specified above. This is what makes it feel like Google Earth.
3. **Add frame-rate independent inertia** (Cesium decay, τ 2.5 and 5 per second) so nothing stops dead.
4. **Use a few elevation levels with layered shadows and a 1 px inner highlight** on every floating panel (the token table).
5. **Frost the small floating panels, not the sheet.** Use a thick tint (75 to 85%) where text is dense, and never put accent-coloured text on glass.
6. **Put a navigation cluster in a corner:** compass that resets north, home button, plan/3D toggle, zoom buttons. The plan-to-perspective tilt is animated (0.8 s, quintic in-out).
7. **Offer a level picker** with eye toggles per storey and per trade, with isolate and ghost.
8. **Give selection a clear look:** outline plus an accent tint, and a handle to move or rotate.
9. **Show clash and snap feedback live** while dragging, in the same colour language as the clash list.
10. **Standardise motion:** 83, 167 and 250 ms with the Fluent curves for every panel, hover and toggle (the one fully sourced set).

Most useful next check for Parts 2 and 3: open Google Earth web, Forma, Snaptrude and Arcol, and record the measured values for blur, radius and shadow directly, since I could not fetch them.
