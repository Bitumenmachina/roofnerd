# CNC and fabrication notes (started)
## A (so far)
- mozaikcabinetsoftware.com does not resolve (network); real site mozaiksoftware.com read.
- Mozaik CNC https://www.mozaiksoftware.com/products/mozaik-cnc : true-shape nesting, 175+ stock post processors free, post files open text, G-code batch/per sheet, $225/mo CNC tier, extra seats $60, customizable construction (dowel/dado), remnant library.
- Cabinet Vision: hexagon page 403. Third party https://www.myarchitectai.com/blog/cabinet-design-software : price unpublished, Windows only, G-code via xMachining; Mozaik $125/225/325; Microvellum quote-based, runs inside AutoCAD/BricsCAD; Polyboard lifetime $2,795/$5,295 Pro PP, OptiNest sep., DXF via post; SketchList 3D DXF only no G-code $999.99 one-time; CabinetSense SketchUp ext, DXF for Vectric/AlphaCam/Mastercam, 32mm line boring, price unlisted.
- Cabinet Vision price: https://www.myarchitectai.com/blog/cabinet-vision-pricing (third-party, conflicting).
- G-code https://en.wikipedia.org/wiki/G-code : RS-274 EIA 1963, RS-274-D 1979; ISO 6983 1982; dialects per control; posts became common 2010s.
- Microvellum https://www.microvellum.com/ : customizable library of construction methods+hardware; specs drive shop drawings, material lists, CNC code; annual subscription, no price listed; estimating of material/hardware/labor/overhead.
- KCD/Cabinet Pro: search results https://kcdsoftware.com/product/cabinet-closet-commander/ (True Shape Nesting, CNC output). Cabinet Pro $2,974-8,974 one-time, no libraries (per https://surfaceandpanel.com/woodworking-software-for-cncs-in-2021-and-beyond, Dec 2020: KCD $295/mo + $1,995 setup; CV from $275/mo or $10,000 perpetual; Microvellum support hourly, no straight price; Router-CIM $7,500).
- Polyboard: polyboard.com failed to resolve (EAI_AGAIN); used myarchitectai.
- Woodshop News https://www.woodshopnews.com/cnc-software-advances-put-powerful-cad-cam-within-reach-of-every-shop : xMachining G-code, xOptimizer yield; posts once custom per machine.
- Open source: FreeCAD Woodworking workbench MIT https://github.com/dprojects/Woodworking (cut list CSV/JSON); OpenCutList GPLv3 https://github.com/lairdubois/lairdubois-opencutlist-sketchup-extension (reference only).
## B/C so far
- FreeCAD LGPL-2.1 https://github.com/FreeCAD/FreeCAD ; LinuxCNC GPL-2.0 (badge mixed) https://github.com/LinuxCNC/linuxcnc ; OpenCAMLib LGPL-2.1 https://github.com/aewallin/opencamlib ; Kiri:Moto MIT https://github.com/GridSpace/grid-apps ; (all of FreeCAD/LinuxCNC/OCL = reference only)
- SVGnest MIT https://github.com/Jack000/SVGnest (NFP + genetic algorithm, FFD start); Deepnest license not stated on page https://github.com/Jack000/Deepnest; libnest2d LGPL-3.0 https://github.com/tamasmeszaros/libnest2d (do not link). aklinker1/cutlist MIT (Onshape).
- Rust crates (crates.io API): cut-optimizer-2d 0.4.2 MIT OR Apache-2.0, last 2022-03, https://github.com/jasonrhansen/cut-optimizer-2d ; cut-optimizer-1d 0.2.2 MIT OR Apache-2.0 (stock lengths); bin-packing 0.6.2 MIT (cut list + irregular nesting, updated 2026-11-24, 1.9k dl) https://github.com/0xdoublesharp/bin-packing ; u-nesting 0.30.0 MIT https://github.com/iyulab/U-Nesting ; rectangle-pack 0.4.2 MIT/Apache https://github.com/chinedufn/rectangle-pack (8M dl).
## E crates (crates.io API)
- dxf 0.6.2 MIT 390k dl https://github.com/ixmilia/dxf-rs ; acadrust MIT (DXF+DWG; docs.rs/crate/acadrust) ; gcode 0.7.0 MIT OR Apache-2.0 parser no-std https://github.com/Michael-F-Bryan/gcode-rs ; ruststep 0.4.0 Apache-2.0 STEP https://github.com/ricosjp/ruststep ; dstv 0.8.0 MIT https://github.com/tluijken/dstv (9k dl, 2025-01) ; btlx: crates.io search empty ; bvbs: crates.io search finds none (only GAEB crates).
- design2machine.com/btlx page lists Documentation/Schema/Viewer/Example files; openness not stated on page I read.
## D (search-result level; page bodies not all fetched)
- BTLx: open XML successor of BTL (BTL by SEMA/cadwork); spec PDF https://design2machine.com/btlx/btlx_2_3_0.pdf (as cited by https://gramaziokohler.github.io/compas_timber/latest/contribution/BTLx_contribution_guide.html); Arkance lists BTL, BTLx, Hundegger BVX, MiTek EHX, Randek, Weinmann WUP https://arkance.world/gb-en/products/be-smart/building/cnc-exporters-wood . Hundegger uses own BVX.
- DSTV NC1: https://support.tekla.com/doc/tekla-structures/2025/int_nc_dstv_file_description ASCII, German Steel Construction Assoc standard; one file per part.
- BVBS: ASCII .abs, 2.0 (2000), 3.0 (2018) https://support.tekla.com/doc/tekla-structures/2025/int_bvbs_exporting
- Fabrication: MAJ default job format; CAMduct nesting+NC, ITM item templates; forum says ITM undocumented https://help.autodesk.com/cloudhelp/ENU/Fabrication-UsersGuide/files/GUID-4246EDD6-D38E-42FE-9089-235B90B18F2A.htm
- DXF reference: https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/index.htm ; PDF archive https://aps.autodesk.com/node/3072 (documented openly by Autodesk)
- NIST RS274NGC: NISTIR 6556 https://nvlpubs.nist.gov/nistpubs/Legacy/IR/nistir6556.pdf (Kramer, Proctor, Messina, Aug 2000) (not opened, search snippet)
- Yield (vendor claims, search snippets): manual 15-25% waste, optimized 5-8%; kerf 3-5% https://woodcuttool.pages.dev/blog/plywood-optimization-kerf-grain-offcuts/ https://maxcutsoftware.com/waste-optimization ; SWOOD 20% https://www.goengineer.com/blog/cut-material-costs-by-20-percent-with-swood ; nest vs point to point https://www.woodworkingnetwork.com/magazine/nesting-vs-point-point
- hsbcad: manufacturers; wall types database user-defined; MiTek Structure for component manufacturers (https://www.mitek-us.com/software/).
## Late findings
- Vectric prices via search snippet (VCarve Desktop $349, Pro $699, Aspire $1,995); vectric.com/purchase page showed no figures. Posts are text config files (docs.vectric.com post-processor-editing).
- Polyboard DXF post: layer names per operation, tool name can be in layer name; per-part files https://wooddesigner.org/help-centre/polyboard-post-processor-parameters-dxf/
- Complaints: woodweb https://woodweb.com/knowledge_base/Cabinet_Drafting_Programs.html (learning curve weeks-years; CV CNC weak; MV problems to some machines; build own libraries). mclinestudios.com/cabinet-vision-vs-microvellum. Switching cost $30-40K quote on a forum.
- NOT VERIFIED (no page opened): Cabinet Vision/Hexagon own page (403), Polyboard own site, KCD own page, Fusion/Easel/Carbide/Mastercam pages, countertop slab layout, pipe spooling, IFC-to-fabrication, DXF layer conventions per CAM.
