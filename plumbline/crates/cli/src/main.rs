//! The command line over the job world. It reads and edits the same saved job the
//! desktop application will.
//!
//!   plumbline sample <file>                      write the sample job
//!   plumbline house <file>                       write the sample whole house, with its MEP
//!   plumbline takeoff <file>                     each storey's walls and floors, and each system's runs by size
//!   plumbline show   <file>                      every element and value, flagged
//!   plumbline set    <file> <path> <field> <value|unknown> --status <Known|Assumed|Unknown> --source "<where from>" [--line "<cost line>"]
//!
//! A value is typed the way the drawing or the tape gives it and is kept in that unit:
//! a length as `2.5 ft`, `30 in`, `19 1/2"` or `25' 6 1/2"`; a pitch as `6`, `6:12` or
//! `1/4:12`. A length with no unit is refused, never read as feet.
//!
//!   plumbline check  <file>                      load, re-derive, confirm the file is exactly what a save would write
//!   plumbline lines  <file>                      the lines the planes imply, set against the lines entered
//!   plumbline apply  <file> <path> <condition> --source "<who says>"   give an element a condition's cost lines
//!   plumbline roof   <file> <name> --points "0,0 40,0 40,30 0,30" --pitch 6:12 --eave-height "10 ft" --status <S> --source "<where from>"
//!                                                a new roof solved from a footprint (coordinates in feet)
//!
//! On a roof solved from a footprint, `set` also takes `edge:<n>` (a pitch, or `gable`) and
//! `move:<n>` (a length; the edge moves out by it, in if it is negative), and solves again.

use std::fmt::Write as _;
use std::io::Write as _;
use std::process::ExitCode;

mod cad;
mod feel;
mod found;
mod marks;

use plumbline_world::value::Status;
use plumbline_world::units::{self, Len};
use plumbline_world::{file, plans, roof, sample, solve, view, Footprint, Rise, Val};

/// Print a line. A reader that has stopped listening (`| head -1`) is not an error and
/// must not be a panic, which is what `println!` makes of it.
macro_rules! say {
    ($($arg:tt)*) => {{
        let _ = writeln!(std::io::stdout(), $($arg)*);
    }};
}

mod drawing;
mod look;
mod money;
mod show;
mod tables_cmd;

use drawing::*;
use look::*;
use money::*;
use show::*;
use tables_cmd::*;

fn main() -> ExitCode {
    // An argument that is not text is refused, not a panic.
    let mut args: Vec<String> = Vec::new();
    for a in std::env::args_os().skip(1) {
        match a.into_string() {
            Ok(s) => args.push(s),
            Err(bad) => {
                eprintln!("plumbline: the argument {:?} is not readable text", bad.to_string_lossy());
                return ExitCode::FAILURE;
            }
        }
    }
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("plumbline: {}", plainly(&e, &args));
            ExitCode::FAILURE
        }
    }
}

fn usage() -> String {
    "usage:\n  plumbline sample <file>\n  plumbline house <file>\n  plumbline sixteen <file>   (the sample whole house with a made-up price on everything, in all sixteen divisions)\n  plumbline show <file> [<thing>]   (a thing alone, with its rows; a wall also says what stands on it and what it stands on)\n  plumbline set <file> <path> <field> <value|unknown> --status <Known|Assumed|Unknown> --source \"<where from>\" [--line \"<cost line>\"] [--with \"name=value\"]\n  plumbline set <file> <item or material, by its name> price <$/unit|unknown> [--supplier \"<name>\"] --status <Known|Assumed|Unknown> --source \"<where from>\"   (the estimator's price, or a named supplier's with --supplier (a supplier new to the job is added); unknown: no price, never $0.00; it says the lines the price prices, on how many things, and the job cost before and after; the same price again changes nothing; a material by its path goes by the same road)\n  plumbline set <file> <item> priced_as \"<assembly>\"|unknown   (what an item is priced as: an assembly that goes on its kind, which each of it set down takes, those already set down following; unknown: priced as nothing, and what came by it comes off; no --status or --source: it is a name)\n  plumbline check <file>\n  plumbline lines <file>\n  plumbline takeoff <file>\n  plumbline drawings <file>\n  plumbline clashes <file> [--clearance \"2 in\"]\n  plumbline vif <file>   (what is still to be verified in the field, and the dollars riding on each)\n  plumbline trim <file> [--pieces]   (--pieces: where each piece of outside trim lies, one by one, drawn only where a cost line prices it, with each kind's total beside what its lines price; the outside trim's figures, worked from what is drawn and priced by nothing here: each eave's and rake's length and how far it overhangs the wall under it, each wall's outside corners and the corner board they take, each window's and door's casing; a cost line names the figure it is priced by, like \"true_length * overhang\")\n  plumbline gable <file>   (each wall that rises to the roof under a rake or the high edge of a shed roof: the gable's area, which a line prices as \"net_area + gable_area\", and the wall's top as points, how far along the wall and how high; a wall under an eave, or with no roof over it, has none and says so; the frame of a gable is not counted)\n  plumbline finishes <file>   (what each wall, roof plane, window and door is finished with, read from what is priced on it)\n  plumbline price <file> [list]   (list: the price list, a row for each material and item a line is made of or that has a price, no price first: its unit, the estimator's price and each supplier's, how sure and where from, the lines it prices, the things they are on and the dollars riding)\n  plumbline estimate <file> [by-item]   (every line under the thing it is on; by-item: each item once, with its places, division by division)\n  plumbline priced <file>\n  plumbline material <file> \"<name>\" <unit it is bought by: lb, sheet, SF>\n  plumbline line <file> <path> \"<description>\" <quantity, or a measure like \"60 SF\" with --status and --source> [--cost-type <material|labor|...>] [--division <1 to 16>] [--of-material]\n  plumbline remove-line <file> <path> \"<cost line>\"   (takes that one line off the thing)\n  plumbline division <file> <path> <1 to 16, or its title> [--line \"<cost line>\"]\n  plumbline cost-type <file> <path> <material|labor|fabrication|supervision|subcontract|general> [--line \"<cost line>\"]\n  plumbline labor <file> <path> <on|off> [--line \"<cost line>\"]\n  plumbline apply <file> <path> <assembly> --source \"<who says>\" [--change]   (--change: in place of the assembly it has; lines the two share keep their prices, and take the new assembly's quantity where nobody changed it by hand; each is named with what it was and what it is)\n  plumbline export <prices|assemblies> <file> <out.csv|out.xlsx>\n  plumbline export estimate <file> <out.xlsx|out.csv>   (.xlsx: a workbook whose amounts, recap and price are formulas; .csv: the lines as values)\n  plumbline export ifc <file> <out.ifc>   (the building as an IFC4 model, for another trade or a designer)\n  plumbline export gltf <file> <out.glb>   (the house as one file to look at in a browser, on a phone or in any 3D program, in metres, each wall, floor, roof plane and thing a named part coloured by what it is finished in; what it does not carry is said)\n  plumbline export bcf <file> <out.bcfzip> [--holes]   (the clashes as an issue list another program opens, each naming its two things as the IFC file does; --holes adds each hole to cut)\n  plumbline export <supplier|subcontractor|pm> <file> <out.xlsx|out.csv> [--division <1 to 16>] [--system \"<name>\"]   (--division and --system cut the subcontractor's list to one trade's or one system's scope; one lens on the estimate: what to quote; the subcontract scope; the lines, budget and what is to verify)\n  plumbline import prices <file> <a price list: CSV or XLSX> [--sheet \"<name>\"] [--supplier \"<whose prices>\"] [--add] [--status <Known|Assumed> --source \"<where from>\"]\n  plumbline import assemblies <file> <an assembly library this program wrote: CSV or XLSX> [--sheet \"<name>\"]\n  plumbline import assemblies <file> starter   (the starter library the program ships with: whole assemblies for walls, floors, rooms, roofs, openings and runs, a line in every division, no prices)\n  plumbline import items <file> <an item library: CSV or XLSX, or the word starter for the one the program ships with> [--sheet \"<name>\"]   (the things that can be placed, by trade: each row an item, its kind, what it is drawn as and its sizes; a row whose item the job has changes it)\n  plumbline export items <file> <out.csv>   (the job's items as that table, to open, change and bring back in)\n  plumbline export vif <file> <out.xlsx|out.csv>   (what is to verify in the field, as a sheet to print and take to the job: a box to tick, the dollars riding, where, what, what to do, and a blank for what was found)\n  plumbline export field-sheet <file> <out.xlsx|out.csv>   (the same list as a sheet to fill on site: a row for each value to measure, the most dollars first, each with a reference of its own and what the job has now; fill As found, Measured by and On)\n  plumbline import field-sheet <file> <filled.xlsx|filled.csv> [--anyway]   (each row with something As found sets that value Known, as a typed measurement does; all of it or none of it, a refusal named by its row; --anyway puts a row over a value changed or measured since the sheet went out)\n  plumbline export database <file> <my database.xlsx, or a folder>   (every table of the job, items, shapes, where the items connect, assemblies and prices, one tab each, to keep or to take to another program)\n  plumbline import database <file> <my database.xlsx, a folder of tables, or starter>   (each tab by its own table, whole or not at all; a tab or a column it does not know is named and left alone)\n  plumbline export shapes <file> <out.csv>   (the shapes the job's items are drawn as, one row a block: each figure a formula in width, depth and height, such as min(100 mm, 0.14 * height); across from the middle, back from the back, up from the foot; the rest of the tool chest with the items)\n  plumbline export cabinet-rules <file> <out.csv|out.xlsx>   (how the codes of cabinets are read, one row a rule: the letters (B, SB, W), what it is, how its number reads, its width, depth and height, where it stands, what it is drawn as and the widths stocked, each Assumed; to open, change and bring back in)\n  plumbline import cabinet-rules <file> <a table of cabinet rules: CSV or XLSX, or the word starter for the one the program ships with> [--sheet \"<name>\"]   (a code the tool chest has not is made by its rule when a run asks for it; boxes already made keep their sizes)\n  plumbline import shapes <file> <a shape table: CSV or XLSX> [--sheet \"<name>\"]   (shapes of your own, for items to be drawn as; bring them in before the items that name them; a shape that cannot be worked is not used and says which block and which figure)\n  plumbline items <file> [\"<item>\"]   (the job's items, trade by trade, each with how many connections it has and, after them, how many of the starter items have none; with an item's name, its shape's blocks and its connections at its sizes, each between its figures)\n  plumbline connection <file> \"<item>\" add \"<name>\" --takes <pipe|duct|conduit|power> [--carries <cold|hot|waste|vent|storm|gas>] [--goes <in|out|both>] --across \"-4 in\" --back \"0\" --up \"height - 12 in\" [--faces <back|front|left|right|up|down>] [--size \"1/2 in\"] --status Assumed --source \"<where from>\"   (a place on the item where a pipe, a duct, a conduit or power meets it: across from the middle, back from the back, up from the foot, each a formula in width, depth and height, and on or inside the item at its own size; every thing set down from the item has it, and it says how many)\n  plumbline connection <file> \"<item>\" set \"<name>\" <takes|carries|goes|across|back|up|faces|size> \"<value>\" [--status Assumed --source \"<where from>\", for a size]   (one thing about a connection changed; a size typed with nothing else says it was typed at the command line; the things set down from the item follow, and a run left standing off its connection is counted)\n  plumbline connection <file> \"<item>\" take \"<name>\"   (a connection taken away from the item and from what is set down from it)\n  plumbline export connections <file> <out.csv>   (where the job's items connect, one row a connection, to open, change and bring back in)\n  plumbline import connections <file> <a connections table: CSV or XLSX, or the word starter for the one the program ships with> [--sheet \"<name>\"]   (an item that has connections already keeps them when the starter ones come in; one that is not in the job is not used)\n  plumbline shape <file> [\"<shape>\"]   (the job's own shapes; with a name, its blocks, and the items drawn as it)\n  plumbline shape <file> keep \"<item>\" \"<new name>\"   (the shape the item is drawn as is copied into the job under a new name, and the item is drawn as the copy)\n  plumbline shape <file> add \"<shape>\" [--made timber|worktop|porcelain|fabric|steel|enamel]   (a block the whole size of the item)\n  plumbline shape <file> set \"<shape>\" <block> \"<across from|across to|back from|back to|up from|up to|made of|form|there when>\" \"<value>\"   (form: box or round; there when: none for always; a figure is a formula in width, depth and height)\n  plumbline shape <file> take \"<shape>\" <block>   (a block taken away; the others are numbered again; a starter shape is never changed: keep it first)\n  plumbline import prior <file> <an estimate made before: CSV or XLSX> [--sheet \"<name>\"] [--name \"<what to call it>\"] --status <Known|Assumed> --source \"<where from>\"\n  plumbline import scan <file> <a scan: a PLY, LAS, LAZ or E57 point cloud> --level <level> [--units m|ft] [--up z|y] [--cut \"4 ft\"]   (a scan of the building, cut above its floor into a sheet to trace over: to scale, in the scan's own place, Assumed; taken as meters with z up unless said, or unless a LAS or LAZ file says its own unit, which is then said; a LAZ or E57 that cannot be read is said so and what to save it as)\n  plumbline find-walls <file> <a scan: a PLY point cloud> --level <level> [--units m|ft] [--up z|y] [--by-ceiling] [--lay [--outside \"<wall type>\"] [--take <n>] [--anyway]]   (the walls, doors and windows a scan shows, said in full and not laid; --by-ceiling finds them by the points near the ceiling, where furniture does not reach: the dense parts of those points laid on the plan, their outlines made straight, opposite faces paired into walls; --lay lays them with the model's own wall, door and window, all Assumed, sourced to the scan: two faces make a wall and give its thickness, a wall seen from one side is laid as the wall type named with --outside, and not laid without one; what stood short of the ceiling is not a wall, and what is not seen behind something is taken as wall, both said)\n  plumbline find-walls <file> <a drawing of the job, scaled> [--plan <n>] [--within \"x,y\" \"x,y\"] [--dimensions] [--scale-by <n> [--anyway]] [--lay [--take <n>] [--doors \"<head>\"] [--windows \"<sill>\" \"<head>\"]]   (the walls, doors and windows a drawn plan shows, said in full and not laid; a sheet of more than one drawing of like size (a plan and a roof plan), or where the wall found outside the largest drawing is half of it or more, says so and lays none until --plan <n> (the drawings are numbered, largest first; it takes that drawing's walls and no others) or --within (two corners, in feet on the plan; it says which walls it cuts) says which is the plan; walls within 8 ft of one another are one drawing, and a gap in a wall wider than 8 ft is an opening only where something is drawn in it, else two walls; a wall drawn with its faces twice is found once, and says how many pairs of lines were joined; lines inside a wall with a room on both sides and its faces running on past them (a shower's glass) are no window, and a tag is no opening; where the page holds its dimensions as words, --dimensions lists each string with where it is and the line it stands by, and the scale most of them give, and says which do not agree, and --scale-by <n> scales the sheet by one (Assumed, its source the string; nothing else is changed, and what was traced or laid at the scale it had is named as not moved; a scale that is Known is not replaced unless you say --anyway; a lone string is said to be alone; a string of no length, inches alone or metric is counted and not used); the drawing is a PDF with lines of its own, and must have its scale; two lines a wall's thickness apart make a wall and give its thickness, a line across ends it, evenly spaced lines (treads, boards) are not walls, a pair that meets nothing is in doubt and laid only with --take; a break in both lines is a door, or a window if lines are drawn across it inside the wall, and a pier between two openings is part of its wall however short; a wall drawn in its layers (several lines a side) is one wall, a line already a wall's face is no second wall, and a house drawn with one line a wall is told so; a wall's lines are solid, so dashed lines (a beam, cabinets above, a property line, a wall to be demolished) are never walls and are said as what they may be; where part of the sheet is a picture, or a clip or the page's edge cuts the lines off, it says so and the walls are not said to close; a scale no drawing is made at is cautioned, with the nearest; a sheet not lined up with the job says where it lays the walls; a plan gives no head or sill, so --doors and --windows say them (each above the floor, a head above its sill) and an opening they do not say is not laid, the walls then standing across it; dimension lines, lettering, the border, the title block and the key are left out and counted, and what could not be told is said with where; --lay lays them with the model's own wall, door and window, all Assumed, sourced to the sheet, and a second laying from the same sheet is refused)\n  plumbline import spec <file> <a maker's sheet: PDF> [--page <n>] [--as <equipment|fixture|furnishing>] [--name \"<what it is called>\"]   (an item to place: its name, maker, model, width, depth, height and weight read off the sheet, each Assumed and sourced to its line; what is not there is Unknown)\n  plumbline import maker-model <file> <a maker's model: IFC2x3 or IFC4> [--as <equipment|fixture|furnishing>]   (items to place, one for each kind of thing in a maker's own model, which needs no building and no level and sets nothing down: its sizes are the extent of its body, its maker and model are read off the model's manufacturer information, where it connects is read off its connection points; each Assumed and sourced to the file, what the model does not say Unknown; an item is drawn as a box of its sizes, and the body's own shape is not kept; a thing of no stated kind is left out and named unless --as says what such things are; a name the tool chest has is refused for that item and the rest come in)\n  plumbline import ifc <file> <a model: IFC2x3 or IFC4> --status <Known|Assumed> --source \"<where from>\" [--only runs]   (--only runs: of a model another trade made of the same building, only its runs and what they serve: ducts, pipes, conduit and cable, their fittings, every fixture and piece of equipment, whether a run reaches it or not (an air handler is wanted before its ducts are drawn), their systems and what is connected to what; its walls, floors, roofs, footings, doors, windows and rooms are not asked for and are counted as that; no level is added, a run on a level at a height the job does not have goes on the job's level it stands within; it says how many clashes the things brought in make with what the job has, by kind, and the job's own count of each before and after)\n  plumbline plan-set <file> <level> <out.pdf> [--scale \"1/4\"] [--paper \"ARCH A\"] [--date <the day>] [--list]   (the plan of a level as it stands, the demolition plan and the proposed plan, a page each, as line work to print: walls joined, doors and windows, room names and areas, a dimension along each wall (VIF after a length that is not Known), a title box and notes; 1/4 in to the foot unless said (1/8, 3/16, 1/4, 3/8, 1/2); the smallest of ARCH A, B, C and D it fits unless said; a level with no mark on anything gets one page, FLOOR PLAN; --list says what is on each page, wall by wall; the name ends .pdf; a plan set already there is written over and it says so, and a file there that is not a PDF is not; the job is not changed)\n  plumbline page <file> [--page <n>] [--lines [--longer-than <points>]] [--png <out.png>]   (what a page of a PDF holds as this program reads it: its size, the straight lines it draws; --png writes the page as a picture)\n  plumbline marks <a marked-up PDF> [--page <n>]   (the measured marks on a page, in the page's own order: each length, run of lengths, area and count with its label and the figure the mark itself says, and where it is; a note that measures nothing is a note; where a mark's points, at the scale it carries, work out to another figure, both are said and the mark is flagged; what cannot be read is named and why)\n  plumbline import marks <file> <a marked-up PDF> [--page <n>] [--counts]   (each measured mark a line of the estimate under the file's name: its label, its figure in LF, SF or EA, Assumed, naming the mark, the page and the file, not priced, with no division until one is set; symbols of one label are one count, which the file does not state and so comes in as a line only with --counts (the line says how many symbols of which label it counted); brought in again, a mark is known by its name in the file and is not added twice, a figure that changed moves its line, a mark gone from the file is named and its line left; what cannot be read is named and left out)\n  plumbline compare <file>\n  plumbline options <layout> <layout> [<layout>]   (two or three layouts of a job, each a job file, set side by side: each room's floor area, the walls by what the work does to them, the job cost and each division; the last less the first, and the lines each job cost leaves out; nothing is changed)\n  plumbline rename <file> \"<item>\" \"<new name>\"   (an item called something else: each of it set down is called that too, and the price list has it under the new name)\n  plumbline remove-cabinet-rule <file> <code>   (a cabinet rule taken away: a rule of the job's own goes; one of the program's own that the job changed goes back to the program's own; one the job does not carry is said to be the program's own and is left; boxes already made by it stay in the tool chest)\n  plumbline price-item <file> \"<item>\" '$180' --status <Known|Assumed|Unknown> --source \"<where from>\"   (one price on the line named for the item, on every one of it set in the job; it says how many; one set later takes none)\n\ndrawing (each but new and system takes --status <Known|Assumed> --source \"<where from>\"; points are x,y in feet):\n  plumbline new <file> \"<title>\"\n  plumbline level <file> \"<name>\" --elevation \"0 ft\" --height \"9 ft\"\n  plumbline type <file> wall \"<name>\" --thickness \"6 in\"\n  plumbline type <file> floor \"<name>\" --thickness \"4 in\"\n  plumbline type <file> <door|window> \"<name>\" --width \"3 ft\" --height \"7 ft\"\n  plumbline type <file> <duct|pipe|conduit|cable> \"<name>\" (--round \"6 in\" | --width \"12 in\" --depth \"8 in\")\n  plumbline system <file> \"<name>\" \"<Supply Air|Domestic Cold Water|...>\"\n  plumbline wall <file> <level> <type> --points \"0,0 20,0 20,10\" [--closed] (--height \"9 ft\" | --top <level>)\n  plumbline type <file> footing \"<name>\" --width \"24 in\"\n  plumbline type <file> <beam|post|pad> \"<name>\" (--round \"6 in\" | --width \"3 1/2 in\" --depth \"11 7/8 in\") [--weight \"18 lb/ft\"]   (a beam's, a post's or a pad footing's section, typed: nothing here sizes anything; --weight is a beam's, what a foot of it weighs)\n  plumbline beam <file> <level> <type> --points \"12,0 12,16\" --top \"9 ft\"   (a beam for each pair of points, its top that high above the level's floor)\n  plumbline post <file> <level> <type> --at \"12,0\" [--height \"8 ft\"] [--base \"0 ft\"]   (with no height it stands to its level's height, Assumed)\n  plumbline pad <file> <level> <type> --at \"12,0\" --thickness \"18 in\" --top \"-1 ft\"   (a pad footing: a post laid flat; its top above the level's floor, below it a minus height)\n  plumbline carries <file> [<level or wall>]   (for each wall, what is drawn standing on it and what it stands on, worked from the levels above and below and the roof; it is what is drawn and no engineer's finding; and how many walls whose bearing is not said have something standing on them)\n  plumbline structure <file>   (what carries the building, for the engineer: the walls the work opens, each beam, post and pad footing with its size, how sure that is and where it came from; and how many are to be sized)\n  plumbline footing <file> <level> <type> --points \"0,0 40,0 40,28\" [--closed] --depth \"12 in\"   (strip footings under a level's floor; the type gives the width)\n  plumbline <door|window> <file> <wall> <type> (--offset \"5 ft\" | --at \"5,0\") --sill \"0 ft\"\n  plumbline floor <file> <level> <type> (--walls | --points \"0,0 30,0 30,20 0,20\")   (--walls: the outline the level's walls enclose)\n  plumbline room <file> <level> \"<name>\" --at \"6,8\"\n  plumbline room-line <file> <level> --points \"24,0 24,16\"   (parts the rooms of a level where no wall stands: one room line for each pair of points that follow one another; it is no wall, and is not priced or counted)\n  plumbline low-slope <file> \"<name>\" --points \"0,0 48,0 48,48 0,48\" --pitch 1/4:12 --height \"11 ft\"   (a low-slope roof: its outline, the slope it finishes at, how high its deck is)\n  plumbline drains <file> <low-slope area> --at \"24,24\" --at-drain \"1/2 in\"   (its roof drains; the tapered insulation is laid out to them at the area's pitch, NRCA's four-way layout)\n  plumbline roof <file> \"<name>\" --points \"0,0 30,0 30,20 0,20\" --pitch 6:12 --eave-height \"8 ft\"   (a hip roof over its footprint: every edge rises at the pitch)\n  plumbline run <file> <level> <duct|pipe|conduit|cable> <type> --points \"2,2 14,2\" --height \"8 ft\" [--system <system>]\n  plumbline run <file> <level> <duct|pipe|conduit|cable> <type> --points \"6,5\" --height \"2 ft\" --rises-to \"9 ft\" [--system <system>]   (a riser: one upright piece, from one height up to the other; pieces end to end are one riser)\n  plumbline place <file> <level> <fixture|equipment|furnishing> \"<what>\" --at \"14,7\" --offset \"0 ft\" [--size \"2,2,6\"] [--system <system>]\n  plumbline place <file> <level> <item> --at \"x,y\" [--offset \"0 ft\"] --status <Known|Assumed> --source \"<where from>\"   (one of an item made from a maker's sheet: its size is the item's)\n  plumbline cabinets <file> <level> <wall> --in <room> --from \"1 ft\" --boxes \"B18 SB36 B24\" [--countertop] --status <Known|Assumed> --source \"<where from>\"   (a run of cabinet boxes by their codes, side by side along the wall's face on the room's side, backs to the wall, from so far along the wall from its first end; base and tall boxes stand on the floor, wall boxes (W...) hang 54 in up; each box carries one line, 1 EA, Division 12, to be priced; any item of the job goes in by its name, a name with a space or a comma in square brackets (\"B18 [Dishwasher] SB36 [Range, 30 in]\"), standing on the floor like a base box; --countertop sets one over the base boxes and what stands no taller than they do, 25 1/2 in deep; a box that would stand in another thing, or across a window, is refused; it says the run's length and what is left to the corner)\n  plumbline drawing <file> <level> \"<name>\" --file <a PDF, PNG, JPEG, TIFF or CAD drawing (DXF) of the plan> [--page 1]   (a TIFF may be of several pages: --page picks one; the measured marks of a marked-up PDF are kept with the sheet; a CAD drawing comes in at true size where it says what it is drawn in, its scale Assumed from that; its layers come with it, the ones its file has off off; what it holds that is not brought in is counted by kind; a DWG, a Revit or a SketchUp file is refused by name, with what to save it as)\n  plumbline layers <file> <drawing>   (a CAD drawing's layers, each on or off, with what it holds)\n  plumbline layer <file> <drawing> \"<layer>\" <on|off>   (a layer of a CAD drawing switched; what is off is not among the lines to snap to, and the walls finder does not see it)\n  plumbline export dxf <file> <out.dxf> --level <level>   (the plan of a level as a CAD drawing, in inches at true size: walls as drawn on plan, doors, windows, room names and areas, things set in place as outlines, runs by their centerlines, each on a layer named the way US practice names them; what is not written is said)\n  plumbline calibrate <file> <drawing> --from \"126,90\" --to \"666,90\" --distance \"30 ft\"   (two places on the page and the dimension between them)\n  plumbline lineup <file> <drawing> --page-point \"126,126\" --plan-point \"0,0\"\n  tracing: wall, floor, door, window, room, run and place take --on <drawing> in place of --status and --source; their points are then places on the drawing's page\n  plumbline mark <file> <path> [<path> ...] <new|existing|demolish|temporary> [--clear|--keep]   (what the work does to a thing; --clear takes the lines for building it off, --keep leaves them priced as work done to it; what is to demolish bounds no room and is in nothing's way; a thing with no mark is new)\n  plumbline turn <file> <fixture or equipment> --by <degrees> --status <Known|Assumed> --source \"<where from>\"   (turned that many degrees counter-clockwise from east, and further than it was; 90 is a quarter turn; what is joined to its connections is not moved)\n  plumbline move <file> <door or window> (--by \"5 ft east\" | --offset \"6 ft\" [--to <wall>]) --status <Known|Assumed> --source \"<where from>\"   (a door or a window slid along its wall, or put at a distance from the first end of its wall, or put in another wall; its middle is what is measured; refused where it would run past an end of its wall or lie over another opening, saying by how much; each wall's face less openings follows)\n  plumbline copy <file> <thing> --by \"6 ft north\" [--source \"<who copies>\"]   (a copy is Assumed whatever it was copied from, and says what it was copied from; a wall is copied with its doors and windows; a fixture, equipment, a furnishing, a run, a footing or a wall)\n  plumbline repeat <file> <thing> --times 4 --apart \"3 ft east\" [--source \"<who copies>\"]   (a row of copies, each that far from the last; all of it or none, and a refusal says which copy)\n  plumbline try <file> <wall|door|window|run|place|move|turn|copy|repeat> <as that verb takes it>   (what it would make, what it would be in the same space as, what hole it would cut, or why it would be refused; the file is not changed; any other verb that makes or takes away a thing (floor, room, footing, level, remove, ...) is tried on a copy and says what the verb says)\n  plumbline find <file> \"<words>\"   (every level, room, thing, type, item, system and line of the estimate with those words in its name: the whole name first, then names that begin with it, then names with every word)\n  plumbline move <file> <path> --by \"3,2\" [--alone]   (a wall takes with it the ends of the walls that meet it, and says which followed; --alone moves it and nothing else)\n  plumbline measure <file> <path> \"<the value, as it is listed: level height, sill, ...>\" <what was read: 8' 10 1/2\", or same> [--to <bottom|center|top>] [--line \"<cost line>\", for a value entered on it]  --source \"<who measured, or with what>\"\n  plumbline remove <file> <path>\n  plumbline fitting <file> <level> --at \"10,2\" [--height \"8 ft\"] [--kind <bend|tee|cross|transition|coupling>]\n  plumbline fitting <file> --all [--system <system>]\n  plumbline connect <file> <fixture or equipment> <run>\n  plumbline split <file> <run> --at \"6,2\"\n\nfields: pitch (6, 6:12, 1/4:12), joists_span (\"the short way\" or \"the long way\": which way a floor's joists run), type (a beam's, a post's or a pad footing's, another type of its kind), height, eave_height, on a roof drawn from its footprint edge:<n> (a pitch, or gable) and move:<n> (how far that edge moves out, like \"6 in\"), stretch_out (2.5 ft, 30 in, 19 1/2\", 25' 6 1/2\"), assembly, wbs, allowance (28, $28.00/SF), material (a material's name)\non the job: overhead (10), profit (8)\non a material: price ($4.10/lb), order (\"1.25 lb/SF\", \"30 SF/sheet\"), quote ($4.25/lb --supplier \"<name>\")\non a cost line (--line): a stretch-out and the quantity that uses it go on together (set <file> <path> quantity \"plan_length * stretch_out\" --line \"<line>\" --with \"stretch_out=19 1/2 in\"); once it is on, stretch_out alone changes it; quantity (\"plan_length * stretch_out\"), waste (10), order (\"30 SF/sheet\", \"1.25 lb/SF\"), unit_cost (4.25, $4.25/SF), or a value the formula names (spacing \"12 in\")".to_string()
}

fn read(path: &str) -> Result<(plumbline_world::Job, file::LoadReport), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    file::load(&text).map_err(|e| format!("{path}: {e}"))
}

fn run(args: &[String]) -> Result<(), String> {
    let cmd = args.first().map(String::as_str).ok_or_else(usage)?;
    // A command that takes one file takes nothing after it: an argument that would be
    // ignored is refused instead.
    if cmd == "roof" {
        return new_roof(args);
    }
    if cmd == "export" && args.get(1).map(String::as_str) == Some("dxf") {
        return cad::export(args);
    }
    if matches!(cmd, "layers" | "layer") {
        return cad::layers(args);
    }
    if matches!(cmd, "export" | "import") {
        return tables(args);
    }
    if cmd == "marks" {
        return marks::list(args);
    }
    if cmd == "plan-set" {
        return plan_set_verb(args);
    }
    if cmd == "page" {
        return page_verb(args);
    }
    if cmd == "find-walls" {
        return found::run(args);
    }
    // How tools feel (call 219): a change tried and taken back, a door or a window moved, a copy, a row of copies, a search by name.
    if matches!(cmd, "find" | "try" | "copy" | "repeat") || (cmd == "move" && feel::is_opening_move(args)) {
        return feel::run(args);
    }
    if cmd == "options" {
        let files = &args[1..];
        if files.len() < 2 || files.len() > 3 {
            return Err(format!("options sets two or three layouts side by side, each a job file; {} {} given", if files.is_empty() { "none".to_string() } else { files.len().to_string() }, if files.len() == 1 || files.is_empty() { "was" } else { "were" }));
        }
        let mut jobs = Vec::new();
        for f in files {
            jobs.push(read(f)?.0);
        }
        let mut refs: Vec<&mut plumbline_world::Job> = jobs.iter_mut().collect();
        let _ = std::io::stdout().write_all(options_said(&view::options(&mut refs), files).as_bytes());
        return Ok(());
    }
    if cmd == "price-item" {
        return price_item(args);
    }
    // An item called something else, as the window's Rename item does (call 206 point 14).
    if cmd == "rename" {
        let [_, path, item, name] = args else {
            return Err("rename takes the item and what it is to be called: plumbline rename <file> \"<item>\" \"<new name>\"".to_string());
        };
        let (mut job, _) = read(path)?;
        let item_path = item_path_of(&mut job, item)?;
        let at = job.find(&item_path).ok_or("the item is not there")?;
        let was = job.world.get::<plumbline_world::Node>(at).map(|n| n.name.clone()).unwrap_or_default();
        let done = plumbline_world::editing::rename_item_counted(&mut job, &item_path, name)?;
        job.derive();
        std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
        let placed = done.set_down;
        let mut tail = match placed {
            0 => "; none is set down".to_string(),
            1 if done.renamed == 1 => "; the 1 set down from it is called that too, and its line made of the item".to_string(),
            n if done.renamed == n => format!("; the {n} set down from it are called that too, and their lines made of the item"),
            n => format!("; {} of the {n} set down from it are called that too, and their lines made of the item", done.renamed),
        };
        // Names a person gave since are theirs, and are left (call 209 point 2).
        if !done.kept.is_empty() {
            tail.push_str(&format!("; {} keep{} the name a person gave: {}", done.kept.len(), if done.kept.len() == 1 { "s" } else { "" }, done.kept.join(", ")));
        }
        say!("{was:?} is now called {:?}{tail}", name.trim());
        return Ok(());
    }
    // A cabinet rule taken away (call 226 point 4).
    if cmd == "remove-cabinet-rule" {
        let [_, path, code] = args else {
            return Err("remove-cabinet-rule takes the letters of the rule: plumbline remove-cabinet-rule <file> X".to_string());
        };
        let (mut job, _) = read(path)?;
        let (changed, said) = plumbline_world::tables::cabinet_rule_away(&mut job, code)?;
        if changed {
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
        }
        say!("{said}");
        return Ok(());
    }
    if cmd == "compare" {
        let [_, path] = args else {
            return Err(format!("compare takes one file and nothing else\n{}", usage()));
        };
        let (mut job, _) = read(path)?;
        let _ = std::io::stdout().write_all(compared_said(&mut job).as_bytes());
        return Ok(());
    }
    if matches!(cmd, "new" | "level" | "storey" | "type" | "system" | "wall" | "footing" | "low-slope" | "drains" | "mark" | "floor" | "door" | "window" | "room" | "room-line" | "run" | "place" | "cabinets" | "beam" | "post" | "pad" | "move" | "turn" | "measure" | "remove" | "split" | "fitting" | "connect" | "drawing" | "calibrate" | "lineup") {
        return draw(args);
    }
    if matches!(cmd, "sample" | "house" | "sixteen" | "priced" | "show" | "check" | "lines" | "takeoff" | "price" | "estimate") && args.len() > 2 && !(cmd == "estimate" && args.len() == 3) && !(cmd == "show" && args.len() == 3) && !(cmd == "price" && args.len() == 3) {
        return Err(format!("{cmd} takes one file and nothing else; {:?} would be ignored\n{}", args[2], usage()));
    }
    match cmd {
        "sample" | "house" | "sixteen" | "priced" => {
            let out = args.get(1).ok_or_else(usage)?;
            // A made-up job is not written over a file, any more than a new job is
            // (J17A-16): the name typed may be the name of somebody's own job.
            if std::path::Path::new(out).exists() {
                return Err(format!("{out} is already there; a made-up job is not written over a file. Give it a name that is not in use"));
            }
            let mut job = match cmd {
                "house" => sample::house(),
                "sixteen" => sample::house_priced(),
                "priced" => sample::priced(),
                _ => sample::sample(),
            };
            std::fs::write(out, file::save(&mut job)).map_err(|e| format!("cannot write {out}: {e}"))?;
            say!("wrote {out}");
            Ok(())
        }
        "show" => {
            let path = args.get(1).ok_or_else(usage)?;
            let (mut job, report) = read(path)?;
            // Written in one piece, and a closed pipe (`| head`) is not an error.
            let text = match args.get(2) {
                None => show(&mut job, &report),
                Some(thing) => show_one(&mut job, thing)?,
            };
            let _ = std::io::stdout().write_all(text.as_bytes());
            Ok(())
        }
        // What stands on each wall and what each stands on (call 176): every wall, a level's, or one.
        "carries" => carries_verb(args),
        "check" => {
            let path = args.get(1).ok_or_else(usage)?;
            let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
            let (mut job, report) = file::load(&text).map_err(|e| format!("{path}: {e}"))?;
            if !report.stale.is_empty() {
                return Err(format!("figures in the file do not agree with what is drawn for: {}", report.stale.join(", ")));
            }
            let saved = file::save(&mut job);
            if saved != text {
                // Where: the first line the two part at, and what each has there.
                let (have, writes): (Vec<&str>, Vec<&str>) = (text.lines().collect(), saved.lines().collect());
                let n = have.iter().zip(&writes).position(|(a, b)| a != b).unwrap_or(have.len().min(writes.len()));
                let at = |lines: &[&str]| lines.get(n).map(|l| if l.trim().is_empty() { "an empty line".to_string() } else { format!("\u{201c}{}\u{201d}", l.trim()) }).unwrap_or_else(|| "nothing more".to_string());
                return Err(format!("{path} reads, but it is not what saving this job writes. They part at line {}: the file has {}; saving writes {}. Opened and saved, it is written the program's way", n + 1, at(&have), at(&writes)));
            }
            say!("{path}: reads, its figures agree with what is drawn, and saving it back writes the same file");
            // What the job holds that cannot be drawn now, and is said of one saved before (call 179, B5 and B6).
            for (_, said) in plumbline_world::building::drawn_faults(&mut job) {
                say!("! {said}");
            }
            // An item drawn as a shape that stands outside its box at its own sizes (call 206 point 2).
            for (_, said) in plumbline_world::editing::shapes::items_outside_their_boxes(&mut job) {
                say!("! {said}");
            }
            // An item whose connections stand outside its box at its own sizes (call 211).
            for (_, said) in plumbline_world::editing::connections_outside_their_boxes(&mut job) {
                say!("! {said}");
            }
            // A shape of the job's own held under a starter shape's name (call 208 point 3).
            for (_, said) in plumbline_world::editing::shapes::shapes_under_starter_names(&mut job) {
                say!("! {said}");
            }
            // A room past its floor, a wall off its footing (call 179, B1).
            for (_, said) in plumbline_world::building::reach_cautions(&mut job) {
                say!("! {said}");
            }
            // What a bearing wall that the work opens needs, and that it is not in the price (call 160).
            for said in plumbline_world::hidden::cautions(&mut job.world) {
                say!("! {}", said.1);
            }
            // What stands on a wall the work opens, and a wall said not bearing with something on it (call 176).
            for said in plumbline_world::carries::cautions(&mut job) {
                say!("! {}", said.1);
            }
            // A lump line and the things it stands for, both in the price (call 162).
            for lump in view::lump_lines(&mut job) {
                say!("! {}", lump.said());
            }
            Ok(())
        }
        "lines" => {
            let path = args.get(1).ok_or_else(usage)?;
            let (mut job, _) = read(path)?;
            let _ = std::io::stdout().write_all(lines(&mut job).as_bytes());
            Ok(())
        }
        "takeoff" => {
            let path = args.get(1).ok_or_else(usage)?;
            let (mut job, _) = read(path)?;
            let mut out = takeoff(&mut job);
            // What was traced before its sheet was rescaled or moved is counted above
            // as it lies. It is said here too, not only where the drawings are listed.
            let drawings: Vec<(String, String)> = {
                let mut q = job.world.query::<(&plumbline_world::Node, &plumbline_world::Kind)>();
                q.iter(&job.world).filter(|(_, k)| **k == plumbline_world::Kind::Drawing).map(|(n, _)| (n.path.clone(), n.name.clone())).collect()
            };
            for (path, name) in drawings {
                let left: Vec<String> = plumbline_world::plans::stale(&mut job, &path).unwrap_or_default().into_iter().flat_map(|s| s.traced).collect();
                if !left.is_empty() {
                    let _ = writeln!(out, "NOTE: {} traced over {name} before its scale or its place was changed, and not moved: {}. The figures above count {} as {} lie{}; check {} against the sheet (plumbline drawings).", count(left.len(), "thing was", "things were"), left.join(", "), if left.len() == 1 { "it" } else { "them" }, if left.len() == 1 { "it" } else { "they" }, if left.len() == 1 { "s" } else { "" }, if left.len() == 1 { "it" } else { "each" });
                }
            }
            let _ = std::io::stdout().write_all(out.as_bytes());
            Ok(())
        }
        "drawings" => {
            let path = args.get(1).ok_or_else(usage)?;
            let (mut job, _) = read(path)?;
            let _ = std::io::stdout().write_all(drawings(&mut job, path).as_bytes());
            Ok(())
        }
        "clashes" => {
            let path = args.get(1).ok_or_else(usage)?;
            let clearance = match &args[2..] {
                [] => None,
                [flag, typed] if flag == "--clearance" => {
                    let (len, unit) = units::parse_length(typed)?;
                    Some(len.feet(unit)?)
                }
                _ => return Err(usage()),
            };
            let (mut job, _) = read(path)?;
            let _ = std::io::stdout().write_all(clashes(&mut job, clearance)?.as_bytes());
            Ok(())
        }
        // What carries the building, for the engineer (call 164).
        "structure" => {
            let path = args.get(1).ok_or_else(usage)?;
            if args.len() > 2 {
                return Err(format!("structure takes one file and nothing else; {:?} would be ignored", args[2]));
            }
            let (mut job, _) = read(path)?;
            let _ = std::io::stdout().write_all(structure(&mut job).as_bytes());
            Ok(())
        }
        // The outside trim's figures: each as it is worked out from what is drawn (call 185).
        "trim" => {
            let path = args.get(1).ok_or_else(usage)?;
            let pieces = args.get(2).map(String::as_str) == Some("--pieces");
            if args.len() > if pieces { 3 } else { 2 } {
                return Err(format!("trim takes one file and nothing else; {:?} would be ignored", args[if pieces { 3 } else { 2 }]));
            }
            let (mut job, _) = read(path)?;
            let said = if pieces { trim_pieces_said(&mut job) } else { trim_said(&mut job) };
            let _ = std::io::stdout().write_all(said.as_bytes());
            Ok(())
        }
        // Each wall's gable: its area and its top (call 192).
        "gable" => {
            let path = args.get(1).ok_or_else(usage)?;
            if args.len() > 2 {
                return Err(format!("gable takes one file and nothing else; {:?} would be ignored", args[2]));
            }
            let (mut job, _) = read(path)?;
            let _ = std::io::stdout().write_all(gable_said(&mut job).as_bytes());
            Ok(())
        }
        // What is still to be verified in the field, and the dollars riding on each.
        "vif" => {
            let path = args.get(1).ok_or_else(usage)?;
            let (mut job, _) = read(path)?;
            let _ = std::io::stdout().write_all(vif(&mut job).as_bytes());
            Ok(())
        }
        // The job's items: the things that can be placed, trade by trade, each with
        // what it is drawn as, its sizes with how sure each is, and how many of it
        // are placed.
        "items" => {
            let path = args.get(1).ok_or_else(usage)?;
            let (mut job, _) = read(path)?;
            match args.get(2) {
                Some(item) => {
                    let said = item_blocks_said(&mut job, item)?;
                    let _ = std::io::stdout().write_all(said.as_bytes());
                }
                None => {
                    let _ = std::io::stdout().write_all(items_said(&mut job).as_bytes());
                }
            }
            Ok(())
        }
        // Where an item connects, changed by hand (call 211): a connection added, one thing about it set, or taken away. Each says what
        // follows: how many things set down from the item have it now, and how many runs are left standing off their connection.
        "connection" => {
            use plumbline_world::building::Facing;
            use plumbline_world::editing as ed;
            use plumbline_world::{ConnectorKind, Domain, Flow};
            let path = args.get(1).ok_or_else(usage)?;
            let item = args.get(2).ok_or_else(usage)?;
            let word = args.get(3).map(String::as_str).ok_or_else(usage)?;
            let (mut job, _) = read(path)?;
            let item_path = item_path_of(&mut job, item)?;
            let done = match word {
                "add" => {
                    let name = args.get(4).ok_or_else(|| format!("connection add needs the connection's name\n{}", usage()))?.clone();
                    let mut given: Vec<(String, String)> = Vec::new();
                    let mut rest = args[5..].iter();
                    while let Some(flag) = rest.next() {
                        if !["--takes", "--carries", "--goes", "--across", "--back", "--up", "--faces", "--size", "--status", "--source"].contains(&flag.as_str()) {
                            return Err(format!("connection add does not take {flag}\n{}", usage()));
                        }
                        if given.iter().any(|(f, _)| f == flag) {
                            return Err(format!("{flag} given twice"));
                        }
                        given.push((flag.clone(), rest.next().cloned().ok_or_else(|| format!("{flag} needs a value"))?));
                    }
                    let flag = |n: &str| given.iter().find(|(f, _)| f == n).map(|(_, v)| v.as_str());
                    let need = |n: &str, like: &str| flag(n).ok_or_else(|| format!("{n} is required, like {like}"));
                    let takes = need("--takes", "pipe (pipe, duct, conduit or power)")?;
                    let takes = [ConnectorKind::Pipe, ConnectorKind::Duct, ConnectorKind::Conduit, ConnectorKind::Power].into_iter().find(|k| plumbline_world::building::connections::kind_word(*k).eq_ignore_ascii_case(takes)).ok_or_else(|| format!("{takes:?} is not something a connection takes: pipe, duct, conduit or power"))?;
                    let carries = match flag("--carries") {
                        None => None,
                        Some(said) => Some([Domain::Cold, Domain::Hot, Domain::Waste, Domain::Vent, Domain::Storm, Domain::Gas].into_iter().find(|d| plumbline_world::building::connections::domain_word(*d).eq_ignore_ascii_case(said)).ok_or_else(|| format!("{said:?} is not something a pipe carries: cold, hot, waste, vent, storm or gas"))?),
                    };
                    let goes = match flag("--goes") {
                        None => Flow::Both,
                        Some(said) => [Flow::In, Flow::Out, Flow::Both].into_iter().find(|f| plumbline_world::building::connections::flow_word(*f).eq_ignore_ascii_case(said)).ok_or_else(|| format!("{said:?} is not a way a connection goes: in, out or both"))?,
                    };
                    // Which way it faces may be left out: a connection nobody says the facing of says nothing of it.
                    let faces = match flag("--faces") {
                        None => None,
                        Some(said) if said.eq_ignore_ascii_case("none") => None,
                        Some(said) => Some(Facing::named(said).ok_or_else(|| format!("{said:?} is not a way a connection faces: back, front, left, right, up or down, or none"))?),
                    };
                    let status = match need("--status", "Assumed")? {
                        "Known" => Status::Known,
                        "Assumed" => Status::Assumed,
                        other => return Err(format!("{other:?} is not a status a connection can be written with; use Assumed (a place is Known only where somebody measures it on the thing)")),
                    };
                    let source = need("--source", "\"a typical rough-in; the product is not chosen\"")?;
                    let row = ed::connection_row_facing(&name, takes, carries, goes, need("--across", "\"-4 in\"")?, need("--back", "\"0\"")?, need("--up", "\"height - 12 in\"")?, faces, flag("--size"), ed::Entered { status, source })?;
                    ed::connection_add(&mut job, &item_path, row)?
                }
                "set" => {
                    if args.len() < 7 {
                        return Err(format!("connection set takes the item, the connection, what to set and its value\n{}", usage()));
                    }
                    // A size says how sure it is and where from, as every value does (call 213 point 9); the other things set are words and places.
                    let (mut status, mut source) = (None, None);
                    let mut rest = args[7..].iter();
                    while let Some(flag) = rest.next() {
                        let value = rest.next().cloned().ok_or_else(|| format!("{flag} needs a value"))?;
                        match flag.as_str() {
                            "--status" => status = Some(value),
                            "--source" => source = Some(value),
                            other => return Err(format!("connection set does not take {other}\n{}", usage())),
                        }
                    }
                    if (status.is_some() || source.is_some()) && !args[5].trim().eq_ignore_ascii_case("size") {
                        return Err("--status and --source go with a connection's size: the other things set are words and places, which stay Assumed".to_string());
                    }
                    let how = match (status.as_deref(), source.as_deref()) {
                        (None, None) => None,
                        (Some(st), Some(src)) => Some(ed::Entered { status: match st { "Known" => Status::Known, "Assumed" => Status::Assumed, other => return Err(format!("{other:?} is not a status; use Assumed")) }, source: src }),
                        _ => return Err("a size says how sure it is and where from: give both --status and --source, or neither".to_string()),
                    };
                    ed::connection_set_how(&mut job, &item_path, &args[4], &args[5], &args[6], ed::Asked::CommandLine, how)?
                }
                "take" => {
                    if args.len() != 5 {
                        return Err(format!("connection take takes the item and the connection\n{}", usage()));
                    }
                    ed::connection_take(&mut job, &item_path, &args[4])?
                }
                other => return Err(format!("connection {other:?}: add, set or take\n{}", usage())),
            };
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            say!("{}", done.said());
            Ok(())
        }
        // A shape changed by hand (call 205): kept as a new shape, a block added, set, taken away. Each says what follows.
        "shape" => {
            use plumbline_world::building::forms::Made;
            use plumbline_world::editing::{self as ed, BlockSet};
            let path = args.get(1).ok_or_else(usage)?;
            let (mut job, _) = read(path)?;
            let word = args.get(2).map(String::as_str);
            let arg = |n: usize, what: &str| args.get(n).cloned().ok_or_else(|| format!("shape {} needs {what}\n{}", word.unwrap_or(""), usage()));
            let block_no = |text: &str| text.trim().parse::<usize>().map_err(|_| format!("{text:?} is not a block number: blocks are numbered from 1"));
            let done = match word {
                None => {
                    let _ = std::io::stdout().write_all(shapes_said(&mut job, None)?.as_bytes());
                    return Ok(());
                }
                Some("keep") => {
                    let item = arg(3, "the item and the new shape's name")?;
                    let item_path = item_path_of(&mut job, &item)?;
                    ed::keep_shape_as(&mut job, &item_path, &arg(4, "the new shape's name")?)?
                }
                Some("add") => {
                    let mut made = None;
                    let mut rest = args[3.min(args.len())..].iter();
                    let name = rest.next().ok_or_else(|| format!("shape add needs the shape's name\n{}", usage()))?.clone();
                    while let Some(a) = rest.next() {
                        match a.as_str() {
                            "--made" => {
                                let w = rest.next().ok_or("--made needs what it is made of")?;
                                made = Some(Made::named(w).ok_or_else(|| format!("{w:?} is not a material: {}", Made::ALL.map(Made::word).join(", ")))?);
                            }
                            other => return Err(format!("unexpected {other:?}\n{}", usage())),
                        }
                    }
                    ed::shape_block_add(&mut job, &name, made)?
                }
                Some("set") => {
                    let name = arg(3, "the shape's name")?;
                    let block = block_no(&arg(4, "the block's number")?)?;
                    let what = arg(5, "what to set")?.to_lowercase();
                    let value = arg(6, "its value")?;
                    let set = match FIGURES_SAID.iter().position(|f| *f == what) {
                        Some(n) => BlockSet::Figure(n, value),
                        None => match what.as_str() {
                            "made of" | "made" | "material" => BlockSet::Made(Made::named(&value).ok_or_else(|| format!("{value:?} is not a material: {}", Made::ALL.map(Made::word).join(", ")))?),
                            "form" => match value.trim().to_lowercase().as_str() {
                                "box" => BlockSet::Round(false),
                                "round" => BlockSet::Round(true),
                                other => return Err(format!("a block's form is box or round, not {other:?}")),
                            },
                            "there when" | "when" => BlockSet::When(if matches!(value.trim().to_lowercase().as_str(), "none" | "always" | "") { None } else { Some(value) }),
                            other => return Err(format!("{other:?} is not something a block has: {}, made of, form, there when", FIGURES_SAID.join(", "))),
                        },
                    };
                    ed::shape_block_set(&mut job, &name, block, set)?
                }
                Some("take") => {
                    let name = arg(3, "the shape's name")?;
                    ed::shape_block_take(&mut job, &name, block_no(&arg(4, "the block's number")?)?)?
                }
                Some(name) => {
                    let _ = std::io::stdout().write_all(shapes_said(&mut job, Some(name))?.as_bytes());
                    return Ok(());
                }
            };
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            say!("{}", done.said());
            Ok(())
        }
        "finishes" => {
            // What each wall, roof plane, window and door is finished with, read from
            // what is priced on it: what the model window dresses the house in.
            let path = args.get(1).ok_or_else(usage)?;
            let (mut job, _) = read(path)?;
            let lines = plumbline_world::finish::report(&mut job);
            let mut out = String::from("FINISHES: what each thing is dressed in, and what priced on it says so\n");
            if lines.is_empty() {
                out.push_str("  nothing in the job takes a finish yet\n");
            }
            for l in lines {
                out.push_str("  ");
                out.push_str(&l);
                out.push('\n');
            }
            let _ = std::io::stdout().write_all(out.as_bytes());
            Ok(())
        }
        "price" => {
            let path = args.get(1).ok_or_else(usage)?;
            let (mut job, _) = read(path)?;
            let said = match args.get(2).map(String::as_str) {
                None => price(&mut job),
                Some("list") => price_list_said(&mut job),
                Some(other) => return Err(format!("{other:?} is not a way to read the price: `plumbline price <file>` is the job's price, `plumbline price <file> list` is the price list, a row for each material and item")),
            };
            let _ = std::io::stdout().write_all(said.as_bytes());
            Ok(())
        }
        "estimate" => {
            let path = args.get(1).ok_or_else(usage)?;
            let (mut job, _) = read(path)?;
            let said = match args.get(2).map(String::as_str) {
                None => estimate(&mut job),
                Some("by-item") => estimate_by_item(&mut job),
                Some(other) => return Err(format!("{other:?} is not a way to read the estimate: `plumbline estimate <file>` is every line, `plumbline estimate <file> by-item` is each item once")),
            };
            let _ = std::io::stdout().write_all(said.as_bytes());
            Ok(())
        }
        // A material the job can be made of: its name and the unit it is bought and priced by.
        "material" => {
            let [_, path, name, unit] = args else { return Err(usage()) };
            let (mut job, _) = read(path)?;
            let made = job.add_material(name, unit)?;
            job.derive();
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            say!("added material {name:?} as {made}, priced by the {unit}; its price is not entered yet");
            Ok(())
        }
        // A cost line typed by hand onto an element (or onto the job, for general conditions).
        "line" => {
            let [_, path, element, description, quantity, rest @ ..] = args else { return Err(usage()) };
            let (mut class, mut of_material, mut division) = (None, false, None);
            let (mut status, mut source): (Option<String>, Option<String>) = (None, None);
            let mut it = rest.iter();
            while let Some(a) = it.next() {
                match a.as_str() {
                    "--status" | "--source" if (a == "--status" && status.is_some()) || (a == "--source" && source.is_some()) => return Err(format!("{a} given twice")),
                    "--status" => status = Some(it.next().cloned().ok_or("--status needs a value")?),
                    "--source" => source = Some(it.next().cloned().ok_or("--source needs a value")?),
                    "--class" | "--cost-type" => class = Some(plumbline_world::Class::parse(it.next().ok_or("--cost-type needs a cost type")?)?),
                    "--division" => division = Some(plumbline_world::Division::parse(it.next().ok_or("--division needs a division, 1 to 16")?)?),
                    "--of-material" => of_material = true,
                    other => return Err(format!("unexpected {other:?}\n{}", usage())),
                }
            }
            let (mut job, _) = read(path)?;
            // "60 SF": a measure typed outright, a value entered on the line (call 159). It
            // says how sure it is and where it came from, as every entered value does.
            // What is typed as a number and a word is read as a measure, as estimators write
            // one (1,200 SF, 60 sq ft, 24 ft); one that is not right is refused for what is
            // wrong, with the units a measure takes (call 167).
            let typed_measure = plumbline_world::units::looks_like_measure(quantity);
            let mut typed_said = quantity.trim().to_string();
            if typed_measure {
                let (x, unit) = plumbline_world::units::parse_measure_on_line(quantity)?;
                typed_said = plumbline_world::units::show_length(&plumbline_world::Len::One(x), unit);
                let status = match status.as_deref() {
                    Some("Known") => Status::Known,
                    Some("Assumed") => Status::Assumed,
                    Some("Unknown") => Status::Unknown,
                    Some(other) => return Err(format!("{other:?} is not a status; use Known, Assumed or Unknown")),
                    None => return Err(format!("{quantity:?} is a measure typed on the line, so --status is required: say whether it is Known or Assumed")),
                };
                let source = source.as_deref().ok_or_else(|| format!("{quantity:?} is a measure typed on the line, so --source is required: say where it came from"))?;
                job.add_line_measured(element, description, quantity, class, of_material, status, source)?;
            } else if status.is_some() || source.is_some() {
                return Err(format!("--status and --source go with a measure typed on the line, like \"60 SF\"; {quantity:?} is a formula or a count"));
            } else {
                job.add_line(element, description, quantity, class, of_material)?;
            }
            if let Some(division) = division {
                job.set_division(element, Some(description), division)?;
            }
            job.derive();
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            say!("added to {element}: {description:?} = {}{}{}", if typed_measure { format!("{} ({}, {})", typed_said, status.as_deref().unwrap_or_default(), source.as_deref().unwrap_or_default()) } else { quantity.to_string() }, if of_material { ", made of the element's material".to_string() } else { ", with no price yet".to_string() }, division.map(|d| format!("; {}", d.said())).unwrap_or_default());
            Ok(())
        }
        // One cost line taken off a thing, with whatever was entered on it (call 166).
        "remove-line" => {
            let [_, path, element, description] = args else {
                return Err("remove-line takes the thing and the line to take off it: plumbline remove-line <file> <thing> \"<line>\"".to_string());
            };
            let (mut job, _) = read(path)?;
            // What the line came to before it goes, as the estimate has it.
            job.derive();
            let before = view::estimate(&mut job);
            let was = before.lines.iter().find(|l| l.path == *element && l.description == *description).map(|l| (l.quantity.value, l.quantity.unit.clone(), l.amount.value));
            let line = job.remove_line(element, description);
            // What was entered on it, against its assembly's line (call 206 point 1).
            let entered = match &line {
                Ok(l) => {
                    // (the line is off the thing: put back for a moment is not needed; its assembly is on the thing still)
                    let t = job.took_of(element, l);
                    t.entered
                }
                Err(_) => Vec::new(),
            };
            line?;
            job.derive();
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            let stood = match was {
                Some((Some(q), unit, Some(amount))) => format!("{} {unit}, {}", view::num(q), view::money(amount)),
                Some((Some(q), unit, None)) => format!("{} {unit}, with no price on it", view::num(q)),
                _ => "with no figure on it".to_string(),
            };
            let entered = plumbline_world::Took { line: description.clone(), entered, amount: None }.tail();
            say!("took {description:?} off {element}: {stood}{entered}");
            Ok(())
        }
        // Which row of the recap a line lands in; and a Labor line switched off or on.
        // Which of the sixteen divisions a line's work is.
        "division" => {
            let [_, path, element, value, rest @ ..] = args else { return Err(usage()) };
            let line = match rest {
                [] => None,
                [flag, line] if flag == "--line" => Some(line.as_str()),
                _ => return Err(usage()),
            };
            let (mut job, _) = read(path)?;
            let division = plumbline_world::Division::parse(value)?;
            job.set_division(element, line, division)?;
            job.derive();
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            say!("{element}{}: {}", line.map(|l| format!(" {l:?}")).unwrap_or_default(), division.said());
            Ok(())
        }
        // A line's cost type; "class" is the word this verb had before and is still taken.
        "cost-type" | "class" | "labor" => {
            let [_, path, element, value, rest @ ..] = args else { return Err(usage()) };
            let line = match rest {
                [] => None,
                [flag, line] if flag == "--line" => Some(line.as_str()),
                _ => return Err(usage()),
            };
            let (mut job, _) = read(path)?;
            if cmd != "labor" {
                let class = plumbline_world::Class::parse(value)?;
                job.set_class(element, line, class)?;
                say!("{element}{}: cost type is {}", line.map(|l| format!(" {l:?}")).unwrap_or_default(), class.title());
            } else {
                let off = match value.as_str() {
                    "off" => true,
                    "on" => false,
                    other => return Err(format!("{other:?} is not on or off")),
                };
                job.set_off(element, line, off)?;
                say!("{element}{}: labor is {value}", line.map(|l| format!(" {l:?}")).unwrap_or_default());
            }
            job.derive();
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            Ok(())
        }
        "apply" => {
            let [_, path, element, condition, rest @ ..] = args else { return Err(usage()) };
            // --change: in place of the assembly the thing has, as the window's list does it.
            let (source, change) = match rest {
                [flag, source] if flag == "--source" => (source, false),
                [flag, source, change] | [change, flag, source] if flag == "--source" && change == "--change" => (source, true),
                _ => return Err("--source is required: say who says this thing is that assembly".into()),
            };
            let (mut job, report) = read(path)?;
            if !report.stale.is_empty() {
                say!("WARNING: before this change, the file's figures did not agree with what is drawn for: {}. They have been worked out again.", report.stale.join(", "));
            }
            let priced_before = view::estimate(&mut job);
            let changed = if change { job.change_assembly_said(element, condition, Status::Known, source)? } else { plumbline_world::Changed { added: job.apply(element, condition, source)?, ..Default::default() } };
            let plumbline_world::Changed { added, taken, took, rewritten, left } = changed;
            job.derive();
            let came_off = view::came_off_said(&priced_before, &mut job);
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            say!("{element} is now {:?} and has gained {} cost {}:", job.shown(element, "condition", None).unwrap_or_default(), added.len(), if added.len() == 1 { "line" } else { "lines" });
            // On a thing that is existing or coming down these are not lines for building
            // it, and that is said where they go on (judged 2026-11-21).
            if let Some(w) = job.find(element).and_then(|e| job.world.get::<plumbline_world::components::Work>(e)).and_then(|w| w.status.value).filter(|w| matches!(w, plumbline_world::components::WorkStatus::Existing | plumbline_world::components::WorkStatus::Demolish)) {
                say!("! {element} is {}: an assembly's lines are for building a thing new, and these are now priced on it. Take off what is not done to it", w.said());
            }
            for d in added {
                say!("  {d}");
            }
            // A line both assemblies have keeps its price, its waste, how it is ordered and whether it is off; its quantity is the
            // new assembly's where it read as the old one wrote it (call 199), and each is named, with what it was and what it is.
            if !rewritten.is_empty() {
                say!("{} it already had {} the new assembly's quantity, and kept its price, its waste and how it is ordered:", if rewritten.len() == 1 { "1 cost line".to_string() } else { format!("{} cost lines", rewritten.len()) }, if rewritten.len() == 1 { "now reads as" } else { "now read as" });
                for r in &rewritten {
                    say!("  {}: {} to {} ({} to {})", r.line, r.was, r.now, r.was_written, r.now_written);
                }
            }
            if !left.is_empty() {
                say!("{} it already had {} left as {}:", if left.len() == 1 { "1 cost line".to_string() } else { format!("{} cost lines", left.len()) }, if left.len() == 1 { "was" } else { "were" }, if left.len() == 1 { "it is" } else { "they are" });
                for l in &left {
                    match l.why {
                        plumbline_world::LeftAsIs::Entered => say!("  {}: left as entered, {} ({}); the new assembly writes {}, which comes to {}", l.line, l.has, l.has_written, l.would_written, l.would),
                        plumbline_world::LeftAsIs::OldAssemblyGone => say!("  {}: left as it was, {} ({}): the assembly it had is not in this job, so what it wrote cannot be compared with this line; the new assembly writes {}, which comes to {}", l.line, l.has, l.has_written, l.would_written, l.would),
                        plumbline_world::LeftAsIs::Unit => say!("  {}: left as it was, {} ({}): the new assembly's quantity comes out in another unit than this line is priced and ordered in, and a price does not carry from one to the other; the new assembly writes {}, which comes to {}", l.line, l.has, l.has_written, l.would_written, l.would),
                    }
                }
            }
            if !taken.is_empty() {
                say!("{} of the assembly it had came off, with whatever was priced on {}:", if taken.len() == 1 { "1 cost line".to_string() } else { format!("{} cost lines", taken.len()) }, if taken.len() == 1 { "it" } else { "them" });
                for d in taken {
                    // With what a person had entered on it, and what it came to (call 206 point 1).
                    let mut t = took.iter().find(|t| t.line == d).cloned().unwrap_or(plumbline_world::Took { line: d.clone(), ..Default::default() });
                    t.amount = priced_before.lines.iter().find(|l| l.path == *element && l.description == d && !l.off).and_then(|l| l.amount.value);
                    say!("  {d}{}", t.tail());
                }
                if let Some(said) = came_off {
                    say!("  {said}");
                }
            }
            Ok(())
        }
        "set" => {
            let [_, path, element, field, value, rest @ ..] = args else { return Err(usage()) };
            // A field is named by its words, joined or not: "unit cost" is unit_cost.
            let field = &field.trim().replace(' ', "_");
            let said = view::words(&field.to_lowercase());
            let mut status = None;
            let mut source = None;
            let mut line = None;
            let mut supplier: Option<String> = None;
            let mut with: Vec<(String, String)> = Vec::new();
            let mut it = rest.iter();
            while let Some(a) = it.next() {
                match a.as_str() {
                    "--status" | "--source" | "--line" if (a == "--status" && status.is_some()) || (a == "--source" && source.is_some()) || (a == "--line" && line.is_some()) => return Err(format!("{a} given twice")),
                    "--line" => line = Some(it.next().cloned().ok_or("--line needs the cost line's description")?),
                    "--supplier" => supplier = Some(it.next().cloned().ok_or("--supplier needs the supplier's name")?),
                    "--with" => {
                        let pair = it.next().ok_or("--with needs name=value, like --with \"turn_up=8 in\"")?;
                        let (name, typed) = pair.split_once('=').ok_or_else(|| format!("--with {pair:?} is not name=value"))?;
                        // A name is given by its words, joined or not: "turn up" is turn_up.
                        let name = name.trim().replace(' ', "_");
                        if with.iter().any(|(have, _)| *have == name) {
                            return Err(format!("--with gives {:?} twice", view::words(&name)));
                        }
                        with.push((name, typed.trim().to_string()));
                    }
                    "--status" => status = Some(it.next().cloned().ok_or("--status needs a value")?),
                    "--source" => source = Some(it.next().cloned().ok_or("--source needs a value")?),
                    other => return Err(format!("unexpected {other:?}\n{}", usage())),
                }
            }
            // An item is priced as an assembly: a name, not a value that is Known or Assumed (call 196).
            if matches!(field.to_lowercase().as_str(), "priced_as" | "assembly") {
                let (mut job, _) = read(path)?;
                let is_item = job.world.query::<(&plumbline_world::Node, &plumbline_world::Item)>().iter(&job.world).any(|(n, _)| n.path == *element || n.name.eq_ignore_ascii_case(element.trim()));
                if is_item {
                    if status.is_some() || source.is_some() || line.is_some() {
                        return Err("what an item is priced as is the name of an assembly, not a value with a status: leave --status, --source and --line off".into());
                    }
                    let followed = plumbline_world::editing::item_priced_as(&mut job, element, if value == "unknown" { None } else { Some(value.as_str()) })?;
                    std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
                    for said in followed.said() {
                        say!("{said}");
                    }
                    return Ok(());
                }
            }
            // An item's width, depth or height typed with no status is the item's size typed (contract 200P): a
            // size that differs is the item changed, Assumed; the same again is the check, Known, checked on site;
            // each thing set down from it follows. --source, where given, says who checked it.
            if matches!(field.to_lowercase().as_str(), "width" | "depth" | "height") && line.is_none() {
                let (mut job, _) = read(path)?;
                let item_path = job.world.query::<(&plumbline_world::Node, &plumbline_world::Item)>().iter(&job.world).find(|(n, _)| n.path == *element || n.name.eq_ignore_ascii_case(element.trim())).map(|(n, _)| n.path.clone());
                if let Some(item_path) = item_path {
                    use plumbline_world::editing as ed;
                    let what = field.to_lowercase();
                    // At the command line the same size typed again is no check (call 213 points 5 and 6): a value becomes Known the way every
                    // other does, with how sure and where from. Without them it is typed, Assumed, and says it was typed here.
                    let done = match status.as_deref() {
                        None => ed::item_size_asked(&mut job, &item_path, &what, &value, source.as_deref().unwrap_or(""), ed::Asked::CommandLine)?,
                        Some("Known") => ed::item_size_known(&mut job, &item_path, &what, &value, source.as_deref().ok_or("--source is required: say who measured it, or with what")?)?,
                        Some("Assumed") => ed::item_size_asked(&mut job, &item_path, &what, &value, source.as_deref().ok_or("--source is required: say where the value came from")?, ed::Asked::CommandLine)?,
                        Some(other) => return Err(format!("{other:?} is not a status an item's size can be typed with; use Known (measured, with --source) or Assumed")),
                    };
                    std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
                    say!("{}", done.said());
                    return Ok(());
                }
            }
            let status = match status.as_deref() {
                Some("Known") => Status::Known,
                Some("Assumed") => Status::Assumed,
                Some("Unknown") => Status::Unknown,
                Some(other) => return Err(format!("{other:?} is not a status; use Known, Assumed or Unknown")),
                None => return Err("--status is required: say whether the value is Known, Assumed or Unknown".into()),
            };
            let source = source.ok_or("--source is required: say where the value came from")?;
            let value = if value == "unknown" { None } else { Some(value.as_str()) };
            let (mut job, report) = read(path)?;
            if !report.stale.is_empty() {
                say!("WARNING: before this change, the file's figures did not agree with what is drawn for: {}. They have been worked out again.", report.stale.join(", "));
            }
            // A price on an item, or on a material by its name: the price list's own change, which says what it priced (call 207).
            if matches!(field.as_str(), "price" | "quote") && line.is_none() && with.is_empty() {
                let by_name = job.world.query::<(&plumbline_world::Node, &plumbline_world::Kind, Option<&plumbline_world::Item>)>().iter(&job.world).any(|(n, k, item)| {
                    let named = n.name.eq_ignore_ascii_case(element.trim()) || plumbline_world::check::same_name(&n.name, element);
                    (item.is_some() && (n.path == *element || named)) || (item.is_none() && *k == plumbline_world::Kind::Material && (n.path == *element || named))
                }) || job.find(element).is_none();
                // One road for a price, whoever sets it and however the material is named (call 208): the road a price list takes.
                if by_name {
                    if field == "quote" && supplier.is_none() {
                        return Err("a quote is a supplier's price: set <material or item> quote <price> --supplier \"<name>\"".into());
                    }
                    let done = plumbline_world::pricelist::set_price(&mut job, element, supplier.as_deref(), value, plumbline_world::editing::Entered { status, source: &source })?;
                    std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
                    for said in done.said() {
                        say!("{said}");
                    }
                    return Ok(());
                }
            }
            // A roof solved from a footprint: change the footprint and solve again.
            let on_footprint = field.starts_with("edge:") || field.starts_with("move:") || (field == "eave_height" && job.find(element).is_some_and(|e| job.world.get::<Footprint>(e).is_some()));
            if on_footprint {
                let typed = value.ok_or("mark an edge unknown in the file; a roof with an edge of no pitch cannot be solved")?;
                let number = |text: &str| text.parse::<usize>().map_err(|_| format!("{text:?} is not an edge number"));
                let change = if let Some(n) = field.strip_prefix("edge:") {
                    let rise = if typed.trim().eq_ignore_ascii_case("gable") { Val { value: Some(Rise::Gable), unit: String::new(), status, source: source.clone() } } else { Val { value: Some(Rise::Pitch(units::parse_pitch(typed)?)), unit: units::PITCH.into(), status, source: source.clone() } };
                    solve::Change::Rise { edge: number(n)?, rise }
                } else if let Some(n) = field.strip_prefix("move:") {
                    let (len, unit) = units::parse_length(typed)?;
                    solve::Change::MoveEdge { edge: number(n)?, by_ft: len.feet(unit)?, status, source: source.clone() }
                } else {
                    let (len, unit) = units::parse_length(typed)?;
                    solve::Change::EaveHeight(Val { value: Some(len), unit: unit.into(), status, source: source.clone() })
                };
                let before = file::save(&mut job);
                let priced_before = view::estimate(&mut job);
                let solved = solve::edit(&mut job, element, change)?;
                job.derive();
                let after = file::save(&mut job);
                std::fs::write(path, &after).map_err(|e| format!("cannot write {path}: {e}"))?;
                say!("set {element} {said} = {typed} ({status:?}) and solved the roof again");
                say_solve(&solved);
                // A plane that is gone took what was priced on it: said to the cent (J23-3).
                if let Some(went) = view::went_with_said(&priced_before, &mut job) {
                    say!("  {went}");
                    // And what the planes that stay did to it (call 103).
                    if let Some(whole) = view::job_cost_was_and_is_said(&priced_before, &mut job) {
                        say!("  {whole}");
                    }
                }
                if before == after {
                    say!("nothing changed");
                }
                return Ok(());
            }
            if !with.is_empty() && field != "quantity" {
                return Err("--with goes with a new quantity formula: it gives a value to a name the formula brings in".into());
            }
            if (field == "quote") != supplier.is_some() {
                return Err("a quote is a supplier's price for a material: set <material> quote <price> --supplier \"<name>\"".into());
            }
            if supplier.is_some() {
                // A quote is set by the one road, `set_price`, for a material or an item (call 208): what is left is a thing that is neither.
                return Err(format!("{element} is neither a material nor an item: a supplier quotes a material or an item"));
            }
            let units_before = job.line_units(element, line.as_deref());
            // A material's order factor reaches every line made of it: which of them
            // come to a figure now, to see that none loses it (J17A-13).
            let had: Vec<(String, String)> = if field == "order" && line.is_none() { view::estimate(&mut job).lines.iter().filter(|l| !l.off && l.amount.value.is_some()).map(|l| (l.path.clone(), l.description.clone())).collect() } else { Vec::new() };
            let dropped = if field == "quantity" {
                let formula = value.ok_or("a quantity is a formula; it cannot be marked unknown")?;
                job.set_formula(element, line.as_deref(), formula, &with, status, &source)?
            } else {
                job.set_on_line(element, field, line.as_deref(), value, status, &source)?;
                Vec::new()
            };
            let changed = job.derive();
            if !had.is_empty() {
                let now = view::estimate(&mut job);
                let lost: Vec<String> = now.lines.iter().filter(|l| l.amount.value.is_none() && had.contains(&(l.path.clone(), l.description.clone()))).map(|l| format!("{:?} on {} ({})", l.description, l.element, l.amount.note)).collect();
                if !lost.is_empty() {
                    return Err(format!(
                        "{} does not fit how this material is bought and what is made of it is measured in: with it, {} that {} to a figure now would come to none: {}. An order factor turns what a line measures into what is bought (1.25 lb/SF, 32 SF/sheet); a material bought by what its lines already measure needs none. Nothing was changed",
                        value.unwrap_or("that"),
                        if lost.len() == 1 { "1 line".to_string() } else { format!("{} lines", lost.len()) },
                        if lost.len() == 1 { "comes" } else { "come" },
                        lost.join("; ")
                    ));
                }
            }
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            // Reported back as the job now holds it, in the unit it was entered in.
            let shown = job.shown(element, field, line.as_deref()).unwrap_or_else(|| value.unwrap_or("unknown").to_string());
            if field == "quantity" {
                say!("set {element} {said} = {shown}");
                for (name, _) in &with {
                    say!("  with {name} = {} ({status:?})", job.shown(element, name, line.as_deref()).unwrap_or_default());
                }
                for name in &dropped {
                    say!("  {name} is no longer used by the formula and has been taken off the line");
                }
            } else {
                say!("set {element} {said} = {shown} ({status:?})");
                // A price of $0.00 is taken, as a thing can be free, and said (call 206 point 13).
                if matches!(field.as_str(), "unit_cost" | "price") && zero_price(value) {
                    say!("  ! that price is $0.00: it is taken, as a thing can be free; nothing is charged for it, and the estimate marks the line");
                }
                // Overhead or profit over 100 % is taken, and answered (J23-8).
                if let Some(large) = value.filter(|_| matches!(field.as_str(), "overhead" | "profit")).and_then(view::markup_large_said) {
                    say!("  {large}");
                }
            }
            // A line that now comes out in another unit, or is priced by another, says so.
            if let (Some((was, was_priced)), Some((now, now_priced))) = (units_before, job.line_units(element, line.as_deref())) {
                if was != now {
                    say!("  the line now comes out in {now}; it came out in {was}");
                }
                if was_priced != now_priced {
                    say!("  it is now priced in {now_priced}; it was priced in {was_priced}");
                }
            }
            if changed.is_empty() && !report.stale.is_empty() {
                say!("no measure changed beyond those recomputed above");
            } else if changed.is_empty() && matches!(field.as_str(), "pitch" | "eave_height") && job.find(element).is_some_and(|e| job.world.get::<plumbline_world::Solved>(e).is_some()) {
                // Put on the roof this plane is solved from, which was solved again (J23-1).
                say!("it is held on the roof this plane is solved from, which was solved again: its planes and their eaves, hips, valleys and ridges follow");
            } else if changed.is_empty() {
                say!("no measure changed");
            } else {
                say!("figures worked out again on: {}", changed.join(", "));
            }
            Ok(())
        }
        _ => Err(usage()),
    }
}

