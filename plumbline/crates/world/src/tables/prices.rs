use super::*;

/// The number a row has in its file: what the reader of the file gave, or its place in
/// the rows counted from 1.
pub(super) fn row_number(numbers: Option<&[usize]>, i: usize) -> usize {
    numbers.and_then(|n| n.get(i).copied()).unwrap_or(i + 1)
}

/// A figure as a list writes it: `4.25`, `1,204.50`, `$450`. A comma is read only where
/// it parts thousands, by the rule a typed price is read by. Anywhere else it may be the
/// decimal mark of a list written in another country, where `9,05` is nine and a bit and
/// `1.204,50` is twelve hundred: the figure is refused, not guessed at.
pub(super) fn list_figure(text: &str) -> Result<f64, String> {
    let t = text.trim().trim_start_matches('$').trim();
    // The same rule a price typed at the keyboard is read by (`units::parse_price`).
    if t.contains(',') && !units::commas_part_thousands(t) {
        return Err(format!("\"{}\" has a comma that does not part thousands; if it is the decimal mark, write a point (9.05)", text.trim()));
    }
    // A price is dollars and cents, as written: 1e3 is a way of writing a figure that nobody writes a price in (call 213 point 10).
    let x: f64 = if t.contains(['e', 'E']) { None } else { t.replace(',', "").parse().ok() }.ok_or_else(|| format!("\"{}\" is not a figure", text.trim()))?;
    if !x.is_finite() {
        return Err(format!("\"{}\" is not a figure", text.trim()));
    }
    Ok(x)
}

/// The job's price list as rows: its headings, then for each material its own price and
/// how it is ordered, then each supplier's price for it.
pub fn prices_out(job: &mut Job) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = vec![PRICE_HEADINGS.iter().map(|h| h.to_string()).collect()];
    for e in job.order.clone() {
        let (Some(node), Some(supply)) = (job.world.get::<Node>(e), job.world.get::<Supply>(e)) else { continue };
        let by = supply.price.unit.trim_start_matches("$/").to_string();
        let priced = |v: &Val<f64>, supplier: &str| -> Vec<String> { vec![node.name.clone(), by.clone(), v.value.map(exact).unwrap_or_default(), supplier.to_string(), status_said(v.status).to_string(), v.source.clone()] };
        let mut own = priced(&supply.price, "");
        if let Some(o) = &supply.order {
            own.push(format!("{} {}", o.factor.value.map(exact).unwrap_or_else(|| "unknown".to_string()), o.factor.unit));
            own.push(if o.round_up { "yes" } else { "no" }.to_string());
            own.push(status_said(o.factor.status).to_string());
            own.push(o.factor.source.clone());
        }
        rows.push(own);
        for (supplier, quote) in &supply.quotes {
            rows.push(priced(quote, supplier));
        }
    }
    rows
}

/// A price as a list writes it: `4.25`, `$4.25`, `4.25/lb`, `$1,204.50 / sheet`. The
/// figure, and the unit after the stroke if there is one.
pub(super) fn price_cell(text: &str) -> Result<(f64, Option<String>), String> {
    let t = text.trim();
    let (figure, unit) = match t.split_once('/') {
        Some((f, u)) => (f.trim(), Some(u.trim().to_string()).filter(|u| !u.is_empty())),
        None => (t, None),
    };
    let x = list_figure(figure).map_err(|why| if why.contains("comma") { format!("its price: {why}") } else { format!("its price, \"{}\", is not a figure; write a price as dollars, like $4.25/SF or $180/EA", text.trim()) })?;
    if x < 0.0 {
        return Err(format!("its price, \"{}\", is less than nothing", text.trim()));
    }
    Ok((x, unit))
}

/// A unit as a list writes it, less what is only the list's way of writing: a full stop
/// after it (`lb.`), and the plural of the pound (`lbs`). Nothing else is changed.
pub(super) fn unit_as_written(unit: &str) -> String {
    let u = unit.trim().trim_end_matches('.');
    if u.eq_ignore_ascii_case("lbs") {
        u[..2].to_string()
    } else {
        u.to_string()
    }
}

/// A price as it is held: to the cent, or to the three or four places a supplier's list
/// gives it to when it has them ($12.345/sheet is not said as $12.35, which would not
/// multiply to the amount: J17A-11).
pub(super) fn dollars(x: f64, by: &str) -> String {
    let places = (2..=4).find(|p| ((x * 10f64.powi(*p)).round() / 10f64.powi(*p) - x).abs() < 1e-9).unwrap_or(4);
    format!("${}/{by}", crate::view::figure(x, places as usize))
}

/// Bring a price list in. `supplier` says whose prices these are when the table has no
/// column for it (none: the job's own). `how` is whoever brings the list in saying how
/// sure its prices are and where they came from: it stands over the list's own columns
/// for that, and the result says so. With no `how`, the list's own columns are read; a
/// list that has neither is refused. With `add`, a row for a material the job does not
/// have adds the material. `numbers` gives each row's number in its file.
pub fn prices_in(job: &mut Job, rows: &[Vec<String>], numbers: Option<&[usize]>, supplier: Option<&str>, add: bool, how: Option<Entered>) -> Result<Brought, String> {
    let cols = columns(rows, &PRICE_COLUMNS, &["material", "price"]).ok_or_else(|| {
        "this is not a price list this program can read: within its first rows it has no row of headings with a column for what is priced (Material, Item or Description) and one for the price (Price, Unit price or Cost)".to_string()
    })?;
    if !cols.has("unit") {
        return Err("the price list has no column for what each thing is bought by (Unit, UOM or Bought by): a price is for a unit, and this cannot tell a pound from a sheet".into());
    }
    // What the person bringing the list in says of it stands over what the list says of
    // itself: a supplier's flyer that calls its own prices Known does not make them so.
    let overruled = how.is_some() && cols.has("status") && cols.has("source");
    let own_words = how.is_none() && cols.has("status") && cols.has("source");
    if !own_words && how.is_none() {
        return Err("say how sure these prices are and where they came from (--status, --source): the list has no columns for it".into());
    }
    if let Some(s) = supplier {
        if s.trim().is_empty() {
            return Err("a supplier has a name".into());
        }
    }
    all_or_nothing(job, |job| {
        let mut out = Brought { rows: rows.len().saturating_sub(cols.row + 1), columns_not_used: cols.not_used.clone(), above: cols.row, ..Brought::default() };
        // The materials the job has, and the suppliers, as the job names them.
        // Each by its name as two names are compared (`check::plain_name`), worked once:
        // a row finds its material in one look, not by going through every material
        // (M21-9: a list of 4,000 rows took a minute and a half).
        let mut by_plain: std::collections::HashMap<String, String> = {
            let mut q = job.world.query::<(&Node, &Kind)>();
            let mut found = std::collections::HashMap::new();
            for (n, _) in q.iter(&job.world).filter(|(_, k)| **k == Kind::Material) {
                found.entry(check::plain_name(&n.name)).or_insert_with(|| n.path.clone());
            }
            // An item is its own entry in the price list (call 196).
            let mut items = job.world.query::<(&Node, &crate::Item)>();
            for (n, _) in items.iter(&job.world) {
                found.entry(check::plain_name(&n.name)).or_insert_with(|| n.path.clone());
            }
            found
        };
        let mut suppliers: Vec<String> = {
            let mut q = job.world.query::<&Supply>();
            let mut all: Vec<String> = q.iter(&job.world).flat_map(|s| s.quotes.keys().cloned()).collect();
            all.sort();
            all.dedup();
            all
        };
        if overruled {
            let how = how.as_ref().expect("overruled only with a how");
            out.notes.push(format!("the list's own columns for how sure each price is and where it came from were not read: every price is {}, {}, as you said", status_said(how.status), how.source));
        }
        // What each row of this list has priced so far: a thing priced twice at two
        // prices is not taken a second time.
        let mut priced: Vec<(String, Option<String>, usize, Option<f64>)> = Vec::new();
        for (i, row) in rows.iter().enumerate().skip(cols.row + 1) {
            let number = row_number(numbers, i);
            let mut skip = |why: String| out.not_used.push((number, why));
            let name = cols.cell(row, "material");
            if name.is_empty() {
                skip("it names no material".into());
                continue;
            }
            // How sure, and where from: the row's own words, or whoever brought the list.
            let (status, source) = if own_words {
                match (status_read(cols.cell(row, "status")), cols.cell(row, "source")) {
                    (Some(status), source) if !source.is_empty() => (status, source.to_string()),
                    (None, _) => {
                        skip(format!("{name}: how sure it is reads {:?}, which is not Known, Assumed or Unknown", cols.cell(row, "status")));
                        continue;
                    }
                    (Some(_), _) => {
                        skip(format!("{name}: it does not say where its price came from"));
                        continue;
                    }
                }
            } else {
                let how = how.as_ref().expect("checked above");
                (how.status, how.source.to_string())
            };
            let typed = cols.cell(row, "price");
            let (price, unit_in_price) = if typed.is_empty() {
                if status != Status::Unknown {
                    skip(format!("{name}: it has no price"));
                    continue;
                }
                (None, None)
            } else {
                match price_cell(typed) {
                    Ok((x, unit)) => (Some(x), unit),
                    Err(why) => {
                        skip(format!("{name}: {why}"));
                        continue;
                    }
                }
            };
            if price.is_some() && status == Status::Unknown {
                skip(format!("{name}: it has a price and says the price is Unknown"));
                continue;
            }
            let unit_in_price = unit_in_price.map(|u| unit_as_written(&u));
            let unit_cell = unit_as_written(cols.cell(row, "unit"));
            let unit = match (unit_cell.as_str(), unit_in_price) {
                ("", Some(u)) => u,
                ("", None) => {
                    skip(format!("{name}: it does not say what it is bought by"));
                    continue;
                }
                (u, Some(other)) if !u.eq_ignore_ascii_case(&other) => {
                    skip(format!("{name}: it is bought by the {u} and priced by the {other}"));
                    continue;
                }
                (u, _) => u.to_string(),
            };
            // Whose price: the row's supplier, or the one named for the whole list.
            let whose = match (cols.cell(row, "supplier"), supplier) {
                ("", given) => given.map(str::to_string),
                (named, _) => Some(named.to_string()),
            };
            let whose = whose.map(|w| suppliers.iter().find(|known| check::same_name(known, &w)).cloned().unwrap_or(w));
            // The material, by its name however it is typed.
            let plain = check::plain_name(name);
            let path = match by_plain.get(&plain) {
                Some(path) => path.clone(),
                None if add => match job.add_material_not_there(name, &unit) {
                    Ok(path) => {
                        out.done.push(format!("{name}: added, bought by the {unit}"));
                        by_plain.insert(plain, path.clone());
                        path
                    }
                    Err(why) => {
                        skip(format!("{name}: {why}"));
                        continue;
                    }
                },
                None => {
                    skip(format!("the job has no material called {name:?}"));
                    continue;
                }
            };
            let e = job.find(&path).ok_or_else(|| format!("no element {path}"))?;
            let known = job.world.get::<Node>(e).map(|n| n.name.clone()).unwrap_or_default();
            // An item with no price yet takes its first: it is its own entry in the price list (call 196).
            if job.world.get::<Supply>(e).is_none() && job.world.get::<crate::Item>(e).is_some() {
                job.world.entity_mut(e).insert(Supply { order: None, price: Val::unknown(&format!("$/{unit}"), "no price entered"), quotes: Default::default() });
            }
            let mut supply = job.world.get::<Supply>(e).cloned().ok_or_else(|| format!("{path} has no price to set"))?;
            let by = supply.price.unit.trim_start_matches("$/").to_string();
            if !unit.eq_ignore_ascii_case(&by) {
                skip(format!("{name}: priced by the {unit}; the job buys {known} by the {by}"));
                continue;
            }
            match priced.iter().find(|(p, w, _, _)| *p == path && *w == whose) {
                Some((_, _, first, at)) if *at != price => {
                    let said = |x: &Option<f64>| x.map(|x| dollars(x, &by)).unwrap_or_else(|| "no price".to_string());
                    skip(format!("{name}: row {first} of this list already prices it, at {}; this row says {}, and was not used", said(at), said(&price)));
                    continue;
                }
                Some(_) => {}
                None => priced.push((path.clone(), whose.clone(), number, price)),
            }
            let new = Val { value: price, unit: supply.price.unit.clone(), status, source: source.clone() };
            // A price of $0.00 is taken, and named (call 206 point 13).
            if price == Some(0.0) {
                let note = format!("{name}{}: priced at $0.00, as the list says: nothing is charged for it, and the estimate marks each line made of it", whose.as_ref().map(|s| format!(" from {s}")).unwrap_or_default());
                if !out.notes.contains(&note) {
                    out.notes.push(note);
                }
            }
            let was = match &whose {
                None => Some(supply.price.clone()),
                Some(s) => supply.quotes.get(s).cloned(),
            };
            let mut changed = was.as_ref() != Some(&new);
            let said = |v: &Val<f64>| v.value.map(|x| dollars(x, &by)).unwrap_or_else(|| "no price".to_string());
            let mut what = match (&whose, &was) {
                (None, Some(old)) if old.value.is_none() => format!("{known}: {}", said(&new)),
                (None, Some(old)) => format!("{known}: {} becomes {}", said(old), said(&new)),
                (Some(s), Some(old)) => format!("{known} from {s}: {} becomes {}", said(old), said(&new)),
                (Some(s), None) => format!("{known} from {s}: {}, which {s} had not quoted", said(&new)),
                (None, None) => format!("{known}: {}", said(&new)),
            };
            if was.as_ref().is_some_and(|old| old.value == new.value && old != &new) {
                what = format!("{known}{}: {} still, now {} ({})", whose.as_ref().map(|s| format!(" from {s}")).unwrap_or_default(), said(&new), status_said(status), source);
            }
            match &whose {
                None => supply.price = new,
                Some(s) => {
                    // Call 108: a supplier's price is the job's too where the job has
                    // none: an estimator who brings in one supplier's list is pricing
                    // the job with it (J25A-12: a list named as Supplier A's priced
                    // nothing, since only a list with no supplier's name set the price
                    // a line is priced by). A price the job has is not written over;
                    // the quote stands beside it.
                    if supply.price.value.is_none() && new.value.is_some() {
                        supply.price = Val { value: new.value, unit: new.unit.clone(), status: new.status, source: format!("{s}'s price list: {}", new.source) };
                        what.push_str(&format!("; the job had no price for it and takes {s}'s"));
                        changed = true;
                    }
                    supply.quotes.insert(s.clone(), new);
                    if !suppliers.contains(s) {
                        suppliers.push(s.clone());
                    }
                }
            }
            // How it is ordered, where the row says: on the job's own row only.
            let order = cols.cell(row, "order");
            if !order.is_empty() && whose.is_some() {
                out.notes.push(format!("row {number}: {name}: how it is ordered ({order}) was not read: that is the job's to say of the material, not one supplier's row's"));
            }
            if !order.is_empty() && whose.is_none() {
                let (figure, per) = match order.split_once(' ') {
                    Some(parts) => parts,
                    None => {
                        skip(format!("{name}: it is ordered at {order:?}, which is not a figure and what per what (1.1 lb/SF)"));
                        continue;
                    }
                };
                let (order_status, order_source) = match (status_read(cols.cell(row, "order status")), cols.cell(row, "order source")) {
                    (Some(s), src) if !src.is_empty() => (s, src.to_string()),
                    _ if !own_words => (status, source.clone()),
                    _ => {
                        skip(format!("{name}: how it is ordered does not say how sure it is and where it came from"));
                        continue;
                    }
                };
                let value = if figure == "unknown" {
                    None
                } else {
                    match units::parse_number(figure) {
                        Ok(x) => Some(x),
                        Err(why) => {
                            skip(format!("{name}: it is ordered at {order:?}: {why}"));
                            continue;
                        }
                    }
                };
                // Whole ones where the list says so; where it does not say, by the rule a
                // typed factor goes by: a package is bought whole (so many SF to the
                // sheet), a weight is not (so many lb to the SF). Call 84.
                let whole = match cols.cell(row, "whole").trim().to_lowercase().as_str() {
                    "yes" | "y" | "true" | "whole" => true,
                    "no" | "n" | "false" | "exact" => false,
                    _ => per.trim().split_once('/').is_some_and(|(_, bottom)| bottom.trim().eq_ignore_ascii_case(by.trim())),
                };
                let new = Order { unit: by.clone(), factor: Val { value, unit: per.trim().to_string(), status: order_status, source: order_source }, round_up: whole };
                if supply.order.as_ref() != Some(&new) {
                    changed = true;
                    what.push_str(&format!("; ordered at {order}{}", if whole { ", whole ones only" } else { ", to the exact amount" }));
                }
                supply.order = Some(new);
            }
            if !changed {
                out.same += 1;
                continue;
            }
            if let Err(why) = check::supply(&path, Kind::Material, &supply) {
                skip(format!("{name}: {why}"));
                continue;
            }
            job.put(&path, supply)?;
            out.done.push(what);
        }
        // A list prices the job's materials. A cost line that is only named as one of
        // them, and is not made of it, does not take its price: an estimator who
        // brings in a supplier's list expects the lines of the same name priced, and
        // is told which are not and what makes them so (J22A-15: "16 set", on what was
        // not said, and every line still read "no price").
        // Call 104: such a line, where it has no price and is priced by the unit the
        // material is, takes the material's own price (not one supplier's quote), with
        // the list's status and source. One that has a price keeps it, and is named
        // with both figures: a price a person typed is not written over by a file.
        let mut took: Vec<String> = Vec::new();
        let mut kept: Vec<String> = Vec::new();
        let mut other_unit: Vec<String> = Vec::new();
        // Only what this list names: the job's price for it as it now stands (what a
        // line with no price takes), and what this list says (what a line that has a
        // price is set against). A list brought in said nothing of, and did nothing
        // to, the lines named as some other list's items.
        let prices: std::collections::HashMap<String, (Val<f64>, Option<f64>)> = by_plain
            .iter()
            .filter_map(|(plain, path)| {
                let listed = priced.iter().find(|(p, _, _, _)| p == path)?.3;
                job.find(path).and_then(|e| job.world.get::<Supply>(e)).map(|s| (plain.clone(), (s.price.clone(), listed)))
            })
            .collect();
        for e in job.order.clone() {
            let Some(on) = job.world.get::<Node>(e).map(|n| if n.name.is_empty() { n.path.clone() } else { n.name.clone() }) else { continue };
            let Some(mut cost) = job.world.get_mut::<crate::Cost>(e) else { continue };
            for l in cost.lines.iter_mut().filter(|l| !l.of_material) {
                let Some((price, listed)) = prices.get(&check::plain_name(&l.description)) else { continue };
                let Some(at) = price.value else { continue };
                let by = price.unit.trim_start_matches("$/");
                let said = format!("{:?} on {on}", l.description);
                if !l.unit_cost.unit.is_empty() && !l.unit_cost.unit.eq_ignore_ascii_case(&price.unit) {
                    other_unit.push(format!("{said} (priced by the {}, the list's by the {by})", l.unit_cost.unit.trim_start_matches("$/")));
                    continue;
                }
                let lists = listed.unwrap_or(at);
                match l.unit_cost.value {
                    None => {
                        l.unit_cost = Val { value: Some(at), unit: price.unit.clone(), status: price.status, source: price.source.clone() };
                        // The job's price for it is the list's, unless the job had one
                        // already: then the line takes the job's, and both are said.
                        took.push(if (lists - at).abs() > 1.0e-9 { format!("{said}: priced at {}, the job's price (this list says {})", dollars(at, by), dollars(lists, by)) } else { format!("{said}: priced at {}, the list's", dollars(at, by)) });
                    }
                    Some(has) if (has - lists).abs() > 1.0e-9 => kept.push(format!("{said} keeps {} (the list says {})", dollars(has, by), dollars(lists, by))),
                    Some(_) => {}
                }
            }
        }
        // What the list names that prices nothing: an item no cost line is named as and
        // nothing is made of. An estimator needs it said: a name spelled another way
        // on the list looks like any other row "added" (J25A-12: a renamed batt
        // insulation was added and not said to match nothing).
        let idle: Vec<String> = {
            let mut named: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut q = job.world.query::<&crate::Cost>();
            for cost in q.iter(&job.world) {
                named.extend(cost.lines.iter().map(|l| check::plain_name(&l.description)));
            }
            let mut q = job.world.query::<&crate::MadeOf>();
            let mut made: std::collections::HashSet<String> = q.iter(&job.world).filter_map(|m| m.0.value.as_deref().map(check::plain_name)).collect();
            // What things set down from an item are priced from, by its line made of the item (call 196).
            let mut things = job.world.query::<(&crate::Placed, &crate::Cost)>();
            let made_of_item: Vec<String> = things.iter(&job.world).filter(|(_, c)| c.lines.iter().any(|l| l.of_material)).filter_map(|(p, _)| p.item.clone()).collect();
            for item in made_of_item {
                if let Some(name) = job.find(&item).and_then(|e| job.world.get::<Node>(e)).map(|n| n.name.clone()) {
                    made.insert(check::plain_name(&crate::editing::item_material_name(&name)));
                }
            }
            let mut idle: Vec<(usize, String)> = by_plain
                .iter()
                .filter(|(plain, _)| !named.contains(*plain) && !made.contains(*plain))
                .filter_map(|(_, path)| {
                    let row = priced.iter().find(|(p, _, _, _)| p == path)?.2;
                    job.find(path).and_then(|e| job.world.get::<Node>(e)).map(|n| (row, n.name.clone()))
                })
                .collect();
            idle.sort();
            idle.into_iter().map(|(_, name)| name).collect()
        };
        // The lines priced are said first: they are what the list did to the estimate
        // (J25A-12: they were last, behind "and 16 more").
        out.done.splice(0..0, took);
        let shown = |all: &[String]| if all.len() > 4 { format!("{}, and {} more", all[..4].join("; "), all.len() - 4) } else { all.join("; ") };
        if !kept.is_empty() {
            out.notes.push(format!("{} named as an item on the list had a price and {}: {}. To take the list's, type it on the line", crate::view::so_many(kept.len(), "cost line"), if kept.len() == 1 { "keeps it" } else { "keep it" }, shown(&kept)));
        }
        if !idle.is_empty() {
            out.notes.push(format!(
                "{} of this list's items {} nothing on the job yet (no cost line is named as {}, and nothing is made of {}): {}",
                idle.len(),
                if idle.len() == 1 { "prices" } else { "price" },
                if idle.len() == 1 { "it" } else { "one" },
                if idle.len() == 1 { "it" } else { "one" },
                shown(&idle)
            ));
        }
        if !other_unit.is_empty() {
            out.notes.push(format!("{} named as an item on the list {} priced by another unit and {} not touched: {}", crate::view::so_many(other_unit.len(), "cost line"), if other_unit.len() == 1 { "is" } else { "are" }, if other_unit.len() == 1 { "was" } else { "were" }, shown(&other_unit)));
        }
        Ok(out)
    })
}
