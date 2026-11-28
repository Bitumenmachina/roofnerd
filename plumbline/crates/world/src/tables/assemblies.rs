use super::*;

pub const ASSEMBLY_HEADINGS: [&str; 27] = [
    "Assembly",
    "Goes on",
    "Cost code",
    "Code title",
    "Code scheme",
    "Code: how sure",
    "Code: where from",
    "Line",
    "Cost type",
    "Quantity",
    "Stretch-out",
    "Stretch-out: how sure",
    "Stretch-out: where from",
    "Waste %",
    "Waste: how sure",
    "Waste: where from",
    "Ordered by",
    "Ordered at",
    "Whole ones only",
    "Ordered at: how sure",
    "Ordered at: where from",
    "Unit cost",
    "Unit cost: how sure",
    "Unit cost: where from",
    "Made of the element's material",
    "Off",
    "Division",
];

/// The headings as this program wrote them before a line carried its division, and
/// when it called a line's cost type its class: a library written then still reads.
pub(super) fn assembly_headings_before() -> Vec<&'static str> {
    ASSEMBLY_HEADINGS[..26].iter().map(|h| if *h == "Cost type" { "Class" } else { *h }).collect()
}

/// A length, or a plain number, as a cell: `30 in`, `25' 6 1/2"`, `2`. What is not known
/// is `unknown` and its unit. Written so that reading it gives the same value: the way a
/// drawing writes it where that is exact, and in full otherwise.
pub(super) fn length_out(v: &Val<units::Len>) -> String {
    let Some(len) = &v.value else { return format!("unknown {}", v.unit).trim().to_string() };
    let plain = match len {
        units::Len::One(x) => format!("{} {}", exact(*x), v.unit).trim().to_string(),
        units::Len::FtIn([feet, inches]) => format!("{}' {}\"", exact(*feet), exact(*inches)),
    };
    if v.unit.is_empty() {
        return plain;
    }
    let as_drawn = units::show_length(len, &v.unit);
    match units::parse_length(&as_drawn) {
        Ok((back, unit)) if back == *len && unit == v.unit => as_drawn,
        _ => plain,
    }
}

pub(super) fn length_in(cell: &str, status: Status, source: &str) -> Result<Val<units::Len>, String> {
    let t = cell.trim();
    if let Some(unit) = t.strip_prefix("unknown") {
        return Ok(Val { value: None, unit: unit.trim().to_string(), status, source: source.to_string() });
    }
    if let Ok(x) = t.parse::<f64>() {
        return Ok(Val { value: Some(units::Len::One(x)), unit: String::new(), status, source: source.to_string() });
    }
    // 60 SF, 2 CY, 24 LF, 3 SQ, 4 EA: a measure typed on a line (call 159).
    if let Some(read) = units::read_measure(t, false) {
        let (x, unit) = read?;
        return Ok(Val { value: Some(units::Len::One(x)), unit: unit.to_string(), status, source: source.to_string() });
    }
    let (len, unit) = units::parse_length(t)?;
    Ok(Val { value: Some(len), unit: unit.to_string(), status, source: source.to_string() })
}

/// A figure and what it is in, as a cell: `30 SF/sheet`, `unknown lb/SF`.
pub(super) fn figure_out(v: &Val<f64>) -> String {
    format!("{} {}", v.value.map(exact).unwrap_or_else(|| "unknown".to_string()), v.unit).trim().to_string()
}

pub(super) fn figure_in(cell: &str, status: Status, source: &str) -> Result<Val<f64>, String> {
    let t = cell.trim();
    let (figure, unit) = t.split_once(' ').unwrap_or((t, ""));
    let value = if figure == "unknown" { None } else { Some(figure.parse::<f64>().map_err(|_| format!("{cell:?} is not a figure and what it is in"))?) };
    Ok(Val { value, unit: unit.trim().to_string(), status, source: source.to_string() })
}

/// Dollars for a unit, as a cell: `$4.25/SF`; not known, `$?/SF`.
pub(super) fn rate_out(v: &Val<f64>) -> String {
    format!("${}/{}", v.value.map(exact).unwrap_or_else(|| "?".to_string()), v.unit.trim_start_matches("$/"))
}

pub(super) fn rate_in(cell: &str, status: Status, source: &str) -> Result<Val<f64>, String> {
    // "$4.25 per SF" is "$4.25/SF".
    let written = cell.trim().replace(" per ", "/");
    let t = written.trim_start_matches('$');
    // A library's line goes on things of more than one kind and its unit is not known
    // until it does, so its price says what it is for (J17C-19).
    let (figure, unit) = t.split_once('/').ok_or_else(|| format!("{cell:?} does not say what it is for: a price in a library is dollars for a unit, like $4.25/SF or $4.25/EA ($?/SF where the price is not known yet)"))?;
    if figure.trim().is_empty() {
        return Err(format!("{cell:?} has no figure: write the dollars, like $4.25/{u}, or $?/{u} where the price is not known yet", u = unit.trim()));
    }
    if unit.trim().is_empty() {
        return Err(format!("{cell:?} does not say what it is for: write the unit after the slash, like $4.25/SF"));
    }
    // Read as every other price is read (J17A-17): a comma is a thousands mark or the
    // figure is refused, never dropped ("$1,23" is not $123), and a price is not below zero.
    let value = if figure.trim() == "?" { None } else { Some(crate::units::parse_price(figure.trim(), unit.trim()).map_err(|why| format!("{why} (dollars for a unit, like $4.25/SF)"))?) };
    Ok(Val { value, unit: format!("$/{}", unit.trim()), status, source: source.to_string() })
}

pub(super) fn yes(b: bool) -> String {
    if b { "yes" } else { "" }.to_string()
}

pub(super) fn is_yes(cell: &str) -> bool {
    matches!(cell.trim().to_lowercase().as_str(), "yes" | "y" | "true")
}

/// One cost line as the cells after an assembly's (or an element's) own: from `Line` on.
pub(super) fn line_out(l: &crate::components::CostLine) -> Vec<String> {
    let said = |s: Status| status_said(s).to_string();
    let mut row = vec![l.description.clone(), l.class.map(|c| c.title().to_string()).unwrap_or_default(), l.quantity.clone()];
    match &l.stretch_out {
        Some(v) => row.extend([length_out(v), said(v.status), v.source.clone()]),
        None => row.extend([String::new(), String::new(), String::new()]),
    }
    match &l.waste {
        Some(v) => row.extend([v.value.map(exact).unwrap_or_else(|| "unknown".to_string()), said(v.status), v.source.clone()]),
        None => row.extend([String::new(), String::new(), String::new()]),
    }
    match &l.order {
        Some(o) => row.extend([o.unit.clone(), figure_out(&o.factor), if o.round_up { "yes" } else { "no" }.to_string(), said(o.factor.status), o.factor.source.clone()]),
        None => row.extend(std::iter::repeat_n(String::new(), 5)),
    }
    if crate::components::CostLine::is_no_price_of_its_own(&l.unit_cost) {
        row.extend([String::new(), String::new(), String::new()]);
    } else {
        row.extend([rate_out(&l.unit_cost), said(l.unit_cost.status), l.unit_cost.source.clone()]);
    }
    row.push(yes(l.of_material));
    row.push(yes(l.off));
    row.push(l.division.map(|d| d.number().to_string()).unwrap_or_default());
    for (name, v) in &l.inputs {
        row.extend([format!("{name} = {}", length_out(v)), said(v.status), v.source.clone()]);
    }
    row
}

/// The cells of `line_out`, read back. `cells` starts at `Line`. `with_division` is
/// false for a library written before a line carried its division.
pub(super) fn line_in(cells: &[String], with_division: bool) -> Result<crate::components::CostLine, String> {
    use crate::components::{Class, CostLine, Division};
    let cell = |i: usize| cells.get(i).map(|s| s.trim()).unwrap_or("");
    let how = |i: usize, what: &str| -> Result<(Status, String), String> {
        let status = status_read(cell(i)).ok_or_else(|| format!("its {what} does not say how sure it is (Known, Assumed or Unknown)"))?;
        if cell(i + 1).is_empty() {
            return Err(format!("its {what} does not say where it came from"));
        }
        Ok((status, cell(i + 1).to_string()))
    };
    let description = cell(0).to_string();
    let class = match cell(1) {
        "" => None,
        text => Some(Class::ALL.into_iter().find(|c| c.title().eq_ignore_ascii_case(text)).map(Ok).unwrap_or_else(|| Class::parse(text))?),
    };
    let stretch_out = if cell(3).is_empty() && cell(4).is_empty() {
        None
    } else {
        let (status, source) = how(4, "stretch-out")?;
        Some(length_in(cell(3), status, &source)?)
    };
    let waste = if cell(6).is_empty() && cell(7).is_empty() {
        None
    } else {
        let (status, source) = how(7, "waste")?;
        let value = if cell(6) == "unknown" { None } else { Some(cell(6).parse::<f64>().map_err(|_| format!("its waste, {:?}, is not a percentage", cell(6)))?) };
        Some(Val { value, unit: "%".into(), status, source })
    };
    let order = if cell(9).is_empty() && cell(10).is_empty() {
        None
    } else {
        let (status, source) = how(12, "order conversion")?;
        Some(Order { unit: cell(9).to_string(), factor: figure_in(cell(10), status, &source)?, round_up: is_yes(cell(11)) })
    };
    let unit_cost = if cell(14).is_empty() && cell(15).is_empty() {
        CostLine::no_price_of_its_own()
    } else {
        let (status, source) = how(15, "unit cost")?;
        rate_in(cell(14), status, &source)?
    };
    let division = match cell(19) {
        _ if !with_division => None,
        "" => None,
        text => Some(Division::parse(text)?),
    };
    let mut inputs = BTreeMap::new();
    let mut i = if with_division { 20 } else { 19 };
    while i < cells.len() {
        if !cell(i).is_empty() {
            let (name, value) = cell(i).split_once('=').ok_or_else(|| format!("{:?} is not a figure by name, like spacing = 12 in", cell(i)))?;
            let (status, source) = how(i + 1, name.trim())?;
            inputs.insert(name.trim().to_string(), length_in(value, status, &source)?);
        }
        i += 3;
    }
    Ok(CostLine { description, quantity: cell(2).to_string(), stretch_out, inputs, waste, order, unit_cost, class, division, off: is_yes(cell(18)), of_material: is_yes(cell(17)) })
}

pub(super) fn kind_said(k: Kind) -> String {
    serde_json::to_value(k).ok().and_then(|v| v.as_str().map(str::to_string)).unwrap_or_default()
}

pub(super) fn kind_read(text: &str) -> Option<Kind> {
    serde_json::from_value(serde_json::Value::String(text.trim().to_string())).ok()
}

/// The job's assemblies (its conditions, each with the lines it puts on what it goes on)
/// as rows.
pub fn assemblies_out(job: &mut Job) -> Vec<Vec<String>> {
    use crate::components::Recipe;
    let mut body: Vec<Vec<String>> = Vec::new();
    for e in job.order.clone() {
        let (Some(node), Some(recipe)) = (job.world.get::<Node>(e), job.world.get::<Recipe>(e)) else { continue };
        let code = &recipe.cost.code;
        let (scheme, number, title) = code.value.as_ref().map(|c| (c.scheme.clone(), c.code.clone(), c.title.clone())).unwrap_or_default();
        let own = vec![node.name.clone(), recipe.applies_to.iter().map(|k| kind_said(*k)).collect::<Vec<_>>().join(", "), number, title, scheme, status_said(code.status).to_string(), code.source.clone()];
        if recipe.cost.lines.is_empty() {
            body.push(own.clone());
        }
        for line in &recipe.cost.lines {
            let mut row = own.clone();
            row.extend(line_out(line));
            body.push(row);
        }
    }
    let widest = body.iter().map(Vec::len).max().unwrap_or(0).max(ASSEMBLY_HEADINGS.len());
    let mut headings: Vec<String> = ASSEMBLY_HEADINGS.iter().map(|h| h.to_string()).collect();
    for k in 1..=(widest - ASSEMBLY_HEADINGS.len()) / 3 {
        headings.extend([format!("Figure {k}"), format!("Figure {k}: how sure"), format!("Figure {k}: where from")]);
    }
    let mut rows = vec![headings];
    rows.extend(body);
    rows
}

/// Bring an assembly library in: a table this program wrote. An assembly the job already
/// has by that name is left as it is: counted as the same if it is, listed if it differs.
pub fn assemblies_in(job: &mut Job, rows: &[Vec<String>], numbers: Option<&[usize]>) -> Result<Brought, String> {
    use crate::components::{Cost, CostCode, Recipe};
    let head = rows.first().ok_or("the table is empty")?;
    let wanted: Vec<String> = ASSEMBLY_HEADINGS.iter().map(|h| heading(h)).collect();
    let before: Vec<String> = assembly_headings_before().iter().map(|h| heading(h)).collect();
    let got: Vec<String> = head.iter().map(|h| heading(h)).collect();
    // A library written before a line carried its division has one heading fewer.
    let with_division = got.len() >= wanted.len() && got[..wanted.len()] == wanted[..];
    if !with_division && !(got.len() >= before.len() && got[..before.len()] == before[..]) {
        return Err(format!("this is not an assembly library this program wrote: its headings have to be, in order: {}", ASSEMBLY_HEADINGS.join(", ")));
    }
    all_or_nothing(job, |job| {
        let mut out = Brought { rows: rows.len() - 1, ..Brought::default() };
        // Rows gathered by assembly, in the order each is first named.
        let mut gathered: Vec<(String, Vec<(usize, &Vec<String>)>)> = Vec::new();
        for (i, row) in rows.iter().enumerate().skip(1) {
            let name = row.first().map(|s| s.trim()).unwrap_or("");
            let number = row_number(numbers, i);
            if name.is_empty() {
                out.not_used.push((number, "it names no assembly".into()));
                continue;
            }
            match gathered.iter_mut().find(|(known, _)| known == name) {
                Some((_, its)) => its.push((number, row)),
                None => gathered.push((name.to_string(), vec![(number, row)])),
            }
        }
        let root = job.world.get::<Node>(job.root()).map(|n| n.path.clone()).ok_or("the job has no root")?;
        for (name, its) in gathered {
            let first = its[0].1;
            let cell = |i: usize| first.get(i).map(|s| s.trim()).unwrap_or("");
            // (A fault says which row it is in, when it is in one: J17A-18.)
            let made = (|| -> Result<Recipe, (Option<usize>, String)> {
                // Every row of an assembly says the same of the assembly itself: what it
                // goes on and its cost code. Two that differ are two assemblies under one
                // name, and neither is taken for the other.
                if let Some((n, _)) = its.iter().find(|(_, row)| (1..7).any(|c| row.get(c).map(|s| s.trim()).unwrap_or("") != cell(c))) {
                    return Err((None, format!("its rows do not agree on what it goes on or on its cost code (row {} and row {n}): two assemblies cannot have one name", its[0].0)));
                }
                let applies_to: Vec<Kind> = cell(1).split(',').filter(|s| !s.trim().is_empty()).map(|s| kind_read(s).ok_or_else(|| format!("it goes on {:?}, which is not a kind of element", s.trim()))).collect::<Result<_, String>>().map_err(|why| (None, why))?;
                let status = status_read(cell(5)).ok_or((None, "its cost code does not say how sure it is".to_string()))?;
                let value = if cell(2).is_empty() && cell(3).is_empty() && cell(4).is_empty() { None } else { Some(CostCode { scheme: cell(4).to_string(), code: cell(2).to_string(), title: cell(3).to_string() }) };
                let code = Val { value, unit: String::new(), status, source: cell(6).to_string() };
                let mut lines = Vec::new();
                for (n, row) in &its {
                    if row.len() > 7 && row[7..].iter().any(|c| !c.trim().is_empty()) {
                        lines.push(line_in(&row[7..], with_division).map_err(|why| (Some(*n), format!("its line {:?}: {why}", row.get(7).map(|s| s.as_str()).unwrap_or(""))))?);
                    }
                }
                Ok(Recipe { applies_to, cost: Cost { code, lines } })
            })();
            let numbers: Vec<usize> = its.iter().map(|(n, _)| *n).collect();
            let recipe = match made {
                Ok(r) => r,
                // The row at fault is named for what is wrong with it; the rest of the
                // assembly's rows are said to be left out with it, not blamed for it.
                Err((at, why)) => {
                    out.not_used.extend(numbers.iter().map(|n| match at {
                        Some(bad) if bad != *n => (*n, format!("{name}: nothing wrong with this row; it is left out with the rest of its assembly, which is refused whole because of row {bad}")),
                        _ => (*n, format!("{name}: {why}")),
                    }));
                    continue;
                }
            };
            // An assembly the job has by this name.
            let known: Option<(String, Option<Recipe>)> = {
                let mut q = job.world.query::<(&Node, &Kind, Option<&Recipe>)>();
                q.iter(&job.world).find(|(n, k, _)| **k == Kind::Condition && check::same_name(&n.name, &name)).map(|(n, _, r)| (n.path.clone(), r.cloned()))
            };
            match known {
                Some((_, Some(has))) if has == recipe => out.same += its.len(),
                Some((path, _)) => out.not_used.extend(numbers.iter().map(|n| (*n, format!("the job already has an assembly called {name:?} ({path}) and it is not the same: it was left as it is")))),
                None => {
                    if let Err(why) = check::recipe(&format!("the assembly {name:?}"), Kind::Condition, &recipe) {
                        out.not_used.extend(numbers.iter().map(|n| (*n, why.clone())));
                        continue;
                    }
                    let count = recipe.cost.lines.len();
                    let path = job.add(&root, Kind::Condition, &name)?;
                    job.put(&path, recipe)?;
                    out.done.push(format!("{name}: added, with {count} line{}", if count == 1 { "" } else { "s" }));
                }
            }
        }
        Ok(out)
    })
}
