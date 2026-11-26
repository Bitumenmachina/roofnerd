# glTF 2.0, from the format's own published text (call 229)

Read 2026-11-26 from the Khronos Group's published specification source
(`https://raw.githubusercontent.com/KhronosGroup/glTF/main/specification/2.0/Specification.adoc`,
text licensed CC-BY-4.0 by its header; the page it renders to is
`https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html`) and its JSON schemas
(`.../specification/2.0/schema/mesh.primitive.schema.json`, `bufferView.schema.json`).
Short quotations only. Nothing here comes from memory except where it says so.

## Measure, axes, handedness

- "glTF uses a right-handed coordinate system."
- "glTF defines +Y as up; the front side of a glTF asset faces +Z, the left side of a glTF asset faces +X."
- "The units for all linear distances are meters."
- "After applying the node's global transform, mesh vertex position values are meters."

So a job's place (x east, y north, z up, feet, itself right-handed) becomes
(X, Y, Z) = (x, z, -y) metres: a proper turn of the axes (determinant +1), never a
mirror. North is -Z, so the asset's "front" (+Z) is the house's south face.

## The one-file form (GLB)

- Header: "uint32 magic, uint32 version"; "`magic` MUST be equal to equal 0x46546C67 (ASCII glTF)"; version 2.
- "`length` is the total byte length of the Binary glTF, including the GLB header, the size header, and all chunks." (12 bytes of header: magic, version, length.)
- A chunk is "uint32 chunkLength, uint32 chunkType, uint8[] chunkData"; "The `chunkLength` value MUST be a multiple of four."
- Types: JSON `0x4E4F534A`, BIN `0x004E4942`. "GLBv2 assets MUST have exactly one chunk of this type [JSON] and it MUST be the first chunk"; "at most one chunk of this type [BIN] and it MUST be the second chunk".
- JSON padding: "can be padded by any valid whitespace JSON characters, such as space". BIN padding: "can be padded by any bytes as necessary. Note that the byte length of the corresponding glTF buffer can be less than the byte length of the chunk."
- "a glTF buffer that has index zero ... and has its `uri` property undefined represents the GLB-stored buffer. The GLB file MUST have a `BIN` chunk in this case."

## How points and faces are laid out and described

- Buffers, buffer views (`buffer`, `byteOffset`, `byteLength`, optional `byteStride`, optional `target`; 34962 vertex array, 34963 element array), accessors (`bufferView`, `byteOffset`, `componentType`, `count`, `type`, `min`, `max`).
- Component types used: 5126 float32, 5125 uint32, 5123 uint16 (table at "Accessor Data Types").
- "If the accessor is used for vertex attributes, its elements MUST be aligned to 4-byte boundaries inside the buffer view."
- "Accessors for the `POSITION` semantic MUST have their bounds, i.e., `min` and `max` properties, defined." "The length of these arrays MUST be equal to the number of accessor's components." For floats the bounds "SHOULD exactly match" what the accessor gives (after rounding to float32).
- `mode` of a primitive: 4 Triangle List (the default), 1 Line List (schema). "The number of the primitive's vertices MUST be divisible by three" for triangle lists.
- Indices: a primitive's `indices` accessor gives the vertex numbers; positions are then taken by them.
- "When normals are not specified, the mesh primitive has implicit flat normals." (This program gives none: a plain flat-faced model.)
- Winding: "if the determinant is a positive value, the front-facing triangles use the counterclockwise order".
- "The node hierarchy MUST be a set of disjoint strict trees."
- Names: nodes, meshes and materials may carry `name`; `extras` is the place for anything of ours (the spec says tools put names of morph targets there; it is application data).

## What a colour is

- A material's `pbrMetallicRoughness` has `baseColorFactor`, `metallicFactor`, `roughnessFactor`; "The factor value is a linear multiplier for the corresponding texture components." The spec's sRGB passage speaks of textures: "The red, green, and blue components of the texture MUST be encoded with the piecewise sRGB opto-electronic transfer function ... apply the inverse of that function to get linear base color texture values before multiplying them with base color factors". So the factor is a linear value: this program works its plain colours in the 8-bit way trades read them (sRGB) and writes the linear value of each.
- `alphaMode`: "OPAQUE", "MASK", "BLEND" ("combined with the background using the 'over' operator"). `doubleSided` says whether back faces are drawn.
- Lines: "Points or Lines with no `NORMAL` attribute SHOULD be rendered without lighting and instead use the sum of the base color value ... and the emissive value."

## Not confirmed from the text, and so not relied on

- The numeric value of a mode (1 and 4) is from the schema file, not the prose specification (both are Khronos's own).
- How a given viewer draws a BLEND part in front of another (order is the viewer's) is not in the text.
- The size limit a viewer takes is not in the text; this program keeps vertex numbers as uint32 and does not split a file.

## The format's own checker

The Khronos glTF Validator (`gltf-validator`, Apache-2.0, published on npm at
`https://registry.npmjs.org/gltf-validator`, version 2.0.0-dev.3.10 when read). It is run
as an outside program only (`tools/gltf-check.sh`), never linked, never committed.
