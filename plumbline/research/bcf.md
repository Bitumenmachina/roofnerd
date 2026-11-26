# BCF: the issue list another program opens (call 228)

What was read, from where, and what is written. Read 2026-11-26. Plain words only were sent to the web; no page was fetched that
needed anything of this project. **Not confirmed** is said where it is.

## Sources (buildingSMART's own repository of the format)

Read by fetching the raw files of `https://github.com/buildingSMART/BCF-XML` (its official repository, "managed by the BCF
Implementers Group", as its README says):

- `release_2_1/Documentation/README.md` (the format's text for 2.1), read whole.
- `release_2_1/Schemas/version.xsd`, `markup.xsd`, `visinfo.xsd`, read whole.
- `release_3_0/Schemas/markup.xsd` (first 80 lines read), `visinfo.xsd`, `extensions.xsd`, `documents.xsd`, `project.xsd` fetched;
  3.0's own text was **not** read.
- `master/LICENSE` and `release_3_0/LICENSE`: "(c) buildingSMART International Ltd. This work is licensed under the Creative Commons
  Attribution-NoDerivatives 4.0 International License." (`http://creativecommons.org/licenses/by-nd/4.0/`). **A copy of a
  schema file is therefore not kept in this repository** (NoDerivatives: a verbatim copy might be allowed, but nothing here needs
  it, and a derivative would not be). `tools/bcf-check.py` fetches the three 2.1 schema files into a cache folder when it is run.
  The file `release_2_1/LICENSE` does not exist (404): the licence of the 2.1 branch's files was read from `master`'s and
  `release_3_0`'s, and it is **not confirmed** that it is the same text on the 2.1 branch.

## The files in the zip (2.1, the text and the schemas)

Quoted from `release_2_1/Documentation/README.md`:

- `bcf.version` at the root: "An XML file following the version.xsd schema with information of the BCF schema used." Required.
  `version.xsd`: element `Version`, attribute `VersionId`, optional child `DetailedVersion`. Written: `VersionId="2.1"`,
  `DetailedVersion` 2.1.
- `project.bcfp` at the root: optional ("An XML file referencing the extension.xsd to a project"). **Not written.**
- A folder for each topic (an issue): "The folder name is the GUID of the topic. This GUID is in the UUID form." (the schema's
  `Guid` is `[a-fA-F0-9]{8}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{12}`). In it:
  - `markup.bcf`: "An XML file following the markup.xsd schema". Written.
  - `viewpoint.bcfv`: "An XML file following the visinfo.xsd schema ... One viewpoint needs to be named viewpoint.bcfv even in
    the case of multiple viewpoints." Written, one for each issue.
  - `snapshot.png`: "A snapshot related to the topic (for compatibility with BCF 1.0)". Optional in the schema (`Snapshot`
    `minOccurs="0"`). **Not written: the program makes no picture of the place.** Said in the reply ("not carried: a picture of the
    place").
- "The forward slash (/) should be used as a path separator rather than the backslash": written so.

### `markup.bcf` (markup.xsd, 2.1): `Markup` = `Header?`, `Topic`, `Comment*`, `Viewpoints*`

Required in a Topic by the schema: `Title`, `CreationDate` (xs:dateTime), `CreationAuthor`; attribute `Guid` (required). Written:

| part | written |
|---|---|
| `Header/File@IfcProject` | the project's identifier in the BIM file (`write::guid(title, "project")`): the README says it is the "Reference to the project to which this topic is related in the IFC file". No `Filename`, no `Date`, no `Reference` (the program does not know what the model file will be called; and no path or name of a machine goes in). `isExternal` left at its default (true). |
| `Topic@Guid` | an identifier in 8-4-4-4-12 form made from the job and the clash (`write::uuid`); the same job gives the same one |
| `Topic@TopicType` | `Clash`, or `Hole to cut` (a free string in the schema) |
| `Topic@TopicStatus` | `Open` |
| `Title` | the clash list's own line: the two things' names, "in each other by 3.50 in" as the list writes it |
| `Index` | the issue's place in the list |
| `Labels` | `Finding`, `To verify in the field`, or `Hole to cut` |
| `CreationDate` | the time of writing, UTC, `2026-11-26T09:30:00Z` |
| `CreationAuthor` | `BuildOS` (the program's name; the program knows no person) |
| `Description` | plain text: what is in what, where (level, room, feet and inches), whether it is a finding or a question for the field, the dollars riding on it (questions only) |
| `Viewpoints@Guid`, `Viewpoint`, `Index` | the view's identifier, `viewpoint.bcfv`, 0 |

Not written, all optional: `Comment`, `Priority`, `ModifiedDate`/`ModifiedAuthor`, `DueDate`, `AssignedTo`, `Stage`, `BimSnippet`,
`DocumentReference`, `RelatedTopic`, `ReferenceLink`, `Snapshot`, the `Header` file's other parts.

### `viewpoint.bcfv` (visinfo.xsd, 2.1): `VisualizationInfo@Guid`

- `Components` (children in order `ViewSetupHints?`, `Selection?`, `Visibility` (**required** by the 2.1 schema), `Coloring?`).
  Written: `Selection` with one `Component` for each thing, `IfcGuid` set (22 characters, `[0-9,A-Z,a-z,_$]`, the schema's
  `IfcGuid` type), with `OriginatingSystem` (`BuildOS`) and `AuthoringToolId` (the thing's path in the job: the README calls it
  the "System specific identifier of the component in the originating BIM tool"); `Visibility DefaultVisibility="true"`.
  README on `Selection`: "lists all components that should be either highlighted or selected when displaying a viewpoint".
  **How a thing is named is by its IFC id**: README, Component: "IfcGuid | No | The IfcGuid of the component".
- `PerspectiveCamera`: `CameraViewPoint`, `CameraDirection`, `CameraUpVector` (each `X`, `Y`, `Z`) and `FieldOfView`, all required
  inside it. README: "FieldOfView represents the horizontal field of view", and "is currently restricted to a value between 45
  and 60 degrees" (the schema's `minInclusive 45`, `maxInclusive 60`). Written: 60.
- **Units**: README: "The numeric values in this file are all given in fixed units (meters for length and degrees for angle)."
  Written: metres (feet times 0.3048), degrees.
- **Axes**: the README says "world coordinates" (of the bitmap, the lines and the clipping planes) and **does not say which axes
  they are**; it gives no direction for "up". **Not confirmed from a source.** What is written is the model's world as the BIM file
  writes it (the file's own placement has no offset or turn): x east, y north, z up, as the IFC file this program writes has them.
  `CameraDirection` is a unit vector (the way looked), `CameraUpVector` a unit vector square to it, toward z.

## Which version is written, and why

**2.1.** The schema files and text for both 2.1 and 3.0 are published by buildingSMART; 3.0 adds files (`extensions.xsd`,
`documents.xsd`, a different `bcf.version`) and changes the markup (in the part of its `markup.xsd` read, `Files`, `Labels` and `ReferenceLinks` are wrapped in elements of their own), and was not read in full. For what programs open, from their makers' own pages (a search of
plain words, 2026-11-26):

- BIMCollab's own release notes and help say its BCF managers for Revit, Archicad and Navisworks "load and save 1.0, 2.0 and 2.1
  files", and that 2.1 was adopted "as the new standard across its BCF Managers".
- Procore's help says a file it imports "must comply with the BIM Collaboration Format (BCF) v2.1" (from Navisworks, Solibri or
  Revizto).
- Vertex's help says it "supports 2.1" and that a 3.0 file is opened by converting it to 2.1, dropping 3.0's features.
- **Not confirmed**: that any given program opens 3.0 (no maker's page found says Solibri, Navisworks or Revit does), and which
  version each of those programs writes today. 2.1 is what these sources show opened. Writing 3.0 as well is owed (PROPOSALS 36).

## The outside checker

`tools/bcf-check.py <file.bcfzip>`: validates `bcf.version`, every `markup.bcf` and every `.bcfv` against buildingSMART's own
2.1 `version.xsd`, `markup.xsd` and `visinfo.xsd` with lxml, and checks the layout (a folder named by the topic's Guid holds the
markup and each viewpoint the markup names). lxml is not linked into the program; the script fetches the three schema files once
(from the raw address above) into `$BCF_SCHEMAS`, else `~/.cache/bcf-2.1`. Exit 0 valid, 1 a fault, 2 "NOT CHECKED" (no lxml, or
the schemas neither in the cache nor reachable): said, never a pass. Proved red: a markup with its `CreationAuthor` removed gives
"Element 'Description': This element is not expected. Expected is ( CreationAuthor )", exit 1.

## What is not known

- The axes of BCF's world, above.
- Whether other programs match the issue's things by `IfcGuid` alone when the file the issue is opened against has no `Filename`
  in the header (the README says `Filename` is optional; how each program finds the model is not in the sources read).
- Whether any program shows the view as intended with a 60 degree horizontal field and no snapshot.
- Which version 3.0's markup requires in full (not read).
