# Licenses

Rule: permissive licenses only. GPL and AGPL code is reference only and is never linked.
No dependency is linked before its license is confirmed.

## Linked crates

Everything the workspace links, direct and transitive, with the license each crate
declares in the registry. Regenerate with `tools/licenses.sh`. It parses each license
expression properly (brackets, then AND, then OR) and exits non-zero if any crate can
only be used by accepting a license outside the permissive list (MIT, MIT-0, Apache-2.0,
BSD-2/3-Clause, ISC, Zlib, 0BSD, Unicode-3.0, Unicode-DFS-2016, CC0-1.0, BSL-1.0,
Unlicense; the only exception clause accepted is `WITH LLVM-exception`).
`tools/licenses.sh --self-test` checks that logic against 24 cases, including
`GPL-3.0 AND (MIT OR Apache-2.0)`, which has to fail.

The table has 343 rows for 332 crates; 10 are linked in more than one version
(`foldhash`, `hashbrown`, `itertools`, `linux-raw-sys`, `miniz_oxide`, `rustc-hash`, `rustix`, `syn`, `thiserror`, `thiserror-impl`).

Direct dependencies: `bevy_ecs` 0.20.1, `serde` 1 and `serde_json` 1 in the world crate, and
`bevy` 0.20.1 in the application crate (with audio, gamepad and Wayland support left off).
The rest come in through those. The world crate and the command line still link only the
first three and what they bring; the engine is linked by the 3D application alone.

| Crate | Version | License |
|---|---|---|
| accesskit_winit | 0.33.2 | Apache-2.0 |
| accesskit | 0.24.1 | MIT OR Apache-2.0 |
| adler2 | 2.0.1 | 0BSD OR MIT OR Apache-2.0 |
| aho-corasick | 1.1.5 | Unlicense OR MIT |
| allocator-api2 | 0.2.21 | MIT OR Apache-2.0 |
| annotate-snippets | 0.12.17 | MIT OR Apache-2.0 |
| anstream | 1.0.0 | MIT OR Apache-2.0 |
| anstyle-parse | 1.0.0 | MIT OR Apache-2.0 |
| anstyle-query | 1.1.5 | MIT OR Apache-2.0 |
| anstyle | 1.0.14 | MIT OR Apache-2.0 |
| approx | 0.5.1 | Apache-2.0 |
| arrayvec | 0.7.8 | MIT OR Apache-2.0 |
| as-raw-xcb-connection | 1.0.1 | MIT OR Apache-2.0 |
| ash | 0.38.0+1.3.281 | MIT OR Apache-2.0 |
| assert_type_match | 0.1.1 | MIT OR Apache-2.0 |
| async-broadcast | 0.7.2 | MIT OR Apache-2.0 |
| async-channel | 2.5.0 | Apache-2.0 OR MIT |
| async-executor | 1.14.1 | Apache-2.0 OR MIT |
| async-fs | 2.2.0 | Apache-2.0 OR MIT |
| async-io | 2.6.0 | Apache-2.0 OR MIT |
| async-lock | 3.5.0 | Apache-2.0 OR MIT |
| async-task | 4.7.1 | Apache-2.0 OR MIT |
| atomic-waker | 1.1.2 | Apache-2.0 OR MIT |
| atomicow | 1.2.0 | MIT OR Apache-2.0 |
| autocfg | 1.5.1 | Apache-2.0 OR MIT |
| bevy_a11y | 0.20.1 | MIT OR Apache-2.0 |
| bevy_animation_macros | 0.20.1 | MIT OR Apache-2.0 |
| bevy_animation | 0.20.1 | MIT OR Apache-2.0 |
| bevy_app | 0.20.1 | MIT OR Apache-2.0 |
| bevy_asset_macros | 0.20.1 | MIT OR Apache-2.0 |
| bevy_asset | 0.20.1 | MIT OR Apache-2.0 |
| bevy_camera | 0.20.1 | MIT OR Apache-2.0 |
| bevy_clipboard | 0.20.1 | MIT OR Apache-2.0 |
| bevy_color | 0.20.1 | MIT OR Apache-2.0 |
| bevy_core_pipeline | 0.20.1 | MIT OR Apache-2.0 |
| bevy_curve | 0.20.1 | MIT OR Apache-2.0 |
| bevy_derive | 0.20.1 | MIT OR Apache-2.0 |
| bevy_diagnostic | 0.20.1 | MIT OR Apache-2.0 |
| bevy_ecs_macro_logic | 0.20.1 | MIT OR Apache-2.0 |
| bevy_ecs_macros | 0.20.1 | MIT OR Apache-2.0 |
| bevy_ecs | 0.20.1 | MIT OR Apache-2.0 |
| bevy_encase_derive | 0.20.1 | MIT OR Apache-2.0 |
| bevy_extract_macros | 0.20.1 | MIT OR Apache-2.0 |
| bevy_extract | 0.20.1 | MIT OR Apache-2.0 |
| bevy_gizmos_macros | 0.20.1 | MIT OR Apache-2.0 |
| bevy_gizmos_render | 0.20.1 | MIT OR Apache-2.0 |
| bevy_gizmos | 0.20.1 | MIT OR Apache-2.0 |
| bevy_image | 0.20.1 | MIT OR Apache-2.0 |
| bevy_input_focus | 0.20.1 | MIT OR Apache-2.0 |
| bevy_input | 0.20.1 | MIT OR Apache-2.0 |
| bevy_internal | 0.20.1 | MIT OR Apache-2.0 |
| bevy_light | 0.20.1 | MIT OR Apache-2.0 |
| bevy_log | 0.20.1 | MIT OR Apache-2.0 |
| bevy_macro_utils | 0.20.1 | MIT OR Apache-2.0 |
| bevy_material_macros | 0.20.1 | MIT OR Apache-2.0 |
| bevy_material | 0.20.1 | MIT OR Apache-2.0 |
| bevy_math | 0.20.1 | MIT OR Apache-2.0 |
| bevy_mesh | 0.20.1 | MIT OR Apache-2.0 |
| bevy_mikktspace | 1.0.0 | Zlib AND (MIT OR Apache-2.0) |
| bevy_pbr | 0.20.1 | MIT OR Apache-2.0 |
| bevy_picking | 0.20.1 | MIT OR Apache-2.0 |
| bevy_platform | 0.20.1 | MIT OR Apache-2.0 |
| bevy_ptr | 0.20.1 | MIT OR Apache-2.0 |
| bevy_reflect_derive | 0.20.1 | MIT OR Apache-2.0 |
| bevy_reflect | 0.20.1 | MIT OR Apache-2.0 |
| bevy_render_macros | 0.20.1 | MIT OR Apache-2.0 |
| bevy_render | 0.20.1 | MIT OR Apache-2.0 |
| bevy_shader | 0.20.1 | MIT OR Apache-2.0 |
| bevy_shape | 0.20.1 | MIT OR Apache-2.0 |
| bevy_sprite_render | 0.20.1 | MIT OR Apache-2.0 |
| bevy_sprite | 0.20.1 | MIT OR Apache-2.0 |
| bevy_state_macros | 0.20.1 | MIT OR Apache-2.0 |
| bevy_state | 0.20.1 | MIT OR Apache-2.0 |
| bevy_tasks | 0.20.1 | MIT OR Apache-2.0 |
| bevy_text | 0.20.1 | MIT OR Apache-2.0 |
| bevy_time | 0.20.1 | MIT OR Apache-2.0 |
| bevy_transform | 0.20.1 | MIT OR Apache-2.0 |
| bevy_ui_render | 0.20.1 | MIT OR Apache-2.0 |
| bevy_ui | 0.20.1 | MIT OR Apache-2.0 |
| bevy_utils | 0.20.1 | MIT OR Apache-2.0 |
| bevy_window | 0.20.1 | MIT OR Apache-2.0 |
| bevy_winit | 0.20.1 | MIT OR Apache-2.0 |
| bevy | 0.20.1 | MIT OR Apache-2.0 |
| bit-set | 0.10.0 | Apache-2.0 OR MIT |
| bit-vec | 0.9.1 | Apache-2.0 OR MIT |
| bitflags | 2.13.2 | MIT OR Apache-2.0 |
| bitvec | 1.1.1 | MIT |
| blake3 | 1.8.7 | CC0-1.0 OR Apache-2.0 OR Apache-2.0 WITH LLVM-exception |
| blocking | 1.7.0 | Apache-2.0 OR MIT |
| bumpalo | 3.20.3 | MIT OR Apache-2.0 |
| bytemuck_derive | 1.12.1 | Zlib OR Apache-2.0 OR MIT |
| bytemuck | 1.25.2 | Zlib OR Apache-2.0 OR MIT |
| byteorder-lite | 0.1.0 | Unlicense OR MIT |
| calloop | 0.13.0 | MIT |
| cc | 1.11.1 | MIT OR Apache-2.0 |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 |
| cfg_aliases | 0.2.2 | MIT |
| codespan-reporting | 0.13.1 | Apache-2.0 |
| colorchoice | 1.0.5 | MIT OR Apache-2.0 |
| concurrent-queue | 2.5.0 | Apache-2.0 OR MIT |
| const-fnv1a-hash | 1.1.0 | MIT |
| const_panic | 0.2.17 | Zlib |
| const_soft_float | 0.1.4 | MIT OR Apache-2.0 |
| constant_time_eq | 0.4.2 | CC0-1.0 OR MIT-0 OR Apache-2.0 |
| constgebra | 0.1.4 | MIT OR Apache-2.0 |
| convert_case | 0.10.0 | MIT |
| cpufeatures | 0.3.1 | MIT OR Apache-2.0 |
| crc32fast | 1.5.5 | MIT OR Apache-2.0 |
| crossbeam-channel | 0.5.17 | MIT OR Apache-2.0 |
| crossbeam-queue | 0.3.14 | MIT OR Apache-2.0 |
| crossbeam-utils | 0.8.23 | MIT OR Apache-2.0 |
| ctrlc | 3.5.2 | MIT/Apache-2.0 |
| cursor-icon | 1.2.0 | MIT OR Apache-2.0 OR Zlib |
| derive_more-impl | 2.1.1 | MIT |
| derive_more | 2.1.1 | MIT |
| displaydoc | 0.2.7 | MIT OR Apache-2.0 |
| disqualified | 1.1.0 | MIT OR Apache-2.0 |
| dlib | 0.5.3 | MIT |
| document-features | 0.2.12 | MIT OR Apache-2.0 |
| downcast-rs | 2.0.2 | MIT OR Apache-2.0 |
| dpi | 0.1.2 | Apache-2.0 AND MIT |
| either | 1.19.0 | MIT OR Apache-2.0 |
| encase_derive_impl | 0.12.1 | MIT-0 |
| encase_derive | 0.12.1 | MIT-0 |
| encase | 0.12.1 | MIT-0 |
| equivalent | 1.0.3 | Apache-2.0 OR MIT |
| erased-serde | 0.4.11 | MIT OR Apache-2.0 |
| euclid | 0.22.14 | MIT OR Apache-2.0 |
| event-listener-strategy | 0.5.4 | Apache-2.0 OR MIT |
| event-listener | 5.4.2 | Apache-2.0 OR MIT |
| fastrand | 2.5.0 | Apache-2.0 OR MIT |
| fdeflate | 0.3.7 | MIT OR Apache-2.0 |
| find-msvc-tools | 0.1.15 | MIT OR Apache-2.0 |
| fixedbitset | 0.5.9 | MIT OR Apache-2.0 |
| flate2 | 1.1.11 | MIT OR Apache-2.0 |
| fnv | 1.0.7 | Apache-2.0 / MIT |
| foldhash | 0.1.5 | Zlib |
| foldhash | 0.2.0 | Zlib |
| font-types | 0.12.7 | MIT OR Apache-2.0 |
| fontique | 0.11.1 | Apache-2.0 OR MIT |
| funty | 2.0.0 | MIT |
| futures-core | 0.3.34 | MIT OR Apache-2.0 |
| futures-io | 0.3.34 | MIT OR Apache-2.0 |
| futures-lite | 2.6.1 | Apache-2.0 OR MIT |
| futures-macro | 0.3.34 | MIT OR Apache-2.0 |
| futures-task | 0.3.34 | MIT OR Apache-2.0 |
| futures-util | 0.3.34 | MIT OR Apache-2.0 |
| gethostname | 1.1.0 | Apache-2.0 |
| getrandom | 0.4.3 | MIT OR Apache-2.0 |
| glam | 0.33.12 | MIT OR Apache-2.0 |
| glob | 0.3.4 | MIT OR Apache-2.0 |
| gpu-allocator | 0.28.0 | MIT OR Apache-2.0 |
| guillotiere | 0.7.0 | MIT/Apache-2.0 |
| half | 2.7.1 | MIT OR Apache-2.0 |
| harfrust | 0.12.0 | MIT |
| hashbrown | 0.15.5 | MIT OR Apache-2.0 |
| hashbrown | 0.16.1 | MIT OR Apache-2.0 |
| hashbrown | 0.17.1 | MIT OR Apache-2.0 |
| hexasphere | 19.0.0 | MIT OR Apache-2.0 |
| icu_collections | 2.4.0 | Unicode-3.0 |
| icu_locale_core | 2.4.0 | Unicode-3.0 |
| icu_locale_fallback_data | 2.4.0 | Unicode-3.0 |
| icu_locale_fallback | 2.4.0 | Unicode-3.0 |
| icu_normalizer_data | 2.4.0 | Unicode-3.0 |
| icu_normalizer | 2.4.0 | Unicode-3.0 |
| icu_properties_data | 2.4.0 | Unicode-3.0 |
| icu_properties | 2.4.0 | Unicode-3.0 |
| icu_provider | 2.4.0 | Unicode-3.0 |
| icu_segmenter_data | 2.4.0 | Unicode-3.0 |
| icu_segmenter | 2.4.0 | Unicode-3.0 |
| image | 0.25.10 | MIT OR Apache-2.0 |
| indexmap | 2.14.2 | Apache-2.0 OR MIT |
| inventory | 0.3.25 | MIT OR Apache-2.0 |
| is_terminal_polyfill | 1.70.2 | MIT OR Apache-2.0 |
| itertools | 0.14.0 | MIT OR Apache-2.0 |
| itertools | 0.15.0 | MIT OR Apache-2.0 |
| itoa | 1.0.19 | MIT OR Apache-2.0 |
| ktx2 | 0.5.0 | Apache-2.0 |
| lalrpop-util | 0.23.2 | Apache-2.0 OR MIT |
| lazy_static | 1.5.1 | MIT OR Apache-2.0 |
| lexical-core | 1.0.6 | MIT/Apache-2.0 |
| lexical-parse-float | 1.0.6 | MIT/Apache-2.0 |
| lexical-parse-integer | 1.0.6 | MIT/Apache-2.0 |
| lexical-util | 1.0.7 | MIT/Apache-2.0 |
| lexical-write-float | 1.0.6 | MIT/Apache-2.0 |
| lexical-write-integer | 1.0.6 | MIT/Apache-2.0 |
| lexical | 7.0.5 | MIT/Apache-2.0 |
| libc | 0.2.191 | MIT OR Apache-2.0 |
| libloading | 0.8.9 | ISC |
| libm | 0.2.16 | MIT |
| linebender_resource_handle | 0.1.1 | Apache-2.0 OR MIT |
| linux-raw-sys | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| linux-raw-sys | 0.4.15 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| litemap | 0.8.3 | Unicode-3.0 |
| litrs | 1.0.1 | MIT OR Apache-2.0 |
| lock_api | 0.4.14 | MIT OR Apache-2.0 |
| logos-codegen | 0.16.1 | MIT OR Apache-2.0 |
| logos-derive | 0.16.1 | MIT OR Apache-2.0 |
| logos | 0.16.1 | MIT OR Apache-2.0 |
| log | 0.4.35 | MIT OR Apache-2.0 |
| matchers | 0.2.0 | MIT |
| memchr | 2.8.3 | Unlicense OR MIT |
| memmap2 | 0.9.12 | MIT OR Apache-2.0 |
| miniz_oxide | 0.8.9 | MIT OR Zlib OR Apache-2.0 |
| miniz_oxide | 0.9.1 | MIT OR Zlib OR Apache-2.0 |
| moxcms | 0.8.1 | BSD-3-Clause OR Apache-2.0 |
| mutants | 0.0.3 | MIT |
| naga-types | 30.0.1 | MIT OR Apache-2.0 |
| naga | 30.0.1 | MIT OR Apache-2.0 |
| nix | 0.31.3 | MIT |
| nonmax | 0.5.5 | MIT OR Apache-2.0 |
| nu-ansi-term | 0.50.3 | MIT |
| num-traits | 0.2.19 | MIT OR Apache-2.0 |
| offset-allocator | 0.2.0 | MIT |
| once_cell | 1.21.4 | MIT OR Apache-2.0 |
| ordered-float | 5.5.1 | MIT |
| parking_lot_core | 0.9.12 | MIT OR Apache-2.0 |
| parking_lot | 0.12.5 | MIT OR Apache-2.0 |
| parking | 2.2.1 | Apache-2.0 OR MIT |
| parlance | 0.1.1 | Apache-2.0 OR MIT |
| parley_data | 0.11.1 | Apache-2.0 OR MIT |
| parley | 0.11.1 | Apache-2.0 OR MIT |
| percent-encoding | 2.3.2 | MIT OR Apache-2.0 |
| petgraph | 0.8.3 | MIT OR Apache-2.0 |
| pin-project-lite | 0.2.17 | Apache-2.0 OR MIT |
| piper | 0.2.5 | MIT OR Apache-2.0 |
| pkg-config | 0.3.34 | MIT OR Apache-2.0 |
| png | 0.18.1 | MIT OR Apache-2.0 |
| polling | 3.11.0 | Apache-2.0 OR MIT |
| potential_utf | 0.1.6 | Unicode-3.0 |
| presser | 0.3.1 | MIT OR Apache-2.0 |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 |
| profiling | 1.0.18 | MIT OR Apache-2.0 |
| pxfm | 0.1.30 | BSD-3-Clause OR Apache-2.0 |
| quote | 1.0.47 | MIT OR Apache-2.0 |
| radium | 0.7.0 | MIT |
| radsort | 0.1.1 | MIT OR Apache-2.0 |
| rand_core | 0.10.1 | MIT OR Apache-2.0 |
| rand_distr | 0.6.0 | MIT OR Apache-2.0 |
| rand | 0.10.3 | MIT OR Apache-2.0 |
| raw-window-handle | 0.6.2 | MIT OR Apache-2.0 OR Zlib |
| read-fonts | 0.41.0 | MIT OR Apache-2.0 |
| rectangle-pack | 0.4.2 | MIT/Apache-2.0 |
| regex-automata | 0.4.18 | MIT OR Apache-2.0 |
| regex-syntax | 0.8.11 | MIT OR Apache-2.0 |
| renderdoc-sys | 1.1.0 | MIT OR Apache-2.0 |
| ron | 0.12.3 | MIT OR Apache-2.0 |
| rustc-hash | 1.1.0 | Apache-2.0/MIT |
| rustc-hash | 2.1.3 | Apache-2.0 OR MIT |
| rustc_version | 0.4.1 | MIT OR Apache-2.0 |
| rustix | 0.38.44 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| rustix | 1.1.6 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| ruzstd | 0.9.1 | MIT |
| scopeguard | 1.2.0 | MIT OR Apache-2.0 |
| semver | 1.0.28 | MIT OR Apache-2.0 |
| serde_core | 1.0.229 | MIT OR Apache-2.0 |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 |
| serde_json | 1.0.154 | MIT OR Apache-2.0 |
| serde_spanned | 1.1.3 | MIT OR Apache-2.0 |
| serde | 1.0.229 | MIT OR Apache-2.0 |
| sharded-slab | 0.1.7 | MIT |
| shlex | 2.0.1 | MIT OR Apache-2.0 |
| simd-adler32 | 0.3.10 | MIT |
| skrifa | 0.44.0 | MIT OR Apache-2.0 |
| slab | 0.4.12 | MIT |
| slotmap | 1.1.1 | Zlib |
| smallvec | 1.16.5 | MIT OR Apache-2.0 |
| smol_str | 0.2.2 | MIT OR Apache-2.0 |
| spin | 0.12.3 | MIT |
| spirv | 0.4.0+sdk-1.4.341.0 | Apache-2.0 |
| stable_deref_trait | 1.2.1 | MIT OR Apache-2.0 |
| stackfuture | 0.3.1 | MIT |
| static_assertions | 1.1.0 | MIT OR Apache-2.0 |
| swash | 0.2.10 | Apache-2.0 OR MIT |
| synstructure | 0.14.0 | MIT |
| syn | 2.0.119 | MIT OR Apache-2.0 |
| syn | 3.0.10 | MIT OR Apache-2.0 |
| sys-locale | 0.3.2 | MIT OR Apache-2.0 |
| taffy | 0.14.0 | MIT |
| tap | 1.0.1 | MIT |
| thiserror-impl | 1.0.69 | MIT OR Apache-2.0 |
| thiserror-impl | 2.0.21 | MIT OR Apache-2.0 |
| thiserror | 1.0.69 | MIT OR Apache-2.0 |
| thiserror | 2.0.21 | MIT OR Apache-2.0 |
| thread_local | 1.1.10 | MIT OR Apache-2.0 |
| tinystr | 0.8.5 | Unicode-3.0 |
| toml_datetime | 1.1.3+spec-1.1.0 | MIT OR Apache-2.0 |
| toml_edit | 0.25.19+spec-1.1.0 | MIT OR Apache-2.0 |
| toml_parser | 1.1.5+spec-1.1.0 | MIT OR Apache-2.0 |
| toml_writer | 1.2.0+spec-1.1.0 | MIT OR Apache-2.0 |
| toml | 1.2.1+spec-1.1.0 | MIT OR Apache-2.0 |
| tracing-attributes | 0.1.31 | MIT |
| tracing-core | 0.1.36 | MIT |
| tracing-log | 0.2.0 | MIT |
| tracing-subscriber | 0.3.23 | MIT |
| tracing | 0.1.44 | MIT |
| twox-hash | 2.1.5 | MIT |
| typeid | 1.0.3 | MIT OR Apache-2.0 |
| typewit | 1.15.2 | Zlib |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| unicode-segmentation | 1.13.3 | MIT OR Apache-2.0 |
| unicode-width | 0.2.3 | MIT OR Apache-2.0 |
| unicode-xid | 0.2.6 | MIT OR Apache-2.0 |
| unsynn | 0.3.0 | MIT OR Apache-2.0 |
| utf8_iter | 1.1.0 | Apache-2.0 OR MIT |
| utf8parse | 0.2.2 | Apache-2.0 OR MIT |
| utf_types | 1.0.0 | Apache-2.0 OR MIT |
| uuid | 1.29.2 | Apache-2.0 OR MIT |
| variadics_please | 2.0.0 | MIT OR Apache-2.0 |
| version_check | 0.9.5 | MIT/Apache-2.0 |
| weak-table | 0.4.0 | MIT |
| wesl-core | 0.1.0 | MIT OR Apache-2.0 |
| wesl-macros | 0.6.0 | MIT OR Apache-2.0 |
| wesl | 0.6.0 | MIT OR Apache-2.0 |
| wgpu-core-deps-windows-linux-android | 30.0.1 | MIT OR Apache-2.0 |
| wgpu-core | 30.0.1 | MIT OR Apache-2.0 |
| wgpu-hal | 30.0.1 | MIT OR Apache-2.0 |
| wgpu-naga-bridge | 30.0.1 | MIT OR Apache-2.0 |
| wgpu-types | 30.0.1 | MIT OR Apache-2.0 |
| wgpu | 30.0.1 | MIT OR Apache-2.0 |
| wgsl-parse | 0.6.0 | MIT OR Apache-2.0 |
| wgsl-types | 0.6.0 | MIT OR Apache-2.0 |
| winit | 0.30.13 | Apache-2.0 |
| winnow | 1.0.5 | MIT |
| writeable | 0.6.4 | Unicode-3.0 |
| wyz | 0.5.1 | MIT |
| x11-dl | 2.21.0 | MIT |
| x11rb-protocol | 0.13.2 | MIT OR Apache-2.0 |
| x11rb | 0.13.2 | MIT OR Apache-2.0 |
| xkbcommon-dl | 0.4.2 | MIT |
| xkeysym | 0.2.1 | MIT OR Apache-2.0 OR Zlib |
| yazi | 0.2.2 | Apache-2.0 OR MIT |
| yoke-derive | 0.8.4 | Unicode-3.0 |
| yoke | 0.8.4 | Unicode-3.0 |
| zeno | 0.3.3 | Apache-2.0 OR MIT |
| zerocopy-derive | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT |
| zerocopy | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT |
| zerofrom-derive | 0.1.8 | Unicode-3.0 |
| zerofrom | 0.1.8 | Unicode-3.0 |
| zerotrie | 0.2.5 | Unicode-3.0 |
| zerovec-derive | 0.11.7 | Unicode-3.0 |
| zerovec | 0.11.9 | Unicode-3.0 |
| zmij | 1.0.23 | MIT |

## Candidates checked and not linked

Checked 2026-11-10; the raw output is in `demos/captured/d0-landscape.txt`. "Registry" is the
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

| What | From | License it is used under | Where |
|---|---|---|---|
| The weighted straight skeleton, with the vector type and three geometry helpers it uses | BIMCraft, `crates/core/src/skeleton.rs` and `crates/core/src/geom.rs`, commit `f202abd85d1cf165325e2f7b250da0b5e51f5b71` (https://github.com/IPedrax/bimcraft; MIT OR Apache-2.0) | MIT | `crates/world/src/skeleton.rs`. Its header says it is ported and lists what was changed. The copyright and permission notice are in `NOTICE`. |

Nothing else is borrowed. The owner's earlier program was read for the pattern a cost line
is worked in (DECISIONS.md call 18); none of its code was copied.

## Bundled fonts

`tools/licenses.sh` lists crates. Fonts are not crates, so they are listed here by hand.

| What | From | License | Where |
|---|---|---|---|
| Inter, three weights (Regular, Medium, SemiBold), unmodified | The Inter Project (https://rsms.me/inter/), version 4.0, as packaged by Debian (`fonts-inter` 4.0+ds-1) | SIL Open Font License 1.1 | `crates/app/assets/fonts/`, built into `plumbline-app`. The copyright notice and the license are in `crates/app/assets/fonts/OFL.txt` and referred to from `NOTICE`. |

The OFL is not one of the permissive software licenses rule 1 names. It restricts the
font, not the program that carries it (QUESTIONS.md Q12).
