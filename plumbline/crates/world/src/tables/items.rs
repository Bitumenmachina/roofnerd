use super::*;

// ---------------------------------------------------------------- an item library

/// An item library's headings, as this program writes them. A library is a table of
/// the things that can be placed, each on one row: what it is called, which trade's it
/// is, what kind of thing it is, what it is drawn as, its sizes, who makes it. It is
/// kept in a file a person can open, change and bring back in (Patrick, 2026-11-16:
/// "a workbench for items for trade that can be modified like a database of items").
pub const ITEM_HEADINGS: [&str; 26] = [
    "Item", "Trade", "Kind", "Drawn as", "Width", "Width: how sure", "Width: where from", "Depth", "Depth: how sure", "Depth: where from", "Height", "Height: how sure", "Height: where from", "Maker",
    "Maker: how sure", "Maker: where from", "Model", "Model: how sure", "Model: where from", "Weight", "Weight: how sure", "Weight: where from", "Priced as", "Unit cost", "Unit cost: how sure",
    "Unit cost: where from",
];

pub(super) const ITEM_COLUMNS: [(&str, &[&str]); 28] = [
    ("item", &["item", "name", "description"]),
    ("trade", &["trade", "division", "div"]),
    ("kind", &["kind", "category", "what it is"]),
    ("form", &["drawn as", "form", "shape"]),
    ("width", &["width", "w"]),
    ("width status", &["width: how sure"]),
    ("width source", &["width: where from"]),
    ("depth", &["depth", "d"]),
    ("depth status", &["depth: how sure"]),
    ("depth source", &["depth: where from"]),
    ("height", &["height", "h"]),
    ("height status", &["height: how sure"]),
    ("height source", &["height: where from"]),
    ("maker", &["maker", "manufacturer", "make"]),
    ("maker status", &["maker: how sure"]),
    ("maker source", &["maker: where from"]),
    ("model", &["model", "model number", "model no"]),
    ("model status", &["model: how sure"]),
    ("model source", &["model: where from"]),
    ("weight", &["weight"]),
    ("weight status", &["weight: how sure"]),
    ("weight source", &["weight: where from"]),
    ("priced as", &["priced as"]),
    ("unit cost", &["unit cost"]),
    ("unit cost status", &["unit cost: how sure"]),
    ("unit cost source", &["unit cost: where from"]),
    ("status", &["how sure", "status"]),
    ("source", &["where from", "source"]),
];

pub(super) fn item_kind_said(kind: Kind) -> &'static str {
    match kind {
        Kind::Equipment => "equipment",
        Kind::Fixture => "fixture",
        _ => "furnishing",
    }
}

/// The price an item row gives: the figure, what it is bought by (the each unless the cell says), how sure it is and where
/// it is from. None where the cell is blank (nothing is said of a price). A figure has to say how sure it is and where
/// it is from, in its own two columns or the row's one pair, and is not Unknown.
pub(super) fn unit_cost_of(cols: &Columns, row: &[String]) -> Result<Option<(String, String, Status, String)>, String> {
    let cell = cols.cell(row, "unit cost");
    if cell.is_empty() {
        return Ok(None);
    }
    let (x, unit) = price_cell(cell).map_err(|why| format!("its unit cost: {}", why.trim_start_matches("its price: ")))?;
    let own = (cols.cell(row, "unit cost status"), cols.cell(row, "unit cost source"));
    let (status, source) = if own.0.is_empty() && own.1.is_empty() { (cols.cell(row, "status"), cols.cell(row, "source")) } else { own };
    let status = status_read(status).ok_or_else(|| format!("how sure its unit cost is reads {status:?}, which is not Known, Assumed or Unknown"))?;
    if source.is_empty() {
        return Err("it does not say where its unit cost came from".into());
    }
    if status == Status::Unknown {
        return Err("its unit cost is given and is said to be Unknown".into());
    }
    Ok(Some((exact(x), unit.map(|u| unit_as_written(&u)).unwrap_or_else(|| "EA".to_string()), status, source.to_string())))
}

/// What an item with no weight says of it when the library is written out.
pub(super) const WEIGHT_NOT_SAID: &str = "not said";

/// What an item with no price in the price list says of it when the library is written out.
pub(super) const PRICE_NOT_SAID: &str = "not said";

/// The job's items as a table: the headings, then a row for each item in the order
/// they were made. Brought back in, it gives the same items, value for value.
pub fn items_out(job: &mut Job) -> Vec<Vec<String>> {
    use crate::units::Len;
    use crate::Item;
    let mut rows: Vec<Vec<String>> = vec![ITEM_HEADINGS.iter().map(|h| h.to_string()).collect()];
    for e in job.order.clone() {
        let (Some(node), Some(it)) = (job.world.get::<Node>(e), job.world.get::<Item>(e)) else { continue };
        let length = |v: &Val<Len>| [v.value.as_ref().map(|l| units::show_length(l, &v.unit)).unwrap_or_default(), status_said(v.status).to_string(), v.source.clone()];
        let text = |v: &Val<String>| [v.value.clone().unwrap_or_default(), status_said(v.status).to_string(), v.source.clone()];
        let mut row = vec![
            node.name.clone(),
            it.division.map(|d| format!("{} {}", d.number(), d.title())).unwrap_or_default(),
            item_kind_said(it.category).to_string(),
            it.form.clone().unwrap_or_default(),
        ];
        row.extend(length(&it.width));
        row.extend(length(&it.depth));
        row.extend(length(&it.height));
        row.extend(text(&it.maker));
        row.extend(text(&it.model));
        match &it.weight {
            Some(w) => row.extend([w.value.map(|x| format!("{} {}", crate::view::num(x), w.unit)).unwrap_or_default(), status_said(w.status).to_string(), w.source.clone()]),
            // No weight said: written as Unknown, as a maker not chosen is, and not as
            // three blanks (a blank "how sure" is none of the three: J25A-7).
            None => row.extend([String::new(), status_said(Status::Unknown).to_string(), WEIGHT_NOT_SAID.to_string()]),
        }
        row.push(it.priced_as.clone().unwrap_or_default());
        // Its price: the item's own entry in the price list (a Supply on the item), one home (call 196).
        match job.world.get::<Supply>(e).map(|s| s.price.clone()) {
            Some(p) if p.value.is_some() => row.extend([dollars(p.value.unwrap_or_default(), p.unit.trim_start_matches("$/")), status_said(p.status).to_string(), p.source.clone()]),
            Some(p) => row.extend([String::new(), status_said(Status::Unknown).to_string(), p.source.clone()]),
            None => row.extend([String::new(), status_said(Status::Unknown).to_string(), PRICE_NOT_SAID.to_string()]),
        }
        rows.push(row);
    }
    rows
}

/// Bring an item library in. A row whose item the job does not have adds it; a row
/// whose item it has, by name, changes it to what the row says, and says what changed;
/// a row that says what the job already holds is counted as the same. A value says how
/// sure it is and where it came from in its own two columns, or, where the table has
/// only one pair for the whole row ("How sure", "Where from"), in those. All of it or
/// none: a row that cannot be read is not used and says why, and the rest still is.
pub fn items_in(job: &mut Job, rows: &[Vec<String>], numbers: Option<&[usize]>) -> Result<Brought, String> {
    use crate::units::Len;
    use crate::Item;
    let cols = columns(rows, &ITEM_COLUMNS, &["item", "kind", "width", "depth", "height"]).ok_or_else(|| {
        "this is not an item library this program can read: within its first rows it has no row of headings with Item, Kind, Width, Depth and Height".to_string()
    })?;
    all_or_nothing(job, |job| {
        let mut out = Brought { rows: rows.len().saturating_sub(cols.row + 1), columns_not_used: cols.not_used.clone(), above: cols.row, ..Brought::default() };
        let mut have: std::collections::HashMap<String, String> = {
            let mut q = job.world.query::<(&Node, &Item)>();
            q.iter(&job.world).map(|(n, _)| (check::plain_name(&n.name), n.path.clone())).collect()
        };
        let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        // Each price the table gave: the row's number, the item, and the figure, its unit, how sure it is and where from.
        let mut prices: Vec<(usize, String, (String, String, Status, String))> = Vec::new();
        for (i, row) in rows.iter().enumerate().skip(cols.row + 1) {
            let number = row_number(numbers, i);
            if row.iter().all(|c| c.trim().is_empty()) {
                out.rows -= 1;
                continue;
            }
            let name = cols.cell(row, "item");
            let read = || -> Result<Item, String> {
                if name.is_empty() {
                    return Err("it has no name: what the item is called".into());
                }
                // How sure a value is and where it came from: its own columns, else
                // the row's one pair.
                let how = |what: &str, empty: bool| -> Result<(Status, String), String> {
                    let own = (cols.cell(row, &format!("{what} status")), cols.cell(row, &format!("{what} source")));
                    // Nothing given and nothing said of it: it is not known. (The row's
                    // one pair is for the values the row gives.)
                    if empty && own.0.is_empty() && own.1.is_empty() {
                        return Ok((Status::Unknown, "not said in the item library".to_string()));
                    }
                    let (status, source) = if own.0.is_empty() && own.1.is_empty() { (cols.cell(row, "status"), cols.cell(row, "source")) } else { own };
                    let status = status_read(status).ok_or_else(|| format!("how sure its {what} is reads {status:?}, which is not Known, Assumed or Unknown"))?;
                    if source.is_empty() {
                        return Err(format!("it does not say where its {what} came from"));
                    }
                    if empty && status != Status::Unknown {
                        return Err(format!("its {what} is empty and is said to be {}", status_said(status)));
                    }
                    if !empty && status == Status::Unknown {
                        return Err(format!("its {what} is given and is said to be Unknown"));
                    }
                    Ok((status, source.to_string()))
                };
                let length = |what: &str| -> Result<Val<Len>, String> {
                    let cell = cols.cell(row, what);
                    let (status, source) = how(what, cell.is_empty())?;
                    if cell.is_empty() {
                        return Ok(Val { value: None, unit: "in".into(), status, source });
                    }
                    let (len, unit) = units::parse_length(cell).map_err(|why| format!("its {what}: {why}"))?;
                    Ok(Val { value: Some(len), unit: unit.to_string(), status, source })
                };
                let text = |what: &str| -> Result<Val<String>, String> {
                    let cell = cols.cell(row, what);
                    let (status, source) = how(what, cell.is_empty())?;
                    Ok(Val { value: if cell.is_empty() { None } else { Some(cell.to_string()) }, unit: String::new(), status, source })
                };
                let category = match heading(cols.cell(row, "kind")).as_str() {
                    "equipment" => Kind::Equipment,
                    "fixture" => Kind::Fixture,
                    "furnishing" | "furniture" => Kind::Furnishing,
                    other => return Err(format!("its kind reads {other:?}: an item is equipment, a fixture or a furnishing")),
                };
                let division = match cols.cell(row, "trade") {
                    "" => None,
                    said => Some(crate::components::Division::parse(said.split_once(' ').filter(|(n, _)| n.chars().all(|c| c.is_ascii_digit())).map(|(n, _)| n).unwrap_or(said)).map_err(|why| format!("its trade: {why}"))?),
                };
                let form = match cols.cell(row, "form") {
                    "" => None,
                    said => {
                        crate::building::shape::check_drawn_as(&job.shapes, said).map_err(|why| format!("drawn as: {why}"))?;
                        Some(said.to_lowercase())
                    }
                };
                let weight = match cols.cell(row, "weight") {
                    "" if cols.cell(row, "weight status").is_empty() && cols.cell(row, "weight source").is_empty() => None,
                    // As this program writes an item with no weight.
                    "" if status_read(cols.cell(row, "weight status")) == Some(Status::Unknown) && cols.cell(row, "weight source") == WEIGHT_NOT_SAID => None,
                    cell => {
                        let (status, source) = how("weight", cell.is_empty())?;
                        let value = if cell.is_empty() {
                            None
                        } else {
                            let figure = cell.to_lowercase();
                            let figure = figure.trim_end_matches("lbs").trim_end_matches("lb").trim();
                            Some(units::parse_number(figure).map_err(|why| format!("its weight: {why}"))?)
                        };
                        Some(Val { value, unit: "lb".into(), status, source })
                    }
                };
                Ok(Item { category, maker: text("maker")?, model: text("model")?, width: length("width")?, depth: length("depth")?, height: length("height")?, weight, sheet: String::new(), division, form, priced_as: None })
            };
            let plain = check::plain_name(name);
            let mut item = match read() {
                Ok(item) => item,
                Err(why) => {
                    out.not_used.push((number, if name.is_empty() { why } else { format!("{name}: {why}") }));
                    continue;
                }
            };
            // Its price: one home, the price list. A cell of figures sets it there (below, once the rows are read).
            let price = match unit_cost_of(&cols, row) {
                Ok(p) => p,
                Err(why) => {
                    out.not_used.push((number, format!("{name}: {why}")));
                    continue;
                }
            };
            // Drawn as a shape: held to its box at the sizes it has (call 206 point 2).
            if let Err(why) = crate::editing::shapes::item_fits(&job.shapes, name, &item) {
                out.not_used.push((number, why));
                continue;
            }
            // What it is priced as: said by a column of its own. A table without the column leaves it as the job has it.
            let said_priced_as: Option<Option<String>> = if cols.has("priced as") {
                match cols.cell(row, "priced as") {
                    "" => Some(None),
                    typed => match crate::editing::assembly_named(job, typed, item.category) {
                        Ok((assembly, _)) => Some(Some(assembly)),
                        Err(why) => {
                            out.not_used.push((number, format!("{name}: priced as: {why}")));
                            continue;
                        }
                    },
                }
            } else {
                None
            };
            if let Some(first) = seen.get(&plain) {
                out.not_used.push((number, format!("{name}: row {first} of this library is the same item, and was used")));
                continue;
            }
            seen.insert(plain.clone(), number);
            match have.get(&plain).cloned() {
                Some(path) => {
                    let e = job.find(&path).ok_or_else(|| format!("no element {path}"))?;
                    let was = job.world.get::<Item>(e).cloned().ok_or_else(|| format!("{path} is not an item"))?;
                    // The sheet an item was made from is the job's to remember: a
                    // library has no column for it and does not take it away.
                    // Priced as: its own, unless the table says. It follows below, as sizes do.
                    let item = Item { sheet: was.sheet.clone(), priced_as: was.priced_as.clone(), ..item };
                    let priced_as = said_priced_as.clone().unwrap_or_else(|| was.priced_as.clone());
                    if was == item && priced_as == was.priced_as {
                        if let Some(p) = price {
                            prices.push((number, name.to_string(), p));
                        }
                        out.same += 1;
                        continue;
                    }
                    if let Err(why) = crate::building::check_item(&path, Kind::Type, &item) {
                        out.not_used.push((number, format!("{name}: {why}")));
                        continue;
                    }
                    let mut changed: Vec<&str> = Vec::new();
                    for (what, differs) in [("its trade", was.division != item.division), ("its kind", was.category != item.category), ("what it is drawn as", was.form != item.form), ("its width", was.width != item.width), ("its depth", was.depth != item.depth), ("its height", was.height != item.height), ("its maker", was.maker != item.maker), ("its model", was.model != item.model), ("its weight", was.weight != item.weight), ("what it is priced as", priced_as != was.priced_as)] {
                        if differs {
                            changed.push(what);
                        }
                    }
                    job.put(&path, item.clone())?;
                    // Its trade changed: the lines on what is set down from it that took its trade follow (call 206 point 6).
                    let trade_said = if was.division != item.division { crate::editing::trade_follow(job, &path, was.division, item.division)?.said(item.division) } else { String::new() };
                    // What is already set down from it is as big as it says: each
                    // follows, and how many did is said (J25A-6: a width changed in the
                    // library left the range standing in the room at its old size, with
                    // the screen saying "changed: its width").
                    let followed = crate::editing::placed_follow(job, &path, name, &item)?;
                    let mut also = match followed.len() {
                        0 => String::new(),
                        1 => "; the one set down is now that size".to_string(),
                        n => format!("; the {n} set down are now that size"),
                    };
                    if !trade_said.is_empty() {
                        also.push_str(&format!("; {trade_said}"));
                    }
                    // What it is priced as: each set down follows (call 196).
                    if priced_as != was.priced_as {
                        let said = crate::editing::item_priced_as(job, &path, priced_as.as_deref())?;
                        let n = said.followed.len();
                        if said.set_down > 0 {
                            also.push_str(&format!("; {} set down {} priced as {}", n, if n == 1 { "is now" } else { "are now" }, priced_as.as_deref().map(|a| format!("{a:?}")).unwrap_or_else(|| "nothing".to_string())));
                        }
                        for line in said.said().into_iter().skip(1) {
                            out.notes.push(format!("{name}: {}", line.trim()));
                        }
                    }
                    if let Some(p) = price {
                        prices.push((number, name.to_string(), p));
                    }
                    out.done.push(format!("{name}: changed: {}{also}", changed.join(", ")));
                }
                None => {
                    item.priced_as = said_priced_as.clone().flatten();
                    match crate::editing::item(job, name, item) {
                        Ok(path) => {
                            have.insert(plain, path);
                            out.done.push(format!("{name}: added"));
                            if let Some(p) = price {
                                prices.push((number, name.to_string(), p));
                            }
                        }
                        Err(why) => out.not_used.push((number, format!("{name}: {why}"))),
                    }
                }
            }
        }
        // The prices the table gave, set in the price list (one home, two doors).
        if !prices.is_empty() {
            let mut list: Vec<Vec<String>> = vec![["Material", "Unit", "Price", "How sure", "Where from"].iter().map(|s| s.to_string()).collect()];
            let mut numbers: Vec<usize> = vec![0];
            for (number, name, (figure, unit, status, source)) in &prices {
                list.push(vec![crate::editing::item_material_name(name), unit.clone(), figure.clone(), status_said(*status).to_string(), source.clone()]);
                numbers.push(*number);
            }
            let set = prices_in(job, &list, Some(&numbers), None, true, None)?;
            let n = list.len() - 1 - set.same - set.not_used.len();
            if n > 0 {
                out.done.push(format!("{} in the price list", if n == 1 { "1 price was set".to_string() } else { format!("{n} prices were set") }));
            }
            for (number, why) in set.not_used {
                out.not_used.push((number, format!("its price: {why}")));
            }
            // What the price list says of a price that prices nothing yet (call 206 point 7).
            out.notes.extend(set.notes);
        }
        Ok(out)
    })
}

/// The library the program ships with, brought in again: it adds the items the job has
/// not got and leaves every item the job has as it is, whatever the shipped library
/// says of it. A person's own changes to an item (a width measured, a trade, what it is
/// drawn as) are not written over by pressing Starter items a second time (J25A-18:
/// they were, and the screen called it "Brought in 3 starter items"). Returns what was
/// done, and the names of the items left as the job has them. A library a person
/// brings in themselves is `items_in`: there a changed row is meant.
pub fn items_in_missing(job: &mut Job, rows: &[Vec<String>], numbers: Option<&[usize]>) -> Result<(Brought, Vec<String>), String> {
    let cols = columns(rows, &ITEM_COLUMNS, &["item", "kind", "width", "depth", "height"]).ok_or_else(|| "this is not an item library this program can read".to_string())?;
    let have: std::collections::HashSet<String> = {
        let mut q = job.world.query::<(&Node, &crate::Item)>();
        q.iter(&job.world).map(|(n, _)| check::plain_name(&n.name)).collect()
    };
    let (mut wanted, mut their_numbers, mut kept) = (Vec::new(), Vec::new(), Vec::new());
    for (i, row) in rows.iter().enumerate() {
        let name = cols.cell(row, "item");
        if i > cols.row && !name.is_empty() && have.contains(&check::plain_name(name)) {
            kept.push(name.to_string());
            continue;
        }
        wanted.push(row.clone());
        their_numbers.push(row_number(numbers, i));
    }
    items_in(job, &wanted, Some(&their_numbers)).map(|brought| (brought, kept))
}
