use super::*;

/// A table written out of the job, or brought into it: `export prices <file> <out.csv>`,
/// `import prices <file> <list.csv or .xlsx> [--sheet "<name>"] [--supplier "<name>"]
/// [--add] [--status <Known|Assumed> --source "<where from>"]`.
pub(super) fn tables(args: &[String]) -> Result<(), String> {
    use plumbline_world::tables;
    let cmd = args[0].as_str();
    let (mut plain, mut flags): (Vec<&str>, Vec<(&str, Option<&str>)>) = (Vec::new(), Vec::new());
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--add" => flags.push(("--add", None)),
            "--counts" => flags.push(("--counts", None)),
            "--holes" => flags.push(("--holes", None)),
            "--anyway" => flags.push(("--anyway", None)),
            f @ ("--sheet" | "--supplier" | "--status" | "--source" | "--name" | "--page" | "--as" | "--level" | "--units" | "--up" | "--cut" | "--division" | "--system" | "--only") => {
                let v = args.get(i + 1).ok_or_else(|| format!("{f} needs a value after it"))?;
                if flags.iter().any(|(known, _)| *known == f) {
                    return Err(format!("{f} is given twice"));
                }
                flags.push((f, Some(v.as_str())));
                i += 1;
            }
            f if f.starts_with("--") => return Err(format!("{cmd} does not take {f}\n{}", usage())),
            a => plain.push(a),
        }
        i += 1;
    }
    let flag = |name: &str| flags.iter().find(|(f, _)| *f == name).and_then(|(_, v)| *v);
    let [what, path, other] = plain.as_slice() else {
        return Err(format!("{cmd} takes what, the job's file and the table's file\n{}", usage()));
    };
    // What is written out is never written over a job: not the job it is written from,
    // and not any other file that is a job, whatever its name ends in (J17C-1).
    if cmd == "export" {
        let same = match (std::fs::canonicalize(path), std::fs::canonicalize(other)) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        };
        if same {
            return Err(format!("{other} is the job itself: what is written out of a job is not written over it. Give the table a name of its own"));
        }
        if std::fs::read_to_string(other).ok().is_some_and(|text| file::load(&text).is_ok()) {
            return Err(format!("{other} is a job: what is written out is not written over a job. Give the table a name that is not in use"));
        }
    }
    let (mut job, _) = read(path)?;
    // How sure what is brought in is, and where it came from, when the table does not say.
    let how = match (flag("--status"), flag("--source")) {
        (None, None) => None,
        (Some(status), Some(source)) => {
            let status = match status {
                "Known" => Status::Known,
                "Assumed" => Status::Assumed,
                other => return Err(format!("--status {other:?}: what is brought in is Known or Assumed")),
            };
            if source.trim().is_empty() {
                return Err("--source says where it came from; it is empty".into());
            }
            Some((status, source.to_string()))
        }
        _ => return Err("--status and --source go together: how sure it is, and where it came from".into()),
    };
    let table = |sheet: Option<&str>| -> Result<plumbline_sheets::Sheet, String> {
        // "starter" in place of an assembly library's file is the one the program ships with.
        let starter = matches!(*what, "assemblies" | "items" | "connections" | "cabinet-rules") && *other == "starter" && !std::path::Path::new(other).exists();
        let shipped = match *what {
            "items" => plumbline_world::tables::STARTER_ITEMS,
            "connections" => plumbline_world::tables::STARTER_CONNECTIONS,
            "cabinet-rules" => plumbline_world::tables::STARTER_CABINET_RULES,
            _ => plumbline_world::tables::STARTER_LIBRARY,
        };
        let bytes = if starter { shipped.as_bytes().to_vec() } else { std::fs::read(other).map_err(|e| format!("cannot read {other}: {e}"))? };
        plumbline_sheets::read(&bytes, sheet).map_err(|why| format!("{other}: {why}"))
    };
    let only = |allowed: &[&str]| -> Result<(), String> {
        match flags.iter().find(|(f, _)| !allowed.contains(f)) {
            Some((f, _)) => Err(format!("{cmd} {what} does not take {f}")),
            None => Ok(()),
        }
    };
    match (cmd, *what) {
        ("export", "database") => {
            only(&[])?;
            database_out(&mut job, path, other)
        }
        ("import", "database") => {
            only(&[])?;
            let said = database_in(&mut job, other)?;
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            let _ = std::io::stdout().write_all(said.as_bytes());
            Ok(())
        }
        ("export", "assemblies") => {
            only(&[])?;
            let rows = tables::assemblies_out(&mut job);
            std::fs::write(other, table_bytes(other, "Assemblies", &rows)?).map_err(|e| format!("cannot write {other}: {e}"))?;
            say!("wrote {other}: the assemblies of {path}, {} under its headings", count(rows.len() - 1, "row", "rows"));
            Ok(())
        }
        ("import", "assemblies") => {
            only(&["--sheet"])?;
            let sheet = table(flag("--sheet"))?;
            let brought = tables::assemblies_in(&mut job, &sheet.rows, Some(&sheet.numbers)).map_err(|why| format!("{other}: {why}"))?;
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            let _ = std::io::stdout().write_all(brought_said(other, &sheet, &brought).as_bytes());
            Ok(())
        }
        // The item library: the things that can be placed, by trade, as a table a
        // person can open, change and bring back in.
        // What is to verify in the field, as a sheet to print and take to the job.
        ("export", "vif") => {
            only(&[])?;
            let rows = plumbline_world::vif::field_list(&mut job);
            let bytes = if other.to_lowercase().ends_with(".xlsx") {
                let tab = plumbline_sheets::Tab { name: "Field list".into(), rows: rows.iter().map(|r| r.iter().map(|c| if c.is_empty() { plumbline_sheets::Cell::Empty } else { plumbline_sheets::Cell::Text(c.clone()) }).collect()).collect(), head: 1, widths: vec![6.0, 14.0, 24.0, 44.0, 52.0, 30.0] };
                plumbline_sheets::xlsx(&[tab])?
            } else {
                plumbline_sheets::csv(&rows).into_bytes()
            };
            std::fs::write(other, bytes).map_err(|e| format!("cannot write {other}: {e}"))?;
            say!("wrote {other}: what is to verify in the field in {path}, {} under its headings, the most dollars first", count(rows.len() - 1, "row", "rows"));
            Ok(())
        }
        // The list to verify as a sheet to fill on site, a row for each value to measure (call 234); and what was found, brought back.
        ("export", "field-sheet") => {
            only(&[])?;
            let sheet = plumbline_world::fieldsheet::make(&mut job);
            let rows = sheet.table();
            let bytes = if other.to_lowercase().ends_with(".xlsx") {
                use plumbline_sheets::Cell;
                let cells: Vec<Vec<Cell>> = rows
                    .iter()
                    .enumerate()
                    .map(|(r, row)| {
                        row.iter()
                            .enumerate()
                            .map(|(c, text)| match (r, c, sheet.rows.get(r.wrapping_sub(1)).and_then(|x| x.dollars)) {
                                _ if text.is_empty() => Cell::Empty,
                                (r, 5, Some(dollars)) if r >= 1 => Cell::Dollars(dollars),
                                _ => Cell::Text(text.clone()),
                            })
                            .collect()
                    })
                    .collect();
                let tab = plumbline_sheets::Tab { name: "Field sheet".into(), rows: cells, head: 0, widths: vec![24.0, 34.0, 60.0, 18.0, 10.0, 14.0, 20.0, 16.0, 12.0, 40.0] };
                plumbline_sheets::xlsx(&[tab])?
            } else {
                plumbline_sheets::csv(&rows).into_bytes()
            };
            std::fs::write(other, bytes).map_err(|e| format!("cannot write {other}: {e}"))?;
            let mut said = format!("wrote {other}: a sheet to fill on site from {path}: {} to measure, the most dollars first, each with its reference; fill As found, Measured by and On", count(sheet.rows.len(), "row", "rows"));
            let without = sheet.questions_without_rows();
            if without > 0 || !sheet.left.is_empty() {
                said.push_str(&format!("\n{} of the {} on the list to verify {} not rows, and the sheet says why under its rows:", without, count(sheet.questions, "question", "questions"), if without == 1 { "is" } else { "are" }));
                for l in &sheet.left {
                    said.push_str(&format!("\n  {}: {}", l.what, l.why));
                }
            }
            say!("{said}");
            Ok(())
        }
        ("import", "field-sheet") => {
            only(&["--anyway"])?;
            let sheet = table(None)?;
            let name = std::path::Path::new(other).file_name().and_then(|n| n.to_str()).unwrap_or(other);
            let brought = plumbline_world::fieldsheet::bring(&mut job, &sheet.rows, &sheet.numbers, name, flags.iter().any(|(f, _)| *f == "--anyway")).map_err(|why| why.join("\n"))?;
            // A sheet with nothing filled changes nothing, and the job is not written again.
            if !brought.done.is_empty() {
                std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            }
            let mut said = String::new();
            for d in &brought.done {
                for line in d.lines() {
                    said.push_str(&line);
                    said.push('\n');
                }
            }
            said.push_str(&brought.closing());
            said.push('\n');
            let _ = std::io::stdout().write_all(said.as_bytes());
            Ok(())
        }
        ("export", "items") => {
            only(&[])?;
            let rows = tables::items_out(&mut job);
            std::fs::write(other, table_bytes(other, "Items", &rows)?).map_err(|e| format!("cannot write {other}: {e}"))?;
            say!("wrote {other}: the items of {path}, {} under its headings", count(rows.len() - 1, "row", "rows"));
            Ok(())
        }
        // Where the items connect (call 211): one row a connection.
        ("export", "connections") => {
            only(&[])?;
            let rows = tables::connections_out(&mut job);
            std::fs::write(other, table_bytes(other, "Connections", &rows)?).map_err(|e| format!("cannot write {other}: {e}"))?;
            say!("wrote {other}: where the items of {path} connect, {} under its headings", count(rows.len() - 1, "row", "rows"));
            Ok(())
        }
        ("import", "connections") => {
            only(&["--sheet"])?;
            let sheet = table(flag("--sheet"))?;
            // The ones the program ships with are given to an item that has none and leave an item's own as they are.
            let shipped = *other == "starter" && !std::path::Path::new(other).exists();
            let (brought, kept) = if shipped {
                tables::connections_in_missing(&mut job, &sheet.rows, Some(&sheet.numbers)).map_err(|why| format!("{other}: {why}"))?
            } else {
                (tables::connections_in(&mut job, &sheet.rows, Some(&sheet.numbers)).map_err(|why| format!("{other}: {why}"))?, Vec::new())
            };
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            let _ = std::io::stdout().write_all(brought_said(other, &sheet, &brought).as_bytes());
            if !kept.is_empty() {
                say!("  items that have connections already, and keep them: {}", kept.len());
            }
            Ok(())
        }
        // How a cabinet's code is read (call 215): one row a rule, to open, change and bring back in.
        ("export", "cabinet-rules") => {
            only(&[])?;
            let rows = tables::cabinet_rules_out(&mut job);
            std::fs::write(other, table_bytes(other, "Cabinet codes", &rows)?).map_err(|e| format!("cannot write {other}: {e}"))?;
            say!("wrote {other}: how the codes of cabinets are read in {path}, {} under its headings", count(rows.len() - 1, "rule", "rules"));
            Ok(())
        }
        ("import", "cabinet-rules") => {
            only(&["--sheet"])?;
            let sheet = table(flag("--sheet"))?;
            // The program's own rules are always in force: `starter` keeps whatever the job holds for their letters.
            let shipped = *other == "starter" && !std::path::Path::new(other).exists();
            let (brought, kept) = if shipped {
                tables::cabinet_rules_in_missing(&mut job, &sheet.rows, Some(&sheet.numbers)).map_err(|why| format!("{other}: {why}"))?
            } else {
                (tables::cabinet_rules_in(&mut job, &sheet.rows, Some(&sheet.numbers)).map_err(|why| format!("{other}: {why}"))?, Vec::new())
            };
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            let _ = std::io::stdout().write_all(brought_said(other, &sheet, &brought).as_bytes());
            if !kept.is_empty() {
                say!("  the program's own rules are in force already, and are left as the job has them: {}", kept.len());
            }
            Ok(())
        }
        ("export", "shapes") => {
            only(&[])?;
            let (rows, unused) = tables::shapes_out_counted(&mut job);
            std::fs::write(other, table_bytes(other, "Shapes", &rows)?).map_err(|e| format!("cannot write {other}: {e}"))?;
            // The job's own shapes that no item is drawn as are written with the rest (call 213 point 13).
            let tail = if unused.is_empty() { String::new() } else { format!("; no item is drawn as {} of them: {}", unused.len(), unused.join(", ")) };
            say!("wrote {other}: the shapes the items of {path} are drawn as and the job's own, {} under its headings{tail}", count(rows.len() - 1, "row", "rows"));
            Ok(())
        }
        ("import", "shapes") => {
            only(&["--sheet"])?;
            let sheet = table(flag("--sheet"))?;
            let brought = tables::shapes_in(&mut job, &sheet.rows, Some(&sheet.numbers)).map_err(|why| format!("{other}: {why}"))?;
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            let _ = std::io::stdout().write_all(brought_said(other, &sheet, &brought).as_bytes());
            Ok(())
        }
        ("import", "items") => {
            only(&["--sheet"])?;
            let sheet = table(flag("--sheet"))?;
            // The library the program ships with adds what the job has not got and
            // writes over nothing the job has (J25A-18); a person's own file changes
            // the items it names.
            let shipped = *other == "starter" && !std::path::Path::new(other).exists();
            let (brought, kept) = if shipped {
                tables::items_in_missing(&mut job, &sheet.rows, Some(&sheet.numbers)).map_err(|why| format!("{other}: {why}"))?
            } else {
                (tables::items_in(&mut job, &sheet.rows, Some(&sheet.numbers)).map_err(|why| format!("{other}: {why}"))?, Vec::new())
            };
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            let _ = std::io::stdout().write_all(brought_said(other, &sheet, &brought).as_bytes());
            if !kept.is_empty() {
                say!("  in the job already, and left as the job has them: {}", kept.len());
            }
            Ok(())
        }
        ("import", "ifc") => {
            only(&["--status", "--source", "--only"])?;
            let (status, source) = how.clone().ok_or("say how sure the model is and where it came from: --status, --source")?;
            // What of the model is brought in (call 210): all of it, or only its runs and what they serve.
            let choice = match flag("--only") {
                None => plumbline_ifc::Bring::Whole,
                Some("runs") => plumbline_ifc::Bring::RunsOnly,
                Some(other) => return Err(format!("--only {other:?}: the choice is runs (the model's ducts, pipes, conduit and cable, their fittings, and the fixtures and equipment they serve; its walls, floors and the rest are not asked for)")),
            };
            let bytes = std::fs::read(other).map_err(|e| format!("cannot read {other}: {e}"))?;
            let called = std::path::Path::new(other).file_name().and_then(|s| s.to_str()).unwrap_or("the model");
            // The word that makes the choice at the command line is said where the model's own words offer it.
            let with_the_word = |said: String| said.replace("choose only its runs and the equipment they serve", "choose only its runs and the equipment they serve (--only runs)").replace("can be brought in alone, without its walls and floors", "can be brought in alone, without its walls and floors (--only runs)");
            let b = plumbline_ifc::import_choosing(&mut job, &bytes, called, plumbline_world::editing::Entered { status, source: &source }, choice).map_err(|why| format!("{other}: {}", with_the_word(why)))?;
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            let mut out = format!("{other} ({}): brought in{}\n", b.schema, if choice == plumbline_ifc::Bring::RunsOnly { ", only its runs and what they serve" } else { "" });
            // One of a thing is said as one: "1 door", not "1 doors" (J17B-19).
            for (what, n) in &b.made {
                let _ = writeln!(out, "    {n} {}", if *n == 1 { plumbline_ifc::one_of(what) } else { what });
            }
            let _ = writeln!(out, "  not brought in: {}", count(b.left_out.iter().map(|l| l.1).sum(), "thing", "things"));
            for line in b.left_out_said() {
                let _ = writeln!(out, "    {line}");
            }
            if let Some(said) = b.not_asked_said() {
                let _ = writeln!(out, "  {said}");
            }
            for note in &b.notes {
                let _ = writeln!(out, "  note: {}", with_the_word(note.clone()));
            }
            // What it was done for (call 210 point 4): the clashes the things brought in make, where they are listed.
            let said = b.clashes_said();
            if !said.is_empty() {
                let _ = writeln!(out, "  clashes:");
                for line in said {
                    let _ = writeln!(out, "    {}", line.replace("they are in the clash list, with the job's own", &format!("they are listed with the job's own by `plumbline clashes {path}`")));
                }
            }
            let _ = std::io::stdout().write_all(out.as_bytes());
            Ok(())
        }
        // A scan of the building as a point cloud, cut into a sheet to trace over
        // (DECISIONS.md call 128). The sheet is a drawing under the level, already to
        // scale and in the scan's own place; everything about it is Assumed.
        ("import", "scan") => {
            use plumbline_plans::scan::{Units, Up};
            only(&["--level", "--units", "--up", "--cut"])?;
            let level = flag("--level").ok_or("say which level the scan is of: --level <level>")?.to_string();
            // What the person said of the unit: nothing, or m or ft. Where nothing is said, a LAS or LAZ file's own word is
            // taken and said, and meters where it has none (call 230).
            let said_units = match flag("--units") {
                None => None,
                Some("m") => Some(Units::Metres),
                Some("ft") => Some(Units::Feet),
                Some(other) => return Err(format!("--units {other:?}: a scan is in m or ft")),
            };
            let up = match flag("--up") {
                None | Some("z") => Up::Z,
                Some("y") => Up::Y,
                Some(other) => return Err(format!("--up {other:?}: up is z or y")),
            };
            let cut_at = match flag("--cut") {
                None => 4.0,
                Some(text) => {
                    let (len, unit) = plumbline_world::units::parse_length(text).map_err(|why| format!("--cut {text:?}: {why}"))?;
                    len.feet(unit).map_err(|why| format!("--cut {text:?}: {why}"))?
                }
            };
            let bytes = std::fs::read(other).map_err(|e| format!("cannot read {other}: {e}"))?;
            let units = plumbline_plans::scan::units_to_use(&bytes, said_units).map_err(|why| format!("{other}: {why}"))?;
            let kind = plumbline_plans::scan::kind_of(&bytes);
            let called = std::path::Path::new(other).file_stem().and_then(|s| s.to_str()).unwrap_or("scan").to_string();
            let folder = std::path::Path::new(path).parent().map(|p| p.to_path_buf()).unwrap_or_default();
            let ext = kind.map_or("ply", |k| match k {
                plumbline_plans::scan::Kind::Ply => "ply",
                plumbline_plans::scan::Kind::Las => "las",
                plumbline_plans::scan::Kind::Laz => "laz",
                plumbline_plans::scan::Kind::E57 => "e57",
            });
            // The one way a scan is brought in, here and in the window (call 153).
            let brought = plumbline_plans::scan::bring_in(&bytes, other, &called, &plumbline_plans::scan::Said { units, up: Some(up), cut_at }, &mut |s| {
                plumbline_world::plans::lay_scan_sheet_of(&mut job, &level, &folder, s.called, ext, s.taken, s.cut_at, s.png, s.checksum, s.size, s.cut.size, s.cut.dots, s.cut.corner)
            })?;
            let (made, taken, cut, floor, ceiling, leaned, scan_points) = (brought.laid.clone(), brought.taken.clone(), &brought.cut, brought.floor, brought.ceiling, brought.leaned, brought.points);
            let ft_in = |x: f64| plumbline_world::units::feet_as_ft_in(x, 16);
            let drawn = job.find(&made).ok_or("the drawing is not in the job")?;
            let kept = job.world.get::<plumbline_world::plans::Drawing>(drawn).and_then(|d| d.file.value.clone()).unwrap_or_default();
            job.derive();
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            let mut out = format!("{made}: scan {called:?}: {} points, {taken} (--units ft or --up y says otherwise)\n", view::figure(scan_points as f64, 0));
            // To the inch: a scan's points are each a little out, and its edge with them.
            let about = |x: f64| plumbline_world::units::feet_as_ft_in((x * 12.0).round() / 12.0, 16);
            if let Some(k) = kind.filter(|k| *k != plumbline_plans::scan::Kind::Ply) {
                let _ = writeln!(out, "  read as {} {} file", if k == plumbline_plans::scan::Kind::E57 { "an" } else { "a" }, k.said());
            }
            for note in &brought.file_said {
                let _ = writeln!(out, "  {note}");
            }
            if let Some(lo) = brought.moved {
                let _ = writeln!(out, "  the file puts the scan {} east and {} north of its zero (a map grid's), farther than a job can lie: that is taken off, and the sheet lies with the scan's southwest corner at the job's zero", plumbline_world::units::length_said(lo[0]), plumbline_world::units::length_said(lo[1]));
            }
            let _ = writeln!(out, "  on plan, where it is cut, it is about {} by {}", about(cut.extent[0]), about(cut.extent[1]));
            if let Some(lean) = leaned.filter(|l| *l * 120.0 >= 0.125) {
                let _ = writeln!(out, "  it leaned {} in 10 ft and was levelled by its own floor (level it where it was cleaned for a truer sheet)", plumbline_world::units::show_length(&plumbline_world::units::Len::One((lean * 120.0 * 16.0).round() / 16.0), "in"));
            }
            match (floor, ceiling) {
                (Some(f), Some(c)) => {
                    let _ = writeln!(out, "  floor to ceiling reads about {} on the scan (it is not given to the level: measure the level's height with it if you take it)", about(c - f));
                }
                (Some(_), None) => {
                    let _ = writeln!(out, "  its floor shows and no ceiling does: no height read");
                }
                _ => {
                    let _ = writeln!(out, "  no floor shows in it: it is cut {} above its lowest point", ft_in(cut_at));
                }
            }
            let _ = writeln!(out, "  cut {} above the floor: {} points drawn on {kept}, {} by {} dots at {} to the foot", ft_in(cut_at), view::figure(cut.points as f64, 0), cut.size[0], cut.size[1], view::num(cut.dots));
            if cut.strays > 0 {
                let _ = writeln!(out, "  left off the sheet: {} stray points at that height, each alone in its square foot of plan", view::figure(cut.strays as f64, 0));
            }
            let _ = writeln!(out, "  it is a drawing under {level}, to scale and in the scan's own place, Assumed: trace over it as over any sheet");
            for lost in &brought.lost {
                let _ = writeln!(out, "  not kept: {lost}");
            }
            let _ = std::io::stdout().write_all(out.as_bytes());
            Ok(())
        }
        ("import", "maker-model") => {
            only(&["--as"])?;
            let as_kind = match flag("--as") {
                Some("equipment") => Some(plumbline_world::Kind::Equipment),
                Some("fixture") => Some(plumbline_world::Kind::Fixture),
                Some("furnishing") => Some(plumbline_world::Kind::Furnishing),
                Some(other) => return Err(format!("--as {other:?}: a thing of no stated kind is equipment, a fixture or a furnishing")),
                None => None,
            };
            let bytes = std::fs::read(other).map_err(|e| format!("cannot read {other}: {e}"))?;
            let called = std::path::Path::new(other).file_name().and_then(|s| s.to_str()).unwrap_or(other).to_string();
            let brought = plumbline_ifc::maker_model(&mut job, &bytes, &called, as_kind).map_err(|why| format!("{other}: {why}"))?;
            let lines = brought.said(&called, path);
            if brought.made.is_empty() {
                return Err(format!("{other}: no item was made\n{}", lines.join("\n")));
            }
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            for line in lines {
                say!("{line}");
            }
            Ok(())
        }
        ("import", "spec") => {
            only(&["--page", "--as", "--name"])?;
            let page = match flag("--page") {
                Some(p) => p.parse::<usize>().ok().filter(|n| *n >= 1).ok_or_else(|| format!("--page {p:?} is not a page number"))?,
                None => 1,
            };
            let bytes = std::fs::read(other).map_err(|e| format!("cannot read {other}: {e}"))?;
            let words = plumbline_plans::words(&bytes, page).map_err(|why| format!("{other}: {why}"))?;
            let called = std::path::Path::new(other).file_name().and_then(|s| s.to_str()).unwrap_or(other).to_string();
            if words.is_empty() {
                return Err(format!("{other}, page {page}: there are no words on it to read. A scanned sheet is a picture of words: enter the item's sizes by hand"));
            }
            let lines: Vec<(String, f64)> = words.into_iter().map(|w| (w.text, w.size)).collect();
            let sheet = plumbline_world::spec::read(&lines);
            let name = flag("--name").map(str::to_string).or_else(|| sheet.name.as_ref().map(|f| f.value.clone())).unwrap_or_else(|| called.clone());
            let (kind, why) = match flag("--as") {
                Some("equipment") => (plumbline_world::Kind::Equipment, "as you said".to_string()),
                Some("fixture") => (plumbline_world::Kind::Fixture, "as you said".to_string()),
                Some("furnishing") => (plumbline_world::Kind::Furnishing, "as you said".to_string()),
                Some(other) => return Err(format!("--as {other:?}: an item is equipment, a fixture or a furnishing")),
                None => match plumbline_world::spec::kind_said(&name) {
                    Some((kind, word)) => (kind, format!("its name says \u{201c}{word}\u{201d}; --as says otherwise")),
                    None => return Err(format!("{other}: \u{201c}{name}\u{201d} does not say what kind of thing it is. Say which with --as equipment, --as fixture or --as furnishing")),
                },
            };
            let made = plumbline_world::spec::item_of(&sheet, &called, page, kind);
            let path_made = plumbline_world::editing::item(&mut job, &name, made.item.clone()).map_err(|why| format!("{other}: {why}"))?;
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            say!("{path_made}: {name}, {} ({why}), from {called}, page {page}", kind.a());
            if !made.read.is_empty() {
                say!("  read off the sheet, each Assumed until it is checked against the thing:");
                for (what, value, line, note) in &made.read {
                    say!("    ~ {what:<7} {value:<22} \u{201c}{line}\u{201d}{}", note.as_ref().map(|n| format!("  ({n})")).unwrap_or_default());
                }
            }
            if !made.missing.is_empty() {
                say!("  not on the sheet, and Unknown until entered:");
                for what in &made.missing {
                    say!("    ? {what}");
                }
                if made.missing.iter().any(|m| matches!(*m, "width" | "depth" | "height")) {
                    say!("  Without all three sizes it has no shape: placed, it is not drawn solid and not tested for clashes. Enter one with: plumbline measure {path} {path_made} <width|depth|height> \"34 in\" --source \"<where from>\"");
                }
            }
            Ok(())
        }
        // The measured marks of a marked-up PDF as lines of the estimate (call 227).
        ("import", "marks") => {
            only(&["--page", "--counts"])?;
            let page = marks::page_of(flag("--page"))?;
            let (said, changed) = marks::import(&mut job, other, page, flags.iter().any(|(f, _)| *f == "--counts"))?;
            if changed {
                std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            }
            let _ = std::io::stdout().write_all(said.as_bytes());
            Ok(())
        }
        ("import", "prior") => {
            only(&["--sheet", "--name", "--status", "--source"])?;
            let (status, source) = how.clone().ok_or("say how sure the prior estimate is and where it came from: --status, --source")?;
            let sheet = table(flag("--sheet"))?;
            let name = match flag("--name") {
                Some(n) => n.to_string(),
                None => std::path::Path::new(other).file_stem().and_then(|s| s.to_str()).unwrap_or("prior estimate").to_string(),
            };
            let (made, brought) = tables::prior_in(&mut job, &name, &sheet.rows, Some(&sheet.numbers), plumbline_world::editing::Entered { status, source: &source }).map_err(|why| format!("{other}: {why}"))?;
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            say!("{made}: {name}");
            let _ = std::io::stdout().write_all(brought_said(other, &sheet, &brought).as_bytes());
            Ok(())
        }
        ("export", "prices") => {
            only(&[])?;
            let rows = tables::prices_out(&mut job);
            std::fs::write(other, table_bytes(other, "Prices", &rows)?).map_err(|e| format!("cannot write {other}: {e}"))?;
            say!("wrote {other}: the price list of {path}, {} under its headings", count(rows.len() - 1, "row", "rows"));
            Ok(())
        }
        ("import", "prices") => {
            only(&["--sheet", "--supplier", "--add", "--status", "--source"])?;
            let sheet = table(flag("--sheet"))?;
            let entered = how.as_ref().map(|(status, source)| plumbline_world::editing::Entered { status: *status, source });
            let brought = tables::prices_in(&mut job, &sheet.rows, Some(&sheet.numbers), flag("--supplier"), flags.iter().any(|(f, _)| *f == "--add"), entered).map_err(|why| format!("{other}: {why}"))?;
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            let _ = std::io::stdout().write_all(brought_said(other, &sheet, &brought).as_bytes());
            Ok(())
        }
        // The building as a model another trade's or a designer's program reads.
        ("export", "ifc") => {
            only(&[])?;
            if !other.to_ascii_lowercase().ends_with(".ifc") {
                return Err(format!("{other}: a model is written as an .ifc file; name the file with that"));
            }
            let file = std::path::Path::new(other).file_name().and_then(|f| f.to_str()).unwrap_or("model.ifc");
            let written = plumbline_ifc::write::export(&mut job, file, &plumbline_world::job::now())?;
            std::fs::write(other, &written.text).map_err(|e| format!("cannot write {other}: {e}"))?;
            say!("wrote {other}: the building of {path} as an IFC4 model, lengths in feet");
            // One of a thing is said as one: "1 roof", "1 piece of equipment".
            let one = |what: &str| what.strip_prefix("pieces ").map(|rest| format!("piece {rest}")).unwrap_or_else(|| what.strip_suffix('s').unwrap_or(what).to_string());
            say!("  written: {}", written.made.iter().map(|(what, n)| format!("{n} {}", if *n == 1 { one(what) } else { what.clone() })).collect::<Vec<_>>().join(", "));
            say!("  every value entered on each goes with it, with how sure it is and where it came from");
            if !written.not_written.is_empty() {
                say!("  not written (a building model has no place for it, or it has no shape):");
                // The model says each in a builder's words; here, at the command line, the command that writes the estimate out is
                // added to its line (call 209 point 3). The window adds where it is: Estimate workbook, under Write out.
                for ((what, ..), line) in written.not_written.iter().zip(written.not_written_said()) {
                    if what == "cost line" {
                        say!("    {line} (plumbline export estimate <job> <out.xlsx>)");
                    } else {
                        say!("    {line}");
                    }
                }
            }
            Ok(())
        }
        // The house as one file to look at anywhere (call 229): glTF, one named part for each thing the BIM file writes a body for.
        ("export", "gltf") => {
            only(&[])?;
            if !other.to_ascii_lowercase().ends_with(".glb") {
                return Err(format!("{other}: the house is written as a .glb file; name the file with that"));
            }
            let written = plumbline_ifc::gltf::export(&mut job)?;
            std::fs::write(other, &written.bytes).map_err(|e| format!("cannot write {other}: {e}"))?;
            let one = |what: &str| what.strip_prefix("pieces ").map(|rest| format!("piece {rest}")).unwrap_or_else(|| what.strip_suffix('s').unwrap_or(what).to_string());
            say!("wrote {other}: the house of {path} as {} parts in one glTF file, in metres, up as the format has it", written.parts);
            say!("  written: {}", written.made.iter().map(|(what, n)| format!("{n} {}", if *n == 1 { one(what) } else { what.clone() })).collect::<Vec<_>>().join(", "));
            say!("  each part is named as the clash list names it, and coloured by what the estimate finishes it in");
            say!("  not carried:");
            for line in &written.not_carried {
                say!("    {line}");
            }
            Ok(())
        }
        // The clashes as an issue list another program opens (call 228): each names its two things as the BIM file does.
        ("export", "bcf") => {
            only(&["--holes"])?;
            let lower = other.to_ascii_lowercase();
            if !(lower.ends_with(".bcfzip") || lower.ends_with(".bcf")) {
                return Err(format!("{other}: an issue list is written as a .bcfzip file; name the file with that"));
            }
            let holes = flags.iter().any(|(f, _)| *f == "--holes");
            let written = plumbline_ifc::bcf::export(&mut job, holes, &plumbline_world::job::now())?;
            std::fs::write(other, &written.bytes).map_err(|e| format!("cannot write {other}: {e}"))?;
            let issues = |n: usize| plumbline_world::view::so_many(n, "issue");
            if written.issues.is_empty() {
                say!("wrote {other}: the clashes of {path} as an issue list (BCF 2.1): there was no clash in the same space to hand over, so the file has no issue");
            } else {
                let clashes = written.findings + written.questions;
                let what = if written.holes > 0 { format!("{} to cut", plumbline_world::view::so_many(written.holes, "hole")) } else { String::new() };
                say!("wrote {other}: {} from the clashes of {path}, as an issue list (BCF 2.1)", issues(written.issues.len()));
                say!("  in the same space: {clashes} ({} finding{}, {} to verify in the field){}", written.findings, if written.findings == 1 { "" } else { "s" }, written.questions, if what.is_empty() { String::new() } else { format!("; and {what}") });
                say!("  each names its two things by the identifiers the IFC file written from the same job gives them, and has a view: where to stand and what to look at");
            }
            if written.holes_left > 0 {
                say!("  not handed over: {} to cut (a run through a wall or a floor is work to do, not a clash; --holes adds them)", plumbline_world::view::so_many(written.holes_left, "hole"));
            }
            if written.set_against > 0 {
                say!("  not handed over: {} set against a wall (not a clash)", written.set_against);
            }
            if !written.issues.is_empty() {
                say!("  not carried: a picture of the place; what each rests on beyond what the issue says; prices other than the dollars riding on a question");
            }
            Ok(())
        }
        // The estimate, or one lens on it, as a workbook (its figures formulas) or as
        // rows of values: by the name of the file asked for.
        ("export", "estimate" | "supplier" | "subcontractor" | "sub" | "pm") => {
            use plumbline_world::export::{self, Lens, Out};
            only(if matches!(*what, "subcontractor" | "sub") { &["--division", "--system"][..] } else { &[] })?;
            let workbook = match std::path::Path::new(other).extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase).as_deref() {
                Some("xlsx") => true,
                Some("csv") => false,
                _ => return Err(format!("{other}: what is written is a workbook (.xlsx) or rows of values (.csv); name the file with one of those")),
            };
            let (sheets, said) = match Lens::parse(what) {
                None => (export::estimate_out(&mut job, workbook), "the estimate"),
                Some(Lens::Supplier) => (export::lens_out(&mut job, Lens::Supplier), "the materials to quote, with none of our prices"),
                Some(Lens::Subcontractor) => (export::lens_out_for(&mut job, Lens::Subcontractor, flag("--division"), flag("--system"))?, "the subcontract scope, with none of our prices"),
                Some(Lens::Pm) => (export::lens_out(&mut job, Lens::Pm), "the lines, the budget by cost type and by division, and what is to verify, with no overhead, profit or price"),
            };
            let lines: usize = sheets.first().map(|s| s.rows.len().saturating_sub(s.head + 1)).unwrap_or(0);
            if workbook {
                let (bytes, formulas) = plumbline_sheets::workbook(&sheets)?;
                std::fs::write(other, bytes).map_err(|e| format!("cannot write {other}: {e}"))?;
                say!("wrote {other}: {said} of {path}; {}: {}; {}", count(sheets.len(), "sheet", "sheets"), sheets.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join(", "), if formulas == 0 { "figures, no formulas".to_string() } else { count(formulas, "figure that is a formula", "figures that are formulas") });
            } else {
                // The estimate as values is its lines under their headings and nothing
                // else, so it reads back as an estimate made before.
                let rows = match Lens::parse(what) {
                    None => sheets[0].rows[sheets[0].head..].iter().map(|r| r.iter().map(Out::said).collect()).collect(),
                    Some(_) => export::values(&sheets),
                };
                std::fs::write(other, plumbline_sheets::csv(&rows)).map_err(|e| format!("cannot write {other}: {e}"))?;
                say!("wrote {other}: {said} of {path}, {} under its headings", count(lines, "row", "rows"));
            }
            Ok(())
        }
        ("export", _) => Err(format!("export {what}: what is written out is prices, assemblies, items, estimate, supplier, subcontractor, pm, ifc, gltf or bcf\n{}", usage())),
        _ => Err(format!("import {what}: what is brought in is prices, assemblies, items, prior (an estimate made before) or ifc (a model)\n{}", usage())),
    }
}

/// Each prior estimate in the job set against the job's own lines.
/// Two or three layouts side by side, as `view::options` has them: a column for each, and the
/// last less the first. Every figure is the model's; this only sets them in columns.
pub(super) fn options_said(table: &view::Options, files: &[String]) -> String {
    let mut out = String::from("flags:  ~ Assumed    ? Unknown    (no mark) Known\n\n");
    let last = table.titles.len() - 1;
    // Where two layouts have one title, each column is headed by its title and its file's
    // name (call 157): two job costs are not set side by side under the same words.
    let one_title = table.titles.iter().enumerate().any(|(i, t)| table.titles[..i].contains(t));
    let heads: Vec<String> = table
        .titles
        .iter()
        .zip(files)
        .map(|(t, f)| if one_title { format!("{t} ({})", std::path::Path::new(f).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| f.clone())) } else { t.clone() })
        .collect();
    let diff_head = format!("{} less {}", heads[last], heads[0]);
    let label_w = table.rows.iter().map(|r| r.label.chars().count()).max().unwrap_or(0).max(8);
    let widths: Vec<usize> = (0..table.titles.len())
        .map(|i| table.rows.iter().map(|r| r.figures[i].said(r.unit).chars().count()).max().unwrap_or(0).max(heads[i].chars().count()))
        .collect();
    let diff_w = table.rows.iter().map(|r| r.difference_said().chars().count()).max().unwrap_or(0).max(diff_head.chars().count());
    let mut head = format!("{:<label_w$}", "");
    for (i, t) in heads.iter().enumerate() {
        let _ = write!(head, "  {:>w$}", t, w = widths[i]);
    }
    let _ = write!(head, "  {:>diff_w$}", diff_head);
    let _ = writeln!(out, "{head}");
    let mut group = None;
    for r in &table.rows {
        if group.is_some() && group != Some(r.group) {
            out.push('\n');
        }
        group = Some(r.group);
        let mut line = format!("{:<label_w$}", r.label);
        for (i, f) in r.figures.iter().enumerate() {
            let _ = write!(line, "  {:>w$}", f.said(r.unit), w = widths[i]);
        }
        let _ = write!(line, "  {:>diff_w$}", r.difference_said());
        let _ = writeln!(out, "{line}");
    }
    out
}

/// `plumbline price-item <file> "<item>" '$180' --status <...> --source "<...>"`: one price on
/// the line named for an item, on every one of it set in the job (call 157).
pub(super) fn price_item(args: &[String]) -> Result<(), String> {
    use plumbline_world::editing as ed;
    let [_, path, item, price, rest @ ..] = args else {
        return Err("price-item takes a file, an item's name, a price and then --status and --source: plumbline price-item <file> \"B18\" '$180' --status Assumed --source \"supplier quote\"".into());
    };
    let (mut status, mut source) = (None, None);
    let mut it = rest.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--status" if status.is_none() => status = Some(it.next().cloned().ok_or("--status needs a value")?),
            "--source" if source.is_none() => source = Some(it.next().cloned().ok_or("--source needs a value")?),
            "--status" | "--source" => return Err(format!("{a} given twice")),
            other => return Err(format!("price-item does not take {other:?}: it takes --status and --source")),
        }
    }
    let status = match status.as_deref() {
        Some("Known") => Status::Known,
        Some("Assumed") => Status::Assumed,
        Some("Unknown") => Status::Unknown,
        Some(other) => return Err(format!("{other:?} is not a status; use Known, Assumed or Unknown")),
        None => return Err("--status is required: say whether the price is Known, Assumed or Unknown".into()),
    };
    let source = source.ok_or("--source is required: say where the price came from")?;
    let (mut job, _) = read(path)?;
    let done = ed::price_item(&mut job, item, price, status, &source)?;
    if done.priced.is_empty() && done.without.is_empty() {
        say!("none of {} is set in the job yet, so nothing was priced: a price goes on each one set, and one set later takes none", done.item);
        return Ok(());
    }
    std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
    say!("priced {} {} at {price} ({status:?}): {}", done.priced.len(), done.item, done.priced.join(", "));
    if !done.without.is_empty() {
        say!("  {} of {} {} no line of that name and took nothing: {}", done.without.len(), done.item, if done.without.len() == 1 { "has" } else { "have" }, done.without.join(", "));
    }
    say!("  one of {} set later takes no price: run this again then", done.item);
    Ok(())
}

/// The price list as the model gives its rows (call 207): a row for each material and item that a line is made of or that has a
/// price, no price first, each with its lines, its things and the dollars riding. Nothing here is counted or summed.
pub(super) fn price_list_said(job: &mut plumbline_world::Job) -> String {
    use plumbline_world::pricelist::{price_list, PriceCell};
    let list = price_list(job);
    let mut out = String::from("flags:  ~ Assumed    ? Unknown    (no mark) Known\n");
    if list.rows.is_empty() {
        out.push_str("The price list is empty: no line of this job is made of a material or an item, and none has a price.\n");
    }
    let price_said = |c: &PriceCell, unit: &str| -> String {
        let at = match c.value {
            Some(x) => format!("${}/{unit}", view::rate(x)),
            None => "no price".to_string(),
        };
        format!("{} {at} ({}){}", c.status.flag(), c.source, if c.at_nothing { "  ! $0.00 entered: nothing is charged for it" } else { "" })
    };
    for r in &list.rows {
        let _ = writeln!(out, "\n{}  [{}]  ({})  bought by the {}", r.name, r.what, r.path, r.unit);
        let lines = if r.lines == 0 {
            "no line of the job is made of it".to_string()
        } else {
            format!("{} on {}{}", view::so_many(r.lines, "line"), view::so_many(r.things, "thing"), if r.waiting > 0 { format!("; {} with no figure", r.waiting) } else { String::new() })
        };
        let riding = |c: &PriceCell| match c.riding {
            Some(x) => format!("   {}{} riding{}", c.riding_status.flag().replace(' ', ""), view::money(x), if c.lines_at_it < r.lines { format!(" (on {} of {} lines)", c.lines_at_it, r.lines) } else { String::new() }),
            None => String::new(),
        };
        let _ = writeln!(out, "    estimator   {}{}", price_said(&r.price, &r.unit), riding(&r.price));
        for (who, c) in list.suppliers.iter().zip(&r.suppliers) {
            match c {
                Some(c) => {
                    let _ = writeln!(out, "    {who:<11} {}{}", price_said(c, &r.unit), riding(c));
                }
                None => {
                    let _ = writeln!(out, "    {who:<11} has not quoted it");
                }
            }
        }
        let _ = writeln!(out, "    lines       {lines}{}", if r.lines_off > 0 { format!("; {} switched off", r.lines_off) } else { String::new() });
    }
    for (name, n) in &list.not_found {
        let _ = writeln!(out, "\n? {} made of {}, which the job has no material or item called: priced by none of the rows above", view::so_many(*n, "line"), if name.is_empty() { "something it does not say".to_string() } else { format!("{name:?}") });
    }
    out
}

pub(super) fn compared_said(job: &mut plumbline_world::Job) -> String {
    use plumbline_world::tables;
    let money = view::money;
    let signed = |x: f64| format!("{}${}", if x < 0.0 { "-" } else { "+" }, view::figure(x.abs(), 2));
    let est = view::estimate(job);
    let all = tables::compare(job);
    let mut out = String::from("flags:  ~ Assumed    ? Unknown    (no mark) Known\n");
    if all.is_empty() {
        out.push_str("No prior estimate has been brought into this job: plumbline import prior <file> <estimate.csv> --status Assumed --source \"...\"\n");
    }
    for c in all {
        let _ = writeln!(out, "\n{}{} [{}], set against this job  ({})", match c.status { Status::Known => "", Status::Assumed => "~ ", Status::Unknown => "? " }, c.name, c.path, c.source);
        let _ = writeln!(out, "  LINE BY LINE (each prior line beside every line of this job called the same)");
        for l in &c.lines {
            let p = &l.prior;
            let prior_qty = p.quantity.map(|q| format!("{} {}", view::figure(q, 2), p.unit).trim().to_string()).unwrap_or_else(|| "no quantity".into());
            // A rate to the places it is held to: $12.345 is not said as $12.35.
            let rate = |x: f64| {
                let places = (2..=4).find(|p| ((x * 10f64.powi(*p)).round() / 10f64.powi(*p) - x).abs() < 1e-9).unwrap_or(4);
                format!("${}", view::figure(x, places as usize))
            };
            let prior_rate = p.unit_cost.map(|x| format!(" at {}", rate(x))).unwrap_or_default();
            let prior_amount = l.prior_amount.map(|a| format!("{}{}", money(a), if l.worked { " (its quantity times its unit cost)" } else { "" })).unwrap_or_else(|| "no amount".into());
            let _ = writeln!(out, "  {}{}", p.description, if l.of > 1 { format!("  ({} lines of the prior estimate go by this name; they are added together)", l.of) } else { String::new() });
            let _ = writeln!(out, "      prior:     {prior_qty}{prior_rate}: {prior_amount}");
            match &l.job {
                None => {
                    let _ = writeln!(out, "      this job:  has no line called that");
                }
                Some(j) => {
                    let flag = match j.status { Status::Known => "", Status::Assumed => "~", Status::Unknown => "?" };
                    let qty = match j.quantity {
                        Some(q) => format!("{flag}{} {}", view::figure(q, 2), j.unit),
                        None if j.unit.is_empty() => "its lines are in more than one unit and do not add".to_string(),
                        None => "no quantity".to_string(),
                    };
                    let amount = j.amount.map(|a| format!("{flag}{}", money(a))).unwrap_or_else(|| format!("no figure: {}", j.why));
                    let _ = writeln!(out, "      this job:  {qty}: {amount}  ({} on {})", count(j.on.len(), "line", "lines"), j.on.join(", "));
                    // A difference is as sure as the less sure of the two it is between.
                    let flag = match j.status.max(c.status) { Status::Known => "", Status::Assumed => "~", Status::Unknown => "?" };
                    let mut differs = Vec::new();
                    if let (Some(pq), Some(jq)) = (p.quantity, j.quantity) {
                        // One unit written two ways is one unit: SQFT is SF, LIN FT is LF.
                        let plain = |u: &str| {
                            let u: String = u.to_uppercase().chars().filter(|c| c.is_ascii_alphanumeric()).collect();
                            match u.as_str() {
                                "SQFT" | "SQUAREFEET" | "SQUAREFOOT" | "SFT" => "SF".to_string(),
                                "LNFT" | "LINFT" | "LINEARFEET" | "LINEALFEET" | "LFT" => "LF".to_string(),
                                "EACH" => "EA".to_string(),
                                "SQUARE" | "SQUARES" => "SQ".to_string(),
                                "CUYD" | "CUBICYARD" | "CUBICYARDS" | "YD3" => "CY".to_string(),
                                "HRS" | "HOUR" | "HOURS" => "HR".to_string(),
                                _ => u,
                            }
                        };
                        if plain(&p.unit) == plain(&j.unit) {
                            differs.push(format!("quantity {flag}{}{} {}", if jq - pq < 0.0 { "-" } else { "+" }, view::figure((jq - pq).abs(), 2), j.unit));
                        } else {
                            differs.push(format!("the prior line is in {} and this job's in {}: the quantities are not compared", if p.unit.is_empty() { "no unit" } else { &p.unit }, j.unit));
                        }
                    }
                    if let (Some(pa), Some(ja)) = (l.prior_amount, j.amount) {
                        differs.push(format!("amount {flag}{}", signed(ja - pa)));
                    }
                    if !differs.is_empty() {
                        let _ = writeln!(out, "      this job less the prior: {}", differs.join("; "));
                    }
                }
            }
        }
        let _ = writeln!(out, "  IN THIS JOB AND NOT IN THE PRIOR ESTIMATE: {}", c.only_in_job.len());
        for (description, on, amount, status) in &c.only_in_job {
            let flag = match status { Status::Known => "", Status::Assumed => "~", Status::Unknown => "?" };
            let _ = writeln!(out, "    {description}: {}  ({})", amount.map(|a| format!("{flag}{}", money(a))).unwrap_or_else(|| "no figure".into()), on.join(", "));
        }
        let unmatched: Vec<&tables::ComparedLine> = c.lines.iter().filter(|l| l.job.is_none()).collect();
        let _ = writeln!(out, "  IN THE PRIOR ESTIMATE AND NOT IN THIS JOB: {}", unmatched.len());
        for l in unmatched {
            let _ = writeln!(out, "    {}: {}", l.prior.description, l.prior_amount.map(money).unwrap_or_else(|| "no amount".into()));
        }
        let _ = writeln!(out, "  the prior estimate's lines add to {}{}", money(c.prior_total), if c.prior_left_out > 0 { format!(", leaving out {} with no amount", count(c.prior_left_out, "line", "lines")) } else { String::new() });
        let job_cost = match est.job_cost.value {
            Some(x) => format!("{}{}", match est.job_cost.status { Status::Known => "", Status::Assumed => "~", Status::Unknown => "?" }, money(x)),
            None => "no figure".to_string(),
        };
        let _ = writeln!(out, "  this job's cost is {job_cost}; nothing of the prior estimate is in it");
    }
    out
}

/// A table as it is written to a file: a workbook where the name ends .xlsx, CSV where it does not. (A workbook is a workbook: it was CSV text under
/// a name ending .xlsx, which no spreadsheet opens, and the importing side called it CSV; call 213 point 10.)
pub(super) fn table_bytes(out: &str, name: &str, rows: &[Vec<String>]) -> Result<Vec<u8>, String> {
    if out.to_lowercase().ends_with(".xlsx") {
        let widths = rows.first().map(|h| h.iter().map(|c| (c.chars().count() as f64 + 6.0).clamp(10.0, 40.0)).collect()).unwrap_or_default();
        let tab = plumbline_sheets::Tab { name: name.into(), rows: rows.iter().map(|r| r.iter().map(|c| if c.is_empty() { plumbline_sheets::Cell::Empty } else { plumbline_sheets::Cell::Text(c.clone()) }).collect()).collect(), head: 0, widths };
        plumbline_sheets::xlsx(&[tab])
    } else {
        Ok(plumbline_sheets::csv(rows).into_bytes())
    }
}

pub(super) fn count(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// What bringing a table in did, said so that every row and column of the file is
/// accounted for: used, the same as the job already had, or not used and why.
pub(super) fn brought_said(file: &str, sheet: &plumbline_sheets::Sheet, b: &plumbline_world::tables::Brought) -> String {
    let from = if sheet.name.is_empty() { sheet.sort.said().to_string() } else { format!("{}, sheet {:?}", sheet.sort.said(), sheet.name) };
    format!("{file} ({from}): {} under its headings\n{}", count(b.rows, "row", "rows"), brought_body(b, &sheet.lost))
}

/// What bringing a table in did, line by line: the same words for one table brought in alone and for each tab of a database.
pub(super) fn brought_body(b: &plumbline_world::tables::Brought, lost: &[String]) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "  set: {}", b.done.len());
    for d in &b.done {
        let _ = writeln!(out, "    {d}");
    }
    let _ = writeln!(out, "  the same as the job already has: {}", b.same);
    let _ = writeln!(out, "  not used: {}", b.not_used.len());
    for (row, why) in &b.not_used {
        let _ = writeln!(out, "    row {row}: {why}");
    }
    for note in &b.notes {
        let _ = writeln!(out, "  NOTE: {note}");
    }
    if b.above > 0 {
        let _ = writeln!(out, "  not read: {} above the headings", count(b.above, "row", "rows"));
    }
    for c in &b.columns_not_used {
        let _ = writeln!(out, "  not read: the column {c:?}");
    }
    for lost in lost {
        let _ = writeln!(out, "  not read: {lost}");
    }
    out
}

/// "My database" written out: one workbook with a tab for each table, or a folder with a CSV file for each. Each is the table's own
/// writing-out (`plumbline_world::database::out`).
pub(super) fn database_out(job: &mut plumbline_world::Job, path: &str, other: &str) -> Result<(), String> {
    let all = plumbline_world::database::out(job);
    let said: Vec<String> = all.iter().map(|(name, rows)| format!("{name} {}", rows.len() - 1)).collect();
    if other.to_lowercase().ends_with(".xlsx") {
        let tabs: Vec<plumbline_sheets::Tab> = all
            .iter()
            .map(|(name, rows)| {
                let widths = rows.first().map(|h| h.iter().map(|c| (c.chars().count() as f64 + 6.0).clamp(10.0, 40.0)).collect()).unwrap_or_default();
                plumbline_sheets::Tab { name: name.to_string(), rows: rows.iter().map(|r| r.iter().map(|c| if c.is_empty() { plumbline_sheets::Cell::Empty } else { plumbline_sheets::Cell::Text(c.clone()) }).collect()).collect(), head: 0, widths }
            })
            .collect();
        std::fs::write(other, plumbline_sheets::xlsx(&tabs)?).map_err(|e| format!("cannot write {other}: {e}"))?;
        say!("wrote {other}: my database from {path}, a tab for each table: {}", said.join(", "));
    } else {
        if std::path::Path::new(other).is_file() {
            return Err(format!("{other} is a file: my database is written to a workbook (name it with .xlsx) or to a folder"));
        }
        std::fs::create_dir_all(other).map_err(|e| format!("cannot make the folder {other}: {e}"))?;
        for (name, rows) in &all {
            let file = std::path::Path::new(other).join(format!("{}.csv", name.to_lowercase()));
            std::fs::write(&file, plumbline_sheets::csv(rows)).map_err(|e| format!("cannot write {}: {e}", file.display()))?;
        }
        say!("wrote {other}: my database from {path}, a file for each table: {}", said.join(", "));
    }
    Ok(())
}

/// My database brought in, whole: a workbook's tabs, the tables of a folder, or the one the program comes with (`starter`).
pub(super) fn database_in(job: &mut plumbline_world::Job, other: &str) -> Result<String, String> {
    use plumbline_world::database::{self, Given};
    let starter = other == "starter" && !std::path::Path::new(other).exists();
    let (given, from): (Vec<Given>, String) = if starter {
        let mut given = Vec::new();
        for (name, text) in database::starter_files() {
            let sheet = plumbline_sheets::read(text.as_bytes(), None).map_err(|why| format!("the starter {name}: {why}"))?;
            given.push(Given { name: name.to_string(), rows: sheet.rows, numbers: sheet.numbers, lost: Vec::new() });
        }
        (given, "the starter database".to_string())
    } else if std::path::Path::new(other).is_dir() {
        let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(other).map_err(|e| format!("cannot read {other}: {e}"))?.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.is_file()).collect();
        files.sort();
        let mut given = Vec::new();
        for file in files {
            let name = file.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
            let called = file.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
            if !called.to_lowercase().ends_with(".csv") {
                // A file that is not a table is named with the tabs it is not read as.
                given.push(Given { name: called, rows: Vec::new(), numbers: Vec::new(), lost: Vec::new() });
                continue;
            }
            let bytes = std::fs::read(&file).map_err(|e| format!("cannot read {}: {e}", file.display()))?;
            let sheet = plumbline_sheets::read(&bytes, None).map_err(|why| format!("{called}: {why}"))?;
            given.push(Given { name, rows: sheet.rows, numbers: sheet.numbers, lost: sheet.lost });
        }
        let from = format!("{other} (a folder, {})", count(given.len(), "file", "files"));
        (given, from)
    } else {
        let bytes = std::fs::read(other).map_err(|e| format!("cannot read {other}: {e}"))?;
        if plumbline_sheets::sort_of(&bytes) != Some(plumbline_sheets::Sort::Xlsx) {
            return Err(format!("{other} is one table: my database is a workbook with a tab for each table, or a folder of tables. For one table, import items, shapes, connections, assemblies or prices"));
        }
        let given: Vec<Given> = plumbline_sheets::read_tabs(&bytes).map_err(|why| format!("{other}: {why}"))?.into_iter().map(|s| Given { name: s.name, rows: s.rows, numbers: s.numbers, lost: s.lost }).collect();
        let from = format!("{other} (a workbook, {})", count(given.len(), "tab", "tabs"));
        (given, from)
    };
    let whole = database::bring_in(job, &given, starter).map_err(|why| format!("{other}: {why}"))?;
    let mut out = format!("{from}: brought in whole\n");
    let mut added = 0usize;
    // What a workbook says of every tab alike (how its cells are shown) is said once, after the tabs, and not under each (call 226 point 3).
    let mut said_once: Vec<String> = Vec::new();
    for t in &whole.tabs {
        let _ = writeln!(out, "the tab {}: {} under its headings", t.name, count(t.brought.rows, "row", "rows"));
        let (alike, own): (Vec<String>, Vec<String>) = t.lost.iter().cloned().partition(|l| l.starts_with("how its cells are shown"));
        for l in alike {
            if !said_once.contains(&l) {
                said_once.push(l);
            }
        }
        out.push_str(&brought_body(&t.brought, &own));
        if t.kept > 0 {
            let _ = writeln!(out, "  in the job already, and left as the job has them: {}", t.kept);
        }
        added += t.brought.done.len();
    }
    for l in &said_once {
        let _ = writeln!(out, "not read: {l}");
    }
    // A tab the program has no table for is named as it was typed, in the quotes a name goes in, and not as a program writes a string.
    for name in &whole.not_read {
        let _ = writeln!(out, "not read: the tab \"{name}\": this program has no table of that name");
    }
    for name in &whole.absent {
        let _ = writeln!(out, "no tab for {name}: what the job has is left as it is");
    }
    let _ = writeln!(out, "in all: {} set; no row refused", added);
    Ok(out)
}
