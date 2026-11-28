use super::*;

// ---------------------------------------------------------------- where an item connects

/// A connections table's headings as this program writes them (call 211): one row for each connection of an item, in the order
/// the item holds them. What it takes (pipe, duct, conduit, power), what a pipe carries (cold, hot, waste, vent, storm, gas),
/// which way it goes (in, out, both), where it is as three figures, **across, back and up, the same three a block of a shape is
/// written in** (across from the middle, back from the back, up from the foot, each a formula in `width`, `depth` and `height`),
/// which way it faces (back, front, left, right, up, down), and how sure and where from, once for the place and once for the size.
pub const CONNECTION_HEADINGS: [&str; 14] = ["Item", "Connection", "Takes", "Carries", "Goes", "Across", "Back", "Up", "Faces", "Place: how sure", "Place: where from", "Size", "Size: how sure", "Size: where from"];

pub(super) const CONNECTION_COLUMNS: [(&str, &[&str]); 16] = [
    ("item", &["item"]),
    ("connection", &["connection", "connection name"]),
    ("takes", &["takes", "takes a"]),
    ("carries", &["carries"]),
    ("goes", &["goes", "flow"]),
    ("across", &["across"]),
    ("back", &["back"]),
    ("up", &["up"]),
    ("faces", &["faces", "facing"]),
    ("place status", &["place: how sure"]),
    ("place source", &["place: where from"]),
    ("size", &["size"]),
    ("size status", &["size: how sure"]),
    ("size source", &["size: where from"]),
    ("status", &["how sure", "status"]),
    ("source", &["where from", "source"]),
];

/// A row of a connections table read into the row of an item it says.
pub(super) fn read_connection(cols: &Columns, row: &[String]) -> Result<crate::building::ItemConnection, String> {
    use crate::building::connections::{domain_word, flow_word, kind_word};
    use crate::{ConnectorKind, Domain, Flow};
    let name = cols.cell(row, "connection");
    if name.is_empty() {
        return Err("it has no name for the connection".into());
    }
    let takes = cols.cell(row, "takes");
    let kind = [ConnectorKind::Pipe, ConnectorKind::Duct, ConnectorKind::Conduit, ConnectorKind::Power].into_iter().find(|k| kind_word(*k).eq_ignore_ascii_case(takes)).ok_or_else(|| format!("it takes {takes:?}, which is none of pipe, duct, conduit or power"))?;
    let carries = match cols.cell(row, "carries") {
        "" => None,
        said => Some([Domain::Cold, Domain::Hot, Domain::Waste, Domain::Vent, Domain::Storm, Domain::Gas].into_iter().find(|d| domain_word(*d).eq_ignore_ascii_case(said)).ok_or_else(|| format!("it carries {said:?}, which is none of cold, hot, waste, vent, storm or gas"))?),
    };
    let goes = match cols.cell(row, "goes") {
        "" => Flow::Both,
        said => [Flow::In, Flow::Out, Flow::Both].into_iter().find(|f| flow_word(*f).eq_ignore_ascii_case(said)).ok_or_else(|| format!("it goes {said:?}, which is none of in, out or both"))?,
    };
    let figure = |what: &str| -> Result<String, String> {
        let cell = cols.cell(row, what);
        if cell.is_empty() {
            return Err(format!("its {what} has no figure"));
        }
        Ok(cell.to_string())
    };
    // An empty cell says nothing of which way it faces.
    let faces = match cols.cell(row, "faces") {
        "" => None,
        said => Some(crate::building::Facing::named(said).ok_or_else(|| format!("it faces {said:?}, which is none of back, front, left, right, up or down"))?),
    };
    // How sure and where from: the place's own two columns, else the row's one pair.
    let pair = |own: (&str, &str)| -> Result<(Status, String), String> {
        let (status, source) = if own.0.is_empty() && own.1.is_empty() { (cols.cell(row, "status"), cols.cell(row, "source")) } else { own };
        let status = status_read(status).ok_or_else(|| format!("how sure it is reads {status:?}, which is not Known, Assumed or Unknown"))?;
        if source.is_empty() {
            return Err("it does not say where it is from".into());
        }
        Ok((status, source.to_string()))
    };
    let (place, place_from) = pair((cols.cell(row, "place status"), cols.cell(row, "place source"))).map_err(|why| format!("its place: {why}"))?;
    let size = match cols.cell(row, "size") {
        "" => None,
        typed => {
            let (len, unit) = units::parse_length(typed).map_err(|why| format!("its size: {why}"))?;
            let (status, source) = pair((cols.cell(row, "size status"), cols.cell(row, "size source"))).map_err(|why| format!("its size: {why}"))?;
            Some(Val { value: Some(len), unit: unit.to_string(), status, source })
        }
    };
    Ok(crate::building::ItemConnection { name: name.to_string(), kind, carries, flow: goes, across: figure("across")?, back: figure("back")?, up: figure("up")?, faces, place, place_from, size })
}

/// The job's items' connections as a table: the headings, then a row for each connection of each item that has any, items in the
/// order they stand. Brought into a job with the same items it gives the same rows to the byte.
pub fn connections_out(job: &mut Job) -> Vec<Vec<String>> {
    use crate::building::connections::{domain_word, flow_word, kind_word};
    let mut rows: Vec<Vec<String>> = vec![CONNECTION_HEADINGS.iter().map(|h| h.to_string()).collect()];
    for e in job.order.clone() {
        let (Some(node), Some(held)) = (job.world.get::<Node>(e), job.world.get::<crate::ItemConnections>(e)) else { continue };
        for r in &held.0 {
            let (size, size_status, size_source) = match &r.size {
                Some(s) => (s.value.as_ref().map(|l| units::show_length(l, &s.unit)).unwrap_or_default(), status_said(s.status).to_string(), s.source.clone()),
                None => (String::new(), String::new(), String::new()),
            };
            rows.push(vec![
                node.name.clone(),
                r.name.clone(),
                kind_word(r.kind).to_string(),
                r.carries.map(|d| domain_word(d).to_string()).unwrap_or_default(),
                flow_word(r.flow).to_string(),
                r.across.clone(),
                r.back.clone(),
                r.up.clone(),
                r.faces.map(|f| f.word().to_string()).unwrap_or_default(),
                status_said(r.place).to_string(),
                r.place_from.clone(),
                size,
                size_status,
                size_source,
            ]);
        }
    }
    rows
}

/// Bring a connections table in. An item the job has gets the rows the table gives it, all of them or none: a row that cannot be
/// read, or a place that is not on or inside the item's box at its own sizes, leaves that item's connections as they were and
/// says which row and why; the other items still are used. An item with the same rows as it has is counted as the same. The
/// things set down from an item follow it, and a run that is left standing off its connection is counted (call 211). An item the
/// job has not got is not used: its rows say so. An item whose rows are not in the table keeps its own.
pub fn connections_in(job: &mut Job, rows: &[Vec<String>], numbers: Option<&[usize]>) -> Result<Brought, String> {
    let cols = columns(rows, &CONNECTION_COLUMNS, &["item", "connection", "takes", "across", "back", "up", "faces"]).ok_or_else(|| {
        "this is not a connections table this program can read: within its first rows it has no row of headings with Item, Connection, Takes, Across, Back, Up and Faces".to_string()
    })?;
    // Each item's rows, in the order they stand, by the item's plain name.
    let mut order: Vec<String> = Vec::new();
    let mut by: BTreeMap<String, (String, Vec<(usize, Result<crate::building::ItemConnection, String>)>)> = BTreeMap::new();
    let mut out = Brought { rows: rows.len().saturating_sub(cols.row + 1), columns_not_used: cols.not_used.clone(), above: cols.row, ..Brought::default() };
    for (i, row) in rows.iter().enumerate().skip(cols.row + 1) {
        let number = row_number(numbers, i);
        if row.iter().all(|c| c.trim().is_empty()) {
            out.rows -= 1;
            continue;
        }
        let item = cols.cell(row, "item");
        if item.is_empty() {
            out.not_used.push((number, "it has no item name".to_string()));
            continue;
        }
        let key = check::plain_name(item);
        if !by.contains_key(&key) {
            order.push(key.clone());
        }
        by.entry(key).or_insert_with(|| (item.to_string(), Vec::new())).1.push((number, read_connection(&cols, row)));
    }
    let have: std::collections::HashMap<String, String> = {
        let mut q = job.world.query::<(&Node, &crate::Item)>();
        q.iter(&job.world).map(|(n, _)| (check::plain_name(&n.name), n.path.clone())).collect()
    };
    all_or_nothing(job, |job| {
        for key in order {
            let (item, pieces) = by.remove(&key).unwrap_or_default();
            let numbers: Vec<usize> = pieces.iter().map(|(n, _)| *n).collect();
            // A reason a row gave of itself, without the item's name said again before it (each row says its own reason once).
            let own = |why: &str| why.strip_prefix(&format!("{item:?}: ")).unwrap_or(why).to_string();
            let say_all = |out: &mut Brought, first: usize, why: String| {
                for n in &numbers {
                    out.not_used.push((*n, if *n == first { format!("{item}: {}", own(&why)) } else { format!("{item}: its connections are left out: its rows cannot be used together (row {first} says why)") }));
                }
            };
            let Some(path) = have.get(&key).cloned() else {
                for n in &numbers {
                    out.not_used.push((*n, format!("{item}: the job has no item called that: bring the item in first")));
                }
                continue;
            };
            // Each row that cannot be used says why, once; the others of the item say how many cannot and which (call 213 point 10).
            let mut faults: Vec<(usize, String)> = Vec::new();
            for (n, p) in &pieces {
                match p {
                    Err(why) => faults.push((*n, own(why))),
                    Ok(row) => {
                        if let Some(why) = crate::editing::connections::row_fault(job, &path, row) {
                            faults.push((*n, why));
                        }
                    }
                }
            }
            if !faults.is_empty() {
                let rows_said = {
                    let nums: Vec<String> = faults.iter().map(|(n, _)| n.to_string()).collect();
                    match nums.as_slice() {
                        [one] => format!("row {one}"),
                        [init @ .., last] => format!("rows {} and {last}", init.join(", ")),
                        [] => String::new(),
                    }
                };
                for n in &numbers {
                    out.not_used.push((*n, match faults.iter().find(|(f, _)| f == n) {
                        Some((_, why)) => format!("{item}: {why}"),
                        None => format!("{item}: its connections are left out, as {} of its rows cannot be used ({rows_said})", faults.len()),
                    }));
                }
                continue;
            }
            let wanted: Vec<crate::building::ItemConnection> = pieces.into_iter().filter_map(|(_, p)| p.ok()).collect();
            let was = crate::editing::item_rows(job, &path).unwrap_or_default();
            if was == wanted {
                out.same += 1;
                continue;
            }
            match crate::editing::connections::connections_replace(job, &path, wanted.clone()) {
                Ok(done) => {
                    let what = if was.is_empty() { "added" } else { "changed" };
                    let follows = match done.followed.len() {
                        0 => String::new(),
                        1 => "; 1 thing set down from it follows".to_string(),
                        n => format!("; {n} things set down from it follow"),
                    };
                    out.done.push(format!("{}: {what}: {}{follows}", done.item, crate::view::so_many(wanted.len(), "connection")));
                    let tail = crate::editing::ConnectionsFollowed { set_down: done.set_down, things: Vec::new(), left_alone: done.left_alone.clone(), off: done.off.clone() };
                    for line in tail.lines() {
                        out.notes.push(format!("{}: {line}", done.item));
                    }
                }
                Err(why) => say_all(&mut out, numbers[0], why),
            }
        }
        out.not_used.sort();
        Ok(out)
    })
}

/// The connections the program ships with, brought in: an item that has none gets the rows the table gives it, and an item that has
/// some keeps them (pressing it a second time writes over nothing a person changed, as `items_in_missing` does for items). Returns
/// what was done, and the names of the items left as the job has them.
pub fn connections_in_missing(job: &mut Job, rows: &[Vec<String>], numbers: Option<&[usize]>) -> Result<(Brought, Vec<String>), String> {
    let cols = columns(rows, &CONNECTION_COLUMNS, &["item", "connection", "takes", "across", "back", "up", "faces"]).ok_or_else(|| "this is not a connections table this program can read".to_string())?;
    let held: std::collections::HashSet<String> = {
        let mut q = job.world.query::<(&Node, &crate::ItemConnections)>();
        q.iter(&job.world).map(|(n, _)| check::plain_name(&n.name)).collect()
    };
    let (mut wanted, mut their_numbers, mut kept) = (Vec::new(), Vec::new(), Vec::new());
    for (i, row) in rows.iter().enumerate() {
        let name = cols.cell(row, "item");
        if i > cols.row && !name.is_empty() && held.contains(&check::plain_name(name)) {
            if !kept.iter().any(|k: &String| check::plain_name(k) == check::plain_name(name)) {
                kept.push(name.to_string());
            }
            continue;
        }
        wanted.push(row.clone());
        their_numbers.push(row_number(numbers, i));
    }
    let mut brought = connections_in(job, &wanted, Some(&their_numbers))?;
    // The rows of an item the job has connections for are rows too (call 213 point 10): the same as the job has, or not used because the job's own
    // stand. Every row of the file is counted as one of them, none as "no rows".
    let mut by_item: BTreeMap<String, Vec<(usize, Vec<String>)>> = BTreeMap::new();
    for (i, row) in rows.iter().enumerate().skip(cols.row + 1) {
        let name = cols.cell(row, "item");
        if !name.is_empty() && held.contains(&check::plain_name(name)) {
            by_item.entry(check::plain_name(name)).or_default().push((row_number(numbers, i), row.clone()));
        }
    }
    for (key, list) in by_item {
        let path = {
            let mut q = job.world.query::<(&Node, &crate::Item)>();
            q.iter(&job.world).find(|(n, _)| check::plain_name(&n.name) == key).map(|(n, _)| n.path.clone())
        };
        let table_rows: Vec<Result<crate::building::ItemConnection, String>> = list.iter().map(|(_, r)| read_connection(&cols, r)).collect();
        let have = path.as_deref().map(|p| crate::editing::item_rows(job, p).unwrap_or_default()).unwrap_or_default();
        let same = table_rows.iter().all(|r| r.is_ok()) && table_rows.iter().filter_map(|r| r.as_ref().ok()).cloned().collect::<Vec<_>>() == have;
        brought.rows += list.len();
        for (n, row) in &list {
            if same {
                brought.same += 1;
            } else {
                brought.not_used.push((*n, format!("{}: the job has connections for this item already, and keeps them", cols.cell(row, "item"))));
            }
        }
    }
    brought.not_used.sort();
    Ok((brought, kept))
}
