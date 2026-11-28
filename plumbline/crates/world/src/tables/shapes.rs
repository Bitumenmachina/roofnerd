use super::*;

// ---------------------------------------------------------------- shapes

/// A shape table's headings as this program writes them: one row for each block of a
/// shape, in the order it is drawn. Across is from the middle of the thing, back from its
/// back (the side that goes to a wall) toward the front, up from its foot; each figure is
/// a formula in `width`, `depth` and `height` (call 198).
pub const SHAPE_HEADINGS: [&str; 10] = ["Shape", "Made of", "Form", "Across from", "Across to", "Back from", "Back to", "Up from", "Up to", "There when"];

pub(super) const SHAPE_COLUMNS: [(&str, &[&str]); 10] = [
    ("shape", &["shape", "name", "drawn as"]),
    ("made", &["made of", "material", "made"]),
    ("form", &["form", "block is"]),
    ("across from", &["across from"]),
    ("across to", &["across to"]),
    ("back from", &["back from"]),
    ("back to", &["back to"]),
    ("up from", &["up from"]),
    ("up to", &["up to"]),
    ("when", &["there when", "when"]),
];

/// A shape table read into shapes, and the rows that were not used with why. A shape with
/// a row that cannot be read is not used at all (half a shape is a wrong drawing), and each
/// of its rows says so. Err only where the table is not a shape table.
pub fn shapes_from_rows(rows: &[Vec<String>], numbers: Option<&[usize]>) -> Result<(Vec<crate::building::shape::Shape>, Vec<(usize, String)>), String> {
    read_shapes(rows, numbers, false)
}

/// `shapes_from_rows`, and, where `a_job_brings_them`, a shape called as one of the program's starter shapes is not used unless its
/// blocks are the starter's word for word (call 205, point 3): a job's own shape may not take a starter's name.
pub(super) fn read_shapes(rows: &[Vec<String>], numbers: Option<&[usize]>, a_job_brings_them: bool) -> Result<(Vec<crate::building::shape::Shape>, Vec<(usize, String)>), String> {
    use crate::building::forms::Made;
    use crate::building::shape::{Piece, Shape};
    let cols = columns(rows, &SHAPE_COLUMNS, &["shape", "made", "across from", "across to", "back from", "back to", "up from", "up to"]).ok_or_else(|| {
        "this is not a shape table this program can read: within its first rows it has no row of headings with Shape, Made of, Across from, Across to, Back from, Back to, Up from and Up to".to_string()
    })?;
    // Each shape's rows in the order they stand, by the shape's name in small letters.
    let mut order: Vec<String> = Vec::new();
    let mut by: BTreeMap<String, (String, Vec<(usize, Result<Piece, String>)>)> = BTreeMap::new();
    let mut not_used: Vec<(usize, String)> = Vec::new();
    let mut last: Option<String> = None;
    let mut apart: Vec<String> = Vec::new();
    for (i, row) in rows.iter().enumerate().skip(cols.row + 1) {
        if row.iter().all(|c| c.trim().is_empty()) {
            continue;
        }
        let number = row_number(numbers, i);
        let name = cols.cell(row, "shape");
        if name.is_empty() {
            not_used.push((number, "it has no shape name".to_string()));
            continue;
        }
        let read = || -> Result<Piece, String> {
            let made = Made::named(cols.cell(row, "made")).ok_or_else(|| format!("it is made of {:?}, which is none of {}", cols.cell(row, "made"), Made::ALL.map(Made::word).join(", ")))?;
            let round = match cols.cell(row, "form").to_lowercase().as_str() {
                "" | "box" => false,
                "round" => true,
                other => return Err(format!("its form reads {other:?}: a block is a box or a round")),
            };
            let fig = |what: &str| -> Result<String, String> {
                let cell = cols.cell(row, what);
                if cell.is_empty() {
                    return Err(format!("its {what} has no figure"));
                }
                Ok(cell.to_string())
            };
            Ok(Piece { made, round, across: [fig("across from")?, fig("across to")?], back: [fig("back from")?, fig("back to")?], up: [fig("up from")?, fig("up to")?], when: Some(cols.cell(row, "when").to_string()).filter(|w| !w.is_empty()) })
        };
        let key = name.to_lowercase();
        if !by.contains_key(&key) {
            order.push(key.clone());
        } else if last.as_deref() != Some(key.as_str()) && !apart.contains(&key) {
            // Another shape's rows came between this shape's (call 206 point 12): they were gathered without a word.
            apart.push(key.clone());
        }
        last = Some(key.clone());
        by.entry(key).or_insert_with(|| (name.to_string(), Vec::new())).1.push((number, read()));
    }
    let mut shapes = Vec::new();
    for key in order {
        let (name, pieces) = by.remove(&key).unwrap_or_default();
        if apart.contains(&key) {
            let at: Vec<String> = pieces.iter().map(|(n, _)| n.to_string()).collect();
            for (n, _) in &pieces {
                not_used.push((*n, format!("{name}: its rows do not stand together (rows {} are its, and other rows stand between them): put one shape's rows next to one another", at.join(", "))));
            }
            continue;
        }
        let unread: Vec<(usize, String)> = pieces.iter().filter_map(|(n, p)| p.as_ref().err().map(|why| (*n, why.clone()))).collect();
        let shape = Shape { name: name.clone(), blocks: pieces.iter().filter_map(|(_, p)| p.as_ref().ok().cloned()).collect() };
        if a_job_brings_them {
            if let Some(starter) = crate::building::shape::starter_named(&name) {
                if !(unread.is_empty() && shape.blocks == starter.blocks) {
                    for (n, _) in &pieces {
                        not_used.push((*n, format!("{name}: {name:?} is a starter shape's name: a job's own shape may not take it; call the shape something else (a starter shape is not changed: keep it as a new shape under a name of its own)")));
                    }
                    continue;
                }
            }
        }
        // What each block of a shape that was read whole cannot be (its figure, its condition, the box):
        // said of the row it is in.
        let numbers: Vec<usize> = pieces.iter().map(|(n, _)| *n).collect();
        let faults: Vec<(usize, String)> = if unread.is_empty() {
            match shape.compile() {
                Err(why) if shape.faults().is_empty() => vec![(numbers[0], why)],
                _ => shape.faults().into_iter().map(|(at, why)| (numbers[at], why)).collect(),
            }
        } else {
            unread
        };
        if faults.is_empty() {
            shapes.push(shape);
        } else {
            let first = faults[0].0;
            for (n, _) in &pieces {
                not_used.push((*n, match faults.iter().find(|(f, _)| f == n) {
                    Some((_, why)) => format!("{name}: {why}"),
                    None => format!("{name}: the shape is left out, as row {first} of it cannot be used"),
                }));
            }
        }
    }
    not_used.sort();
    Ok((shapes, not_used))
}

/// The shapes the job's items say they are drawn as, as a table: the headings, then a row
/// for each block, shapes in the order their items stand. A shape the job carries is written
/// as the job has it, else as the program's starter has it. "box", and a form the program
/// draws without a table row (a sink, a bed), have no rows.
pub fn shapes_out(job: &mut Job) -> Vec<Vec<String>> {
    shapes_out_counted(job).0
}

/// The shapes as a table, and the names of the job's own shapes that no item is drawn as: they are written after the others, so that the
/// table is the one way a shape is carried to another job and loses none (call 213 point 13).
pub fn shapes_out_counted(job: &mut Job) -> (Vec<Vec<String>>, Vec<String>) {
    use crate::Item;
    let mut rows: Vec<Vec<String>> = vec![SHAPE_HEADINGS.iter().map(|h| h.to_string()).collect()];
    let mut done: Vec<String> = Vec::new();
    let put = |shape: &crate::building::shape::Shape, rows: &mut Vec<Vec<String>>| {
        for b in &shape.blocks {
            let [af, at, bf, bt, uf, ut] = b.figures().map(str::to_string);
            rows.push(vec![shape.name.clone(), b.made.word().to_string(), if b.round { "round" } else { "box" }.to_string(), af, at, bf, bt, uf, ut, b.when.clone().unwrap_or_default()]);
        }
    };
    for e in job.order.clone() {
        let Some(said) = job.world.get::<Item>(e).and_then(|it| it.form.clone()) else { continue };
        if done.iter().any(|d| d.eq_ignore_ascii_case(&said)) {
            continue;
        }
        done.push(said.clone());
        let Some(shape) = crate::building::shape::find(&job.shapes, &said) else { continue };
        put(shape, &mut rows);
    }
    let mut unused: Vec<String> = Vec::new();
    for shape in &job.shapes {
        if !done.iter().any(|d| d.eq_ignore_ascii_case(&shape.name)) {
            put(shape, &mut rows);
            unused.push(shape.name.clone());
        }
    }
    (rows, unused)
}

/// Bring a shape table in. A shape the job does not have is added; one it has, by name, is
/// changed to what the rows say, and says so; one that says what the job (or the program's
/// starter) already has is counted as the same. A shape that cannot be worked is not used and
/// each of its rows says why; the others still are.
pub fn shapes_in(job: &mut Job, rows: &[Vec<String>], numbers: Option<&[usize]>) -> Result<Brought, String> {
    use crate::building::shape;
    let (shapes, not_used) = read_shapes(rows, numbers, true)?;
    let cols = columns(rows, &SHAPE_COLUMNS, &["shape", "made", "across from", "across to", "back from", "back to", "up from", "up to"]);
    all_or_nothing(job, |job| {
        let mut out = Brought { rows: rows.len().saturating_sub(cols.as_ref().map(|c| c.row + 1).unwrap_or(0)), columns_not_used: cols.as_ref().map(|c| c.not_used.clone()).unwrap_or_default(), above: cols.as_ref().map(|c| c.row).unwrap_or(0), not_used, ..Brought::default() };
        for s in shapes {
            let held = job.shapes.iter().position(|j| j.name.eq_ignore_ascii_case(&s.name));
            match held {
                Some(at) if job.shapes[at].blocks == s.blocks => out.same += 1,
                Some(at) => {
                    // An item already drawn as it is held to the new blocks at its own sizes (call 206 point 2): the shape is not used
                    // where one would stand outside its box, and the item is named.
                    let offence = {
                        let mut q = job.world.query::<(&crate::Node, &crate::Item)>();
                        let drawn: Vec<(String, crate::Item)> = q.iter(&job.world).filter(|(_, it)| it.form.as_deref().is_some_and(|f| f.trim().eq_ignore_ascii_case(&s.name))).map(|(n, it)| (n.name.clone(), it.clone())).collect();
                        drawn.iter().find_map(|(n, it)| it.size_ft().ok().and_then(|(size, _, _)| s.check_at(n, size).err()))
                    };
                    if let Some(why) = offence {
                        let at_row = rows.iter().position(|r| r.first().is_some_and(|c| c.trim().eq_ignore_ascii_case(&s.name))).map(|i| numbers.and_then(|n| n.get(i).copied()).unwrap_or(i + 1)).unwrap_or(0);
                        out.not_used.push((at_row, format!("{}: {why}", s.name)));
                        continue;
                    }
                    out.done.push(format!("{}: changed: its {}", s.name, crate::view::so_many(s.blocks.len(), "block")));
                    job.shapes[at] = Shape { name: job.shapes[at].name.clone(), ..s };
                }
                None => {
                    // What the program ships with says the same: nothing to carry.
                    if shape::starter_named(&s.name).is_some_and(|t| t.blocks == s.blocks) {
                        out.same += 1;
                        continue;
                    }
                    if job.shapes.len() >= shape::MOST_SHAPES {
                        return Err(format!("a job carries at most {} shapes", shape::MOST_SHAPES));
                    }
                    out.done.push(format!("{}: added: {}", s.name, crate::view::so_many(s.blocks.len(), "block")));
                    job.shapes.push(s);
                }
            }
        }
        Ok(out)
    })
}
