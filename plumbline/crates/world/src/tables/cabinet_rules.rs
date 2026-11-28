use super::*;

pub(super) const CABINET_COLUMNS: [(&str, &[&str]); 12] = [
    ("code", &["code", "letters", "cabinet code"]),
    ("is", &["what it is", "is", "kind"]),
    ("reads", &["number reads", "number", "reads"]),
    ("width", &["width"]),
    ("depth", &["depth"]),
    ("height", &["height"]),
    ("stands", &["stands", "where it stands"]),
    ("drawn", &["drawn as", "shape"]),
    ("stocked", &["stocked widths", "stocked", "stock widths"]),
    ("stocked heights", &["stocked heights", "stock heights"]),
    ("status", &["how sure", "status"]),
    ("source", &["where from", "source"]),
];

/// A table of cabinet rules read into rules, and the rows that were not used with why. Err only where the table is not one.
pub fn cabinet_rules_from_rows(rows: &[Vec<String>], numbers: Option<&[usize]>) -> Result<(Vec<crate::building::cabinet::Rule>, Vec<(usize, String)>), String> {
    read_cabinet_rules(rows, numbers, &[])
}

pub(super) fn read_cabinet_rules(rows: &[Vec<String>], numbers: Option<&[usize]>, shapes: &[Shape]) -> Result<(Vec<crate::building::cabinet::Rule>, Vec<(usize, String)>), String> {
    use crate::building::cabinet::Rule;
    let cols = columns(rows, &CABINET_COLUMNS, &["code", "reads", "depth"]).ok_or_else(|| {
        "this is not a table of cabinet rules this program can read: within its first rows it has no row of headings with Code, Number reads and Depth".to_string()
    })?;
    let mut rules: Vec<Rule> = Vec::new();
    let mut at: Vec<usize> = Vec::new();
    let mut not_used: Vec<(usize, String)> = Vec::new();
    for (i, row) in rows.iter().enumerate().skip(cols.row + 1) {
        if row.iter().all(|c| c.trim().is_empty()) {
            continue;
        }
        let number = row_number(numbers, i);
        let code = cols.cell(row, "code");
        let called = |why: String| if code.is_empty() { why } else { format!("{code}: {why}") };
        if code.is_empty() {
            not_used.push((number, "it has no code: the letters of the cabinets it reads".to_string()));
            continue;
        }
        let read = || -> Result<Rule, String> {
            for (what, heading) in [("what it is", "is"), ("how its number reads", "reads"), ("its width", "width"), ("its height", "height"), ("where it stands", "stands"), ("what it is drawn as", "drawn"), ("how sure it is", "status"), ("where it came from", "source")] {
                if !cols.has(heading) {
                    return Err(format!("the table has no column for {what}"));
                }
            }
            if cols.cell(row, "status").is_empty() {
                return Err("it does not say how sure it is".into());
            }
            if cols.cell(row, "source").is_empty() {
                return Err("it does not say where it came from".into());
            }
            let rule = Rule::from_cells(code, cols.cell(row, "is"), cols.cell(row, "reads"), cols.cell(row, "width"), cols.cell(row, "depth"), cols.cell(row, "height"), cols.cell(row, "stands"), cols.cell(row, "drawn"), cols.cell(row, "stocked"), cols.cell(row, "stocked heights"), cols.cell(row, "status"), cols.cell(row, "source"))?;
            crate::building::shape::check_drawn_as(shapes, &rule.drawn_as).map_err(|why| format!("drawn as: {why}"))?;
            Ok(rule)
        };
        match read() {
            Ok(rule) => match rules.iter().position(|r| r.code == rule.code) {
                Some(first) => not_used.push((number, called(format!("row {} of this table is the same code, and was used", at[first])))),
                None => {
                    at.push(number);
                    rules.push(rule);
                }
            },
            Err(why) => not_used.push((number, called(why))),
        }
    }
    not_used.sort();
    Ok((rules, not_used))
}

/// The cabinet rules in force as a table: its headings, then a row for each rule, the program's own in their order and the job's own after.
pub fn cabinet_rules_out(job: &mut Job) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = vec![crate::building::cabinet::HEADINGS.iter().map(|h| h.to_string()).collect()];
    for rule in crate::building::cabinet::effective(&job.cabinet_rules) {
        rows.push(rule.cells());
    }
    rows
}

/// Bring a table of cabinet rules in. A rule for letters the job has not is added; one it has is changed to what the row says, and says
/// so; a row that says what is in force already is counted as the same. A rule that says what the program's own says is not carried in the
/// job (it keeps nothing the program has), and a rule changed back to it is taken out. Boxes already made keep their sizes: a rule is for
/// the next code asked for. A row that cannot be used says why; the others still are.
pub fn cabinet_rules_in(job: &mut Job, rows: &[Vec<String>], numbers: Option<&[usize]>) -> Result<Brought, String> {
    use crate::building::cabinet;
    let (rules, not_used) = read_cabinet_rules(rows, numbers, &job.shapes)?;
    let cols = columns(rows, &CABINET_COLUMNS, &["code", "reads", "depth"]);
    all_or_nothing(job, |job| {
        let mut out = Brought { rows: rows.len().saturating_sub(cols.as_ref().map(|c| c.row + 1).unwrap_or(0)), columns_not_used: cols.as_ref().map(|c| c.not_used.clone()).unwrap_or_default(), above: cols.as_ref().map(|c| c.row).unwrap_or(0), not_used, ..Brought::default() };
        for rule in rules {
            let held = cabinet::effective(&job.cabinet_rules).into_iter().find(|r| r.code == rule.code);
            let own = cabinet::starter().iter().find(|r| r.code == rule.code);
            match held {
                Some(h) if h == rule => out.same += 1,
                Some(h) => {
                    let mut changed: Vec<&str> = Vec::new();
                    for (what, differs) in [("what it is", h.is != rule.is), ("how its number reads", h.reads != rule.reads), ("its width", h.width != rule.width), ("its depth", h.depth != rule.depth), ("its height", h.height != rule.height), ("where it stands", h.hung != rule.hung), ("what it is drawn as", h.drawn_as != rule.drawn_as), ("its stocked widths", h.stocked != rule.stocked), ("its stocked heights", h.stocked_heights != rule.stocked_heights), ("where it came from", h.source != rule.source)] {
                        if differs {
                            changed.push(what);
                        }
                    }
                    job.cabinet_rules.retain(|r| r.code != rule.code);
                    if own != Some(&rule) {
                        job.cabinet_rules.push(rule.clone());
                    }
                    out.done.push(format!("{}: changed: {}", rule.code, changed.join(", ")));
                }
                None => {
                    if job.cabinet_rules.len() >= cabinet::MOST_RULES {
                        return Err(format!("a job carries at most {} cabinet rules of its own", cabinet::MOST_RULES));
                    }
                    out.done.push(format!("{}: added", rule.code));
                    job.cabinet_rules.push(rule);
                }
            }
        }
        Ok(out)
    })
}

/// The program's own cabinet rules brought in (`starter`): a rule for letters the job has not is added, and one it has, the program's or the
/// job's own, is kept as it is. The program's own are always in force, so this changes nothing unless the table is another's.
pub fn cabinet_rules_in_missing(job: &mut Job, rows: &[Vec<String>], numbers: Option<&[usize]>) -> Result<(Brought, Vec<String>), String> {
    let cols = columns(rows, &CABINET_COLUMNS, &["code", "reads", "depth"]).ok_or_else(|| "this is not a table of cabinet rules this program can read".to_string())?;
    let have: Vec<String> = crate::building::cabinet::effective(&job.cabinet_rules).into_iter().map(|r| r.code).collect();
    let (mut wanted, mut their_numbers, mut kept) = (Vec::new(), Vec::new(), Vec::new());
    for (i, row) in rows.iter().enumerate() {
        let code = cols.cell(row, "code");
        if i > cols.row && !code.is_empty() && have.iter().any(|h| h.eq_ignore_ascii_case(code)) {
            kept.push(code.to_string());
            continue;
        }
        wanted.push(row.clone());
        their_numbers.push(row_number(numbers, i));
    }
    cabinet_rules_in(job, &wanted, Some(&their_numbers)).map(|brought| (brought, kept))
}

/// A cabinet rule taken away (call 226 point 4). A rule of the job's own goes; one of the program's own eight that the job had changed
/// goes back to the program's own; one the job does not carry is said to be the program's own and is left. Returns whether the job
/// changed, and what a person is told. Boxes already made by the rule stay in the tool chest and still say what made them. A code no rule
/// reads is refused, with the letters the rules do read.
pub fn cabinet_rule_away(job: &mut Job, code: &str) -> Result<(bool, String), String> {
    use crate::building::cabinet;
    let letters = code.trim().to_uppercase();
    let known = cabinet::effective(&job.cabinet_rules);
    if !known.iter().any(|r| r.code == letters) {
        return Err(format!("no cabinet rule reads {letters}. The letters the rules read are {}", known.iter().map(|r| r.code.as_str()).collect::<Vec<_>>().join(", ")));
    }
    let Some(at) = job.cabinet_rules.iter().position(|r| r.code == letters) else {
        return Ok((false, format!("{letters} is the program's own rule and the job carries nothing of it: nothing was changed")));
    };
    // The boxes it made, by the words each one's width says of where it came from.
    let by = format!("the cabinet rule for {letters}:");
    let mut made: Vec<String> = {
        let mut q = job.world.query::<(&Node, &crate::Item)>();
        q.iter(&job.world).filter(|(_, i)| i.width.source.contains(&by)).map(|(n, _)| n.name.clone()).collect()
    };
    made.sort();
    let stay = if made.is_empty() {
        "No box was made by it.".to_string()
    } else {
        format!("The boxes it made stay in the tool chest, each still saying the rule for {letters} made it: {}.", made.join(", "))
    };
    job.cabinet_rules.remove(at);
    let own = cabinet::starter().iter().any(|r| r.code == letters);
    Ok((
        true,
        if own {
            format!("{letters} goes back to the program's own: the job's rule for {letters} was taken away. {stay}")
        } else {
            format!("the rule for {letters} was taken away: no rule reads {letters} now. {stay}")
        },
    ))
}
