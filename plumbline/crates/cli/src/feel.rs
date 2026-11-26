//! How tools feel, at the command line (DECISIONS.md call 219, the model's side): a change tried and taken back, a door or a
//! window moved, a copy and a row of copies, and one search by name.
//!
//!   plumbline try <job> <wall|door|window|run|place|move|turn|copy|repeat> <as that verb takes it>
//!   plumbline copy <job> <thing> --by "6 ft north" [--source "<who copies>"]
//!   plumbline repeat <job> <thing> --times 4 --apart "3 ft east" [--source "<who copies>"]
//!   plumbline move <job> <door or window> (--by "5 ft east" | --offset "6 ft" [--to <wall>]) --status <S> --source "<where from>"
//!   plumbline find <job> "<words>"
//!
//! Every one goes by `plumbline_world::editing`: `try` is `try_change`, and says what the real verb would, in the words it
//! would; nothing is written to the file.

use std::io::Write as _;

use plumbline_world::components::Node;
use plumbline_world::editing::{self as ed, AtOpening, Change, Entered, HowTall, OpeningTo};
use plumbline_world::units;
use plumbline_world::value::Status;
use plumbline_world::{file, Job, Kind, P2};

macro_rules! say {
    ($($arg:tt)*) => {{
        let _ = writeln!(std::io::stdout(), $($arg)*);
    }};
}

/// What is typed after the verb and the file: the words before the first flag, and the flags, each given once.
struct Typed {
    plain: Vec<String>,
    flags: Vec<(String, String)>,
    used: Vec<String>,
}

const BARE: [&str; 3] = ["--closed", "--alone", "--anyway"];

impl Typed {
    fn of(rest: &[String]) -> Result<Typed, String> {
        let split = rest.iter().position(|a| a.starts_with("--")).unwrap_or(rest.len());
        let (plain, flags) = rest.split_at(split);
        let mut given: Vec<(String, String)> = Vec::new();
        let mut it = flags.iter();
        while let Some(flag) = it.next() {
            if given.iter().any(|(f, _)| f == flag) {
                return Err(format!("{flag} given twice"));
            }
            let value = if BARE.contains(&flag.as_str()) { String::new() } else { it.next().cloned().ok_or_else(|| format!("{flag} needs a value"))? };
            given.push((flag.clone(), value));
        }
        Ok(Typed { plain: plain.to_vec(), flags: given, used: Vec::new() })
    }
    fn flag(&mut self, name: &str) -> Option<String> {
        self.used.push(name.to_string());
        self.flags.iter().find(|(f, _)| f == name).map(|(_, v)| v.clone())
    }
    fn bare(&mut self, name: &str) -> bool {
        self.flag(name).is_some()
    }
    fn need(&mut self, name: &str, like: &str) -> Result<String, String> {
        self.flag(name).ok_or_else(|| format!("{name} is required, like {like}"))
    }
    fn nothing_left(&self, cmd: &str) -> Result<(), String> {
        match self.flags.iter().find(|(f, _)| !self.used.contains(f)) {
            Some((f, _)) => Err(format!("{f} is not something {cmd} takes; it would be ignored")),
            None => Ok(()),
        }
    }
}

fn points(text: &str) -> Result<Vec<P2>, String> {
    text.split_whitespace()
        .map(|pair| {
            let (x, y) = pair.split_once(',').ok_or_else(|| format!("{pair:?} is not a point; write it x,y in feet"))?;
            Ok([units::parse_number(x)?, units::parse_number(y)?])
        })
        .collect()
}

fn one_point(text: &str, name: &str) -> Result<P2, String> {
    match points(text)?.as_slice() {
        [p] => Ok(*p),
        _ => Err(format!("{name} is one point, x,y in feet")),
    }
}

fn kind_of(word: &str) -> Result<Kind, String> {
    match word {
        "door" => Ok(Kind::Door),
        "window" => Ok(Kind::Window),
        "duct" => Ok(Kind::Duct),
        "pipe" => Ok(Kind::Pipe),
        "conduit" => Ok(Kind::Conduit),
        "cable" => Ok(Kind::Cable),
        "fixture" => Ok(Kind::Fixture),
        "equipment" => Ok(Kind::Equipment),
        "furnishing" => Ok(Kind::Furnishing),
        other => Err(format!("{other:?} is not a kind this verb takes")),
    }
}

/// How sure what is entered is, and who says so: `--status` and `--source`, both given.
fn how_of(t: &mut Typed) -> Result<(Status, String), String> {
    let status = match t.flag("--status").as_deref() {
        Some("Known") => Status::Known,
        Some("Assumed") => Status::Assumed,
        Some(other) => return Err(format!("{other:?} is not a status a thing can be drawn with; use Known or Assumed")),
        None => return Err("--status is required: say whether what is drawn is Known or Assumed".into()),
    };
    Ok((status, t.flag("--source").ok_or("--source is required: say where it came from")?))
}

/// A copy is Assumed whatever is said, so it takes no status: only who copied it, which it is without being told.
fn copier_of(t: &mut Typed, cmd: &str) -> Result<String, String> {
    if t.flags.iter().any(|(f, _)| f == "--status") {
        return Err(format!("--status is not something {cmd} takes: a copy is Assumed, whatever it was copied from"));
    }
    Ok(t.flag("--source").unwrap_or_else(|| "copied at the command line".to_string()))
}

/// What a person typed to move a door, as a change: along its wall (`--by`), or to a place on a wall (`--offset`, with `--to` another wall).
fn opening_to(t: &mut Typed) -> Result<OpeningTo, String> {
    let (by, offset, to) = (t.flag("--by"), t.flag("--offset"), t.flag("--to"));
    match (by, offset, to) {
        (Some(by), None, None) => {
            // Along its wall: east and west, or north and south, as the wall runs; the model works the way along from the wall's own line.
            let (east, north) = ed::read_move(&by)?;
            if east != 0.0 && north != 0.0 {
                return Err("a door or a window slides along its wall: say one way, \"5 ft east\" for a wall that runs east and west".into());
            }
            Ok(OpeningTo::Along(if east != 0.0 { east } else { north }))
        }
        (None, Some(offset), wall) => Ok(OpeningTo::Offset { wall, offset }),
        (None, None, Some(_)) => Err("--to says which wall; say where along it with --offset \"6 ft\" (its middle, from the wall's first end)".into()),
        (None, None, None) => Err("say how it is moved: --by \"5 ft east\" along its wall, or --offset \"6 ft\" (with --to <wall> for another wall)".into()),
        _ => Err("--by moves it along its wall; --offset says where along a wall its middle is: one of the two".into()),
    }
}

/// The along-the-wall distance of a `--by` typed as east or north, turned into feet toward the wall's second end: the model needs the wall's way.
fn along_the_wall(job: &mut Job, door: &str, to: OpeningTo) -> Result<OpeningTo, String> {
    let OpeningTo::Along(d) = to else { return Ok(to) };
    let e = job.find(door).ok_or_else(|| format!("no element {door}"))?;
    let parent = job.world.get::<Node>(e).and_then(|n| n.parent.clone()).ok_or("it is in no wall")?;
    let w = job.find(&parent).and_then(|x| job.world.get::<plumbline_world::Wall>(x).cloned()).ok_or("its wall is not there")?;
    let line = w.line_ft().ok_or("its wall's line is not known")?;
    let (dx, dy) = (line[1][0] - line[0][0], line[1][1] - line[0][1]);
    // A wall that runs more east and west than north and south is moved east or west; the other, north or south.
    let toward_second = if dx.abs() >= dy.abs() { d * dx.signum() } else { d * dy.signum() };
    Ok(OpeningTo::Along(toward_second))
}

/// A change from what was typed for the verb it is of.
fn change_of(job: &mut Job, cmd: &str, t: &mut Typed) -> Result<(Change, Entered<'static>, String), String> {
    let (change, status, source) = match (cmd, t.plain.clone().as_slice()) {
        ("wall", [storey, type_path]) => {
            let (status, source) = how_of(t)?;
            let pts = points(&t.need("--points", "\"0,0 20,0 20,10\" (feet)")?)?;
            let (height, top, closed) = (t.flag("--height"), t.flag("--top"), t.bare("--closed"));
            let tall = match (height, top) {
                (Some(h), None) => HowTall::Height(h),
                (None, Some(l)) => HowTall::Level(l),
                _ => return Err("say how tall the walls are: --height \"9 ft\", or --top <the level their top is held to>; one of them".into()),
            };
            (Change::Walls { storey: storey.clone(), type_path: type_path.clone(), points: pts, closed, tall }, status, source)
        }
        ("door" | "window", [wall, type_path]) => {
            let (status, source) = how_of(t)?;
            let (offset, at) = (t.flag("--offset"), t.flag("--at"));
            let at = match (offset, at) {
                (Some(o), None) => AtOpening::Offset(o),
                (None, Some(p)) => AtOpening::Point(one_point(&p, "--at")?),
                _ => return Err("say where along the wall its center is: --offset \"5 ft\" from the wall's first end, or --at \"5,0\" a point on the plan; one of them".into()),
            };
            let sill = t.need("--sill", "\"0 ft\" for a door, \"3 ft\" for a window")?;
            (Change::Opening { wall: wall.clone(), kind: kind_of(cmd)?, type_path: type_path.clone(), at, sill }, status, source)
        }
        ("run", [storey, what, type_path]) => {
            let (status, source) = how_of(t)?;
            let pts = points(&t.need("--points", "\"2,2 14,2 14,7\" (feet)")?)?;
            if t.flag("--rises-to").is_some() {
                return Err("a riser is not tried: try the run, or make the riser".into());
            }
            let height = t.need("--height", "\"8 ft\" above the level's floor")?;
            (Change::Runs { storey: storey.clone(), kind: kind_of(what)?, type_path: type_path.clone(), points: pts, height, system: t.flag("--system") }, status, source)
        }
        ("place", [storey, item]) => {
            let (status, source) = how_of(t)?;
            let at = one_point(&t.need("--at", "\"14,7\" (feet)")?, "--at")?;
            (Change::Item { storey: storey.clone(), item: item.clone(), at, offset: t.flag("--offset").unwrap_or_else(|| "0 ft".to_string()) }, status, source)
        }
        ("place", [storey, what, name]) => {
            let (status, source) = how_of(t)?;
            let at = one_point(&t.need("--at", "\"14,7\" (feet)")?, "--at")?;
            let size = match t.flag("--size") {
                None => None,
                Some(text) => match text.split(',').map(units::parse_number).collect::<Result<Vec<f64>, String>>()?.as_slice() {
                    [a, b, c] => Some([*a, *b, *c]),
                    _ => return Err("--size is along,across,up in feet, like \"2,2,6\"".into()),
                },
            };
            let offset = t.need("--offset", "\"0 ft\" above the level's floor")?;
            (Change::Place { storey: storey.clone(), kind: kind_of(what)?, what: name.clone(), at, offset, size, system: t.flag("--system") }, status, source)
        }
        ("move", [path]) => {
            let (status, source) = how_of(t)?;
            let opening = job.kind(path).is_some_and(|k| k.is_opening());
            if opening {
                let to = opening_to(t)?;
                (Change::MoveOpening { path: path.clone(), to: along_the_wall(job, path, to)? }, status, source)
            } else {
                let (east, north) = ed::read_move(&t.need("--by", "\"4 ft west\", \"3 ft 6 in north and 2 ft east\", or \"3,2\" (feet east, feet north)")?)?;
                (Change::Move { path: path.clone(), east, north, alone: t.bare("--alone") }, status, source)
            }
        }
        ("turn", [path]) => {
            let (status, source) = how_of(t)?;
            let by: f64 = t.need("--by", "\"90\" (degrees counter-clockwise from east; 90 is a quarter turn)")?.trim().trim_end_matches("deg").trim().parse().map_err(|_| "--by is a number of degrees, like 90 or -45".to_string())?;
            (Change::Turn { path: path.clone(), degrees: by }, status, source)
        }
        ("copy", [path]) => {
            let source = copier_of(t, cmd)?;
            let (east, north) = ed::read_move(&t.need("--by", "\"6 ft north\", or \"3,2\" (feet east, feet north)")?)?;
            (Change::Copy { path: path.clone(), east, north }, Status::Assumed, source)
        }
        ("repeat", [path]) => {
            let source = copier_of(t, cmd)?;
            let times: u32 = t.need("--times", "4 (how many copies)")?.trim().parse().map_err(|_| "--times is a whole number of copies, like 4".to_string())?;
            let (east, north) = ed::read_move(&t.need("--apart", "\"3 ft east\" (how far each copy is from the last)")?)?;
            (Change::Row { path: path.clone(), times, east, north }, Status::Assumed, source)
        }
        (other, _) => return Err(format!("{other} is not tried as it was typed: try takes wall, door, window, run, place, move, turn, copy or repeat, each as that verb takes it")),
    };
    t.nothing_left(cmd)?;
    // The source of what is entered lives as long as the program does: it is the one string, leaked once.
    let source: &'static str = Box::leak(source.into_boxed_str());
    Ok((change, Entered { status, source }, cmd.to_string()))
}

fn write_back(job: &mut Job, path: &str) -> Result<(), String> {
    std::fs::write(path, file::save(job)).map_err(|e| format!("cannot write {path}: {e}"))
}

/// `try`, `copy`, `repeat`, `find`, and `move` of a door or a window.
pub fn run(args: &[String]) -> Result<(), String> {
    let cmd = args[0].as_str();
    let path = args.get(1).ok_or_else(|| format!("{cmd} takes a job file"))?;
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let (mut job, _) = file::load(&text).map_err(|e| format!("{path}: {e}"))?;
    match cmd {
        "find" => {
            let [_, _, words] = args else { return Err("find takes a job and the words to find: plumbline find <job> \"lav\"".into()) };
            let found = ed::find_in_job(&mut job, words);
            if found.is_empty() {
                say!("{}", ed::nothing_said(words));
            } else {
                say!("{} found for {:?}:", found.len(), words.trim());
                for f in &found {
                    say!("  {}", f.said());
                }
            }
            Ok(())
        }
        "try" => {
            let verb = args.get(2).ok_or("try takes a verb and what it takes: plumbline try <job> wall <level> <type> --points ...")?;
            // Every other verb that makes or takes away a thing is tried on a copy of the job, and says what the verb says; the file is not touched.
            if matches!(verb.as_str(), "level" | "storey" | "type" | "system" | "footing" | "low-slope" | "drains" | "mark" | "floor" | "room" | "room-line" | "beam" | "post" | "pad" | "remove" | "split" | "fitting" | "connect" | "cabinets") {
                return try_on_a_copy(path, &text, verb, &args[3..]);
            }
            // `--timed N`: the change is tried N times, each printed with the time it took, and the least of them (call 226 point 1).
            let mut rest: Vec<String> = args[3..].to_vec();
            let mut timed: Option<u32> = None;
            if let Some(i) = rest.iter().position(|a| a == "--timed") {
                let n = rest.get(i + 1).ok_or("--timed needs a number of tries, like 5")?;
                timed = Some(n.trim().parse().ok().filter(|n| *n > 0).ok_or("--timed is a whole number of tries, like 5")?);
                rest.drain(i..i + 2);
            }
            let mut t = Typed::of(&rest)?;
            let (change, how, _) = change_of(&mut job, verb, &mut t)?;
            let mut times: Vec<f64> = Vec::new();
            for _ in 1..timed.unwrap_or(1) {
                let began = std::time::Instant::now();
                let _ = ed::try_change(&mut job, &change, how);
                times.push(began.elapsed().as_secs_f64());
            }
            let began = std::time::Instant::now();
            let tried = ed::try_change(&mut job, &change, how);
            times.push(began.elapsed().as_secs_f64());
            let lines = tried.said();
            if let Some(why) = &tried.refused {
                return Err(format!("tried {verb}: it would be refused: {why}\nnothing was changed"));
            }
            say!("tried {verb}:");
            for l in lines {
                say!("  {l}");
            }
            if timed.is_some() {
                for (i, x) in times.iter().enumerate() {
                    say!("try {}: {x:.3} s", i + 1);
                }
                say!("the least of {}: {:.3} s", times.len(), times.iter().cloned().fold(f64::INFINITY, f64::min));
                // Where the time goes: the job written out and read back, which is what a try does to put it back.
                let began = std::time::Instant::now();
                let saved = file::save(&mut job);
                let wrote = began.elapsed().as_secs_f64();
                let began = std::time::Instant::now();
                let _ = file::load(&saved);
                say!("writing the whole job out: {wrote:.3} s; reading it back: {:.3} s", began.elapsed().as_secs_f64());
            }
            say!("nothing was changed: {path} is as it was");
            Ok(())
        }
        "copy" | "repeat" | "move" => {
            let mut t = Typed::of(&args[2..])?;
            let (change, how, _) = change_of(&mut job, cmd, &mut t)?;
            let mut said = Vec::new();
            match &change {
                Change::Copy { path: from, east, north } => {
                    let c = ed::copy_by(&mut job, from, *east, *north, how)?;
                    said.push(c.said());
                    said.push(format!("  made {}", c.made.join(", ")));
                    said.extend(c.left_behind);
                }
                Change::Row { path: from, times, east, north } => {
                    let all = ed::copy_row(&mut job, from, *times, *east, *north, how)?;
                    said.push(format!("{}: {}", all.first().map(|c| c.from_name.clone()).unwrap_or_default(), ed::row_said(*times, *east, *north)));
                    for c in &all {
                        said.push(format!("  {} [{}]", c.said(), c.made.join(", ")));
                        said.extend(c.left_behind.iter().map(|l| format!("    {l}")));
                    }
                }
                Change::MoveOpening { path: door, to } => {
                    let moved = ed::move_opening(&mut job, door, to, how)?;
                    said.extend(moved.said());
                }
                _ => return Err(format!("{cmd} is not taken here")),
            }
            write_back(&mut job, path)?;
            for l in said {
                say!("{l}");
            }
            Ok(())
        }
        other => Err(format!("{other} is not a verb of this kind")),
    }
}

/// A verb tried on a copy of the job: the reply the verb gives, or the refusal it gives, and the file as it was.
fn try_on_a_copy(path: &str, text: &str, verb: &str, rest: &[String]) -> Result<(), String> {
    let dir = std::env::temp_dir().join(format!("plumbline-try-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot make room to try it in: {e}"))?;
    let name = std::path::Path::new(path).file_name().map(|n| n.to_owned()).unwrap_or_else(|| "job.json".into());
    let copy = dir.join(name);
    let copy_text = copy.to_string_lossy().to_string();
    let done = (|| {
        std::fs::write(&copy, text).map_err(|e| format!("cannot make a copy to try it on: {e}"))?;
        let mut again: Vec<String> = vec![verb.to_string(), copy_text.clone()];
        again.extend(rest.iter().cloned());
        say!("tried {verb} (the reply is the verb's own; the clashes it makes are what `clashes` says once it is made):");
        crate::draw(&again)
    })();
    let _ = std::fs::remove_dir_all(&dir);
    match done {
        Ok(()) => {
            say!("nothing was changed: {path} is as it was");
            Ok(())
        }
        Err(why) => Err(format!("tried {verb}: it would be refused: {why}\nnothing was changed")),
    }
}

/// Whether a `move` is of a door or a window: those go here, every other thing the way it always did.
pub fn is_opening_move(args: &[String]) -> bool {
    let (Some(path), Some(thing)) = (args.get(1), args.get(2)) else { return false };
    let Ok(text) = std::fs::read_to_string(path) else { return false };
    let Ok((mut job, _)) = file::load(&text) else { return false };
    !thing.starts_with("--") && job.kind(thing).is_some_and(|k| k.is_opening())
}
