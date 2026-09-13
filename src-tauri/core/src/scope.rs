// ── The named numbers a condition carries ──────────────────────────────────
// A solve produces geometry AND numbers a formula can use by name: the square
// feet of X-panel, the sump count, the fastener count in the corner zone. Those
// names ride the document event, because they are hundreds of bytes; the
// geometry is pulled on demand, because it is hundreds of kilobytes.
//
// C0 ships the map and fills nothing into it. That is not a placeholder for its
// own sake: the wire, the merge into a formula's scope and the collision rule
// against the reserved property names are all wired and tested now, so that C2
// — which is where the board bill arrives — adds names to a seam that already
// works instead of building the seam and the bill in one go.
//
// `None` is the whole reason the values are optional. A derived name nobody has
// solved yet reads as pending on the line, never as zero
// (`packages/engine/src/formula.ts`).

use std::collections::BTreeMap;

use crate::model::Document;

/// Per condition, the numbers it carries, by name.
pub type Scopes = BTreeMap<String, BTreeMap<String, Option<f64>>>;

/// One entry per condition in the job, each empty until a solver fills it.
///
/// Every condition appears, including the ones that will never carry a derived
/// number. A window that asks for a condition it can see and gets nothing back
/// cannot tell "this condition has no derived numbers" from "the solve did not
/// reach it", and those are different facts.
pub fn scopes_of(document: &Document) -> Scopes {
    let mut out = Scopes::new();
    for condition in &document.conditions {
        out.insert(condition.id.clone(), BTreeMap::new());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn every_condition_is_in_the_map_and_carries_nothing_yet() {
        let doc = Document::read(&json!({
            "conditions": [{ "id": "c-field", "kind": "area" }, { "id": "c-curb", "kind": "line" }],
            "pages": [],
        }))
        .expect("the document reads");
        let scopes = scopes_of(&doc);
        assert_eq!(scopes.len(), 2);
        assert!(scopes["c-field"].is_empty());
        assert!(scopes.contains_key("c-curb"));
    }
}
