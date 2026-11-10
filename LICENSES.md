# Licenses

Rule: permissive licenses only. GPL and AGPL code is reference only and is never linked.
No dependency is linked before its license is confirmed.

## Linked crates

Everything the workspace links, direct and transitive, with the license each crate
declares in the registry. Regenerate with `tools/licenses.sh`. It parses each license
expression properly (brackets, then AND, then OR) and exits non-zero if any crate can
only be used by accepting a license outside the permissive list (MIT, MIT-0, Apache-2.0,
BSD-2/3-Clause, ISC, Zlib, 0BSD, Unicode-3.0, CC0-1.0, BSL-1.0, Unlicense).
`tools/licenses.sh --self-test` checks that logic against 19 cases, including
`GPL-3.0 AND (MIT OR Apache-2.0)`, which has to fail.

Direct dependencies: `bevy_ecs` 0.20.1, `serde` 1, `serde_json` 1. The rest come in through
those three.

| Crate | Version | License |
|---|---|---|
| arrayvec | 0.7.8 | MIT OR Apache-2.0 |
| async-task | 4.7.1 | Apache-2.0 OR MIT |
| atomic-waker | 1.1.2 | Apache-2.0 OR MIT |
| bevy_ecs_macro_logic | 0.20.1 | MIT OR Apache-2.0 |
| bevy_ecs_macros | 0.20.1 | MIT OR Apache-2.0 |
| bevy_ecs | 0.20.1 | MIT OR Apache-2.0 |
| bevy_macro_utils | 0.20.1 | MIT OR Apache-2.0 |
| bevy_platform | 0.20.1 | MIT OR Apache-2.0 |
| bevy_ptr | 0.20.1 | MIT OR Apache-2.0 |
| bevy_tasks | 0.20.1 | MIT OR Apache-2.0 |
| bevy_utils | 0.20.1 | MIT OR Apache-2.0 |
| bitflags | 2.13.2 | MIT OR Apache-2.0 |
| bumpalo | 3.20.3 | MIT OR Apache-2.0 |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 |
| concurrent-queue | 2.5.0 | Apache-2.0 OR MIT |
| convert_case | 0.10.0 | MIT |
| crossbeam-queue | 0.3.14 | MIT OR Apache-2.0 |
| crossbeam-utils | 0.8.23 | MIT OR Apache-2.0 |
| derive_more-impl | 2.1.1 | MIT |
| derive_more | 2.1.1 | MIT |
| disqualified | 1.1.0 | MIT OR Apache-2.0 |
| equivalent | 1.0.3 | Apache-2.0 OR MIT |
| fixedbitset | 0.5.9 | MIT OR Apache-2.0 |
| foldhash | 0.2.0 | Zlib |
| futures-core | 0.3.34 | MIT OR Apache-2.0 |
| futures-lite | 2.6.1 | Apache-2.0 OR MIT |
| hashbrown | 0.16.1 | MIT OR Apache-2.0 |
| hashbrown | 0.17.1 | MIT OR Apache-2.0 |
| indexmap | 2.14.2 | Apache-2.0 OR MIT |
| itoa | 1.0.19 | MIT OR Apache-2.0 |
| log | 0.4.35 | MIT OR Apache-2.0 |
| memchr | 2.8.3 | Unlicense OR MIT |
| mutants | 0.0.3 | MIT |
| nonmax | 0.5.5 | MIT OR Apache-2.0 |
| pin-project-lite | 0.2.17 | Apache-2.0 OR MIT |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 |
| quote | 1.0.47 | MIT OR Apache-2.0 |
| rustc-hash | 2.1.3 | Apache-2.0 OR MIT |
| rustc_version | 0.4.1 | MIT OR Apache-2.0 |
| semver | 1.0.28 | MIT OR Apache-2.0 |
| serde_core | 1.0.229 | MIT OR Apache-2.0 |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 |
| serde_json | 1.0.154 | MIT OR Apache-2.0 |
| serde | 1.0.229 | MIT OR Apache-2.0 |
| slotmap | 1.1.1 | Zlib |
| smallvec | 1.16.5 | MIT OR Apache-2.0 |
| spin | 0.12.3 | MIT |
| syn | 2.0.119 | MIT OR Apache-2.0 |
| syn | 3.0.10 | MIT OR Apache-2.0 |
| thiserror-impl | 2.0.21 | MIT OR Apache-2.0 |
| thiserror | 2.0.21 | MIT OR Apache-2.0 |
| thread_local | 1.1.10 | MIT OR Apache-2.0 |
| toml_datetime | 1.1.3+spec-1.1.0 | MIT OR Apache-2.0 |
| toml_edit | 0.25.19+spec-1.1.0 | MIT OR Apache-2.0 |
| toml_parser | 1.1.5+spec-1.1.0 | MIT OR Apache-2.0 |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| unicode-segmentation | 1.13.3 | MIT OR Apache-2.0 |
| unicode-xid | 0.2.6 | MIT OR Apache-2.0 |
| unsynn | 0.3.0 | MIT OR Apache-2.0 |
| variadics_please | 2.0.0 | MIT OR Apache-2.0 |
| version_check | 0.9.5 | MIT/Apache-2.0 |
| winnow | 1.0.5 | MIT |
| zmij | 1.0.23 | MIT |

## Candidates checked and not linked

Checked 2026-11-10; the raw output is in `evidence/d0-landscape.txt`. "Registry" is the
crates.io API; "repository" is the license file read
from a shallow clone of the source.

| Candidate | License | Checked against | Call |
|---|---|---|---|
| bevy 0.20.1 | MIT OR Apache-2.0 | registry | build on at D2 (not yet linked) |
| bevy_egui 0.43.1 | MIT | registry | build on at D2 (not yet linked) |
| egui / eframe 0.37.0 | MIT OR Apache-2.0 | registry | fallback for the window |
| BIMCraft 0.11.0 | MIT OR Apache-2.0 | repository (`LICENSE-MIT`, `LICENSE-APACHE`, `Cargo.toml`) | borrow (port with notice) |
| rust_xlsxwriter 0.102.0 | MIT OR Apache-2.0 | registry | build on at D6 |
| IronCalc 0.9.1 | MIT OR Apache-2.0 | registry and repository | candidate at D3/D5 |
| calamine 0.38.0 | MIT | registry | candidate |
| hayro 0.9.0 | Apache-2.0 OR MIT | registry and repository | build on at D2 |
| pdfium-render 0.9.5 | MIT OR Apache-2.0 (wrapper) | registry and repository | fallback; PDFium's own license not checked |
| i_overlay 9.1.0 | MIT OR Apache-2.0 | registry | candidate |
| earcutr 0.5.0 | ISC | registry | candidate |
| monstertruck 0.4.1 | Apache-2.0 | registry and repository | not used now |
| truck-modeling 0.6.0 | Apache-2.0 | registry and repository | not used now |
| z3 0.22.0 | MIT | registry (no license file at the repository root) | not used |
| hecs 0.11.2 | MIT OR Apache-2.0 | registry | passed over |
| Brush | Apache-2.0 | repository | no call until D7 is ruled on |
| ifc-lite-core 28.2.0 | MPL-2.0 | registry and repository | not linked (QUESTIONS.md Q3) |
| bimifc-parser 0.3.0 | MPL-2.0 | registry and `Cargo.toml` (no license file at the repository root) | not linked |
| boolmesh 0.2.2 | MPL-2.0 | registry | not linked |
| opencascade 0.3.0 | LGPL-2.1 | registry and repository | not used |
| cadrum 0.9.1 | MIT wrapper over an LGPL-2.1 kernel | registry and repository | not used |
| IfcOpenShell | LGPL-3.0 (and GPL-3.0) | repository (`COPYING.LESSER`, `COPYING`) | separate tool only, never linked |
| OpenAEC open-pdf-studio | LGPL-3.0 | repository | not used |
| Fornjot (`fj` 0.49.0) | 0BSD | registry and repository | shut down; not used |
| straight-skeleton 0.2.1 | GPL-2.0-or-later | registry and repository | reference only |
| OpenTakeoff | AGPL-3.0-only (Apache-2.0 through tag `apache-2.0-final`) | repository (`LICENSE`, README) | reference only |
| OpenConstructionERP | AGPL-3.0 | repository | reference only |
| OpenDroneMap | AGPL-3.0 | repository | separate tool only, never linked |
| IFC5-development | no license file at the root; README says the viewer is MIT | repository | pattern only; no code or schema taken |
| ProTakeoff | not verified (address not reachable) | | not used |
| ConstructCAD | not verified (not located) | | not used |

## Borrowed code

None yet. When BIMCraft's skeleton is ported (DECISIONS.md call 3), its notice goes in a
NOTICE file and the ported file names its source file and commit in a header.
