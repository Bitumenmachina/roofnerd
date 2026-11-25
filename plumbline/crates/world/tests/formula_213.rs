//! Call 213 point 8: nothing that was a figure before reads otherwise. Every formula this program holds or is tested with is read by the
//! reader as it was before call 213 (kept in `before_213/formula.rs`, word for word) and by the reader now: where the old reader read it,
//! the new reads it to the same tree. Held: every cell of every table in `library/`, every quoted string in the tests and the
//! demonstrations, every cost line formula and every shape figure and connection place of the sample whole house.

#[path = "before_213/formula.rs"]
mod before;
// What the reader before call 213 needs of its crate.
mod value {
    pub use plumbline_world::value::Status;
}

use std::path::Path;

/// Whether the old reader's tree was a figure a line or a shape could use: worked, with every name a plain number, a length or an area, it
/// comes to a number, a length, an area or a volume. (`1/2 in` was a number over a length: per foot, which nothing here is measured in.)
fn was_a_figure(old: &before::Expr) -> bool {
    (0..=2).any(|dim| {
        before::work(old, &|n| Ok(before::Operand { value: Some(2.0), dim, status: plumbline_world::value::Status::Known, shown: n.to_string() })).is_ok_and(|w| (0..=3).contains(&w.dim))
    })
}

fn quoted(text: &str) -> Vec<String> {
    // Every "..." in a source file, with its escapes read (\" and \\ and \n), on one line.
    let mut out = Vec::new();
    let bytes: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == '"' {
            let mut s = String::new();
            i += 1;
            while i < bytes.len() && bytes[i] != '"' && bytes[i] != '\n' {
                if bytes[i] == '\\' && i + 1 < bytes.len() {
                    s.push(match bytes[i + 1] {
                        'n' => ' ',
                        c => c,
                    });
                    i += 2;
                } else {
                    s.push(bytes[i]);
                    i += 1;
                }
            }
            out.push(s);
        }
        i += 1;
    }
    out
}

fn cells(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let mut cell = String::new();
        let mut in_quotes = false;
        for c in line.chars() {
            match c {
                '"' => in_quotes = !in_quotes,
                ',' if !in_quotes => out.push(std::mem::take(&mut cell)),
                c => cell.push(c),
            }
        }
        out.push(cell);
    }
    out
}

fn files(dir: &Path, ext: &str, into: &mut Vec<std::path::PathBuf>) {
    if let Ok(read) = std::fs::read_dir(dir) {
        for e in read.flatten() {
            let p = e.path();
            if p.is_dir() {
                files(&p, ext, into);
            } else if p.extension().is_some_and(|x| x == ext) {
                into.push(p);
            }
        }
    }
}

#[test]
fn nothing_that_was_a_figure_before_reads_otherwise() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut corpus: Vec<String> = Vec::new();
    let mut csvs = Vec::new();
    files(&root.join("library"), "csv", &mut csvs);
    // Of each table, the columns that hold a figure (the rest are names and words: a material called "3/4 in" is no formula).
    let figure_columns = ["Quantity", "Across from", "Across to", "Back from", "Back to", "Up from", "Up to", "There when", "Across", "Back", "Up", "Stretch-out"];
    for f in &csvs {
        let text = std::fs::read_to_string(f).unwrap();
        let rows: Vec<Vec<String>> = text.lines().map(|l| cells(l)).collect();
        let head = rows.first().cloned().unwrap_or_default();
        for row in rows.iter().skip(1) {
            for (i, cell) in row.iter().enumerate() {
                if head.get(i).is_some_and(|h| figure_columns.contains(&h.as_str())) {
                    corpus.push(cell.trim_start_matches('\'').to_string());
                }
            }
        }
    }
    let (mut rs, mut sh) = (Vec::new(), Vec::new());
    files(&root.join("crates"), "rs", &mut rs);
    files(&root.join("demos"), "sh", &mut sh);
    for f in rs.iter().chain(sh.iter()).filter(|f| !f.file_name().is_some_and(|n| n.to_string_lossy().starts_with("call213m") || n.to_string_lossy().starts_with("formula_213"))) {
        corpus.extend(quoted(&std::fs::read_to_string(f).unwrap_or_default()));
    }
    // The sample whole house: every cost line's formula, and what it was typed with.
    let mut job = plumbline_world::sample::house_priced();
    let text = plumbline_world::file::save(&mut job);
    corpus.extend(quoted(&text));
    corpus.sort();
    corpus.dedup();
    let (mut read_before, mut same, mut newly) = (0usize, 0usize, 0usize);
    let mut otherwise: Vec<String> = Vec::new();
    let mut not_figures = 0usize;
    for text in &corpus {
        match (before::parse(text), plumbline_world::formula::parse(text)) {
            (Ok(old), new) => {
                read_before += 1;
                if !was_a_figure(&old) {
                    not_figures += 1;
                    continue;
                }
                match new {
                    Ok(new) if format!("{old:?}") == format!("{new:?}") => same += 1,
                    Ok(new) => otherwise.push(format!("{text:?}: was {old:?}, is {new:?}")),
                    Err(why) => otherwise.push(format!("{text:?}: was read, is refused: {why}")),
                }
            }
            (Err(_), Ok(_)) => newly += 1,
            (Err(_), Err(_)) => {}
        }
    }
    eprintln!("{} strings tried; {read_before} read before, of which {not_figures} were no figure (a number over a length) and {same} were figures and read to the same tree now; {newly} were refused before and are read now", corpus.len());
    for o in &otherwise {
        eprintln!("  differs: {o}");
    }
    assert!(otherwise.is_empty(), "{otherwise:#?}");
    assert!(read_before > 3000 && same + not_figures == read_before, "{read_before} {same} {not_figures}");
}

/// The forms the call gives meaning to were refusals before: none was a figure.
#[test]
fn what_is_read_now_that_was_refused_was_never_a_figure() {
    for text in ["1/2 in", "5 1/2 in", "3 ft 6 in", "3'6\"", "3'", "6\"", "3 ft 6 1/2 in", "-1/2 in"] {
        let before = before::parse(text);
        // `-1/2 in` and `1/2 in` parsed before as a number over a length; they are not a length then.
        if let Ok(old) = before {
            let w = before::work(&old, &|n| Err(format!("{n}"))).ok();
            assert!(w.is_none_or(|w| w.dim != 1), "{text}: it was a length before");
        }
    }
}
