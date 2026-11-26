//! Makes the packed (LAZ) and the exchange (E57) copies of a made-up room for `demos/scan-files.sh` (call 230). An example, not
//! a command: it is built only by `cargo build --example made-up-scan-files`, and is no part of the program a person is given.
//!
//!   made-up-scan-files <room.las> <folder>
//!
//! Reads a LAS file of point record format 0 or 1 that the demonstration's own script wrote (a made-up room, no real
//! scan) and writes `room.laz` (the same records, packed with the laz crate) and `room.e57` (the same points, in
//! meters, with the e57 crate) into the folder. The files are only ever read back by the program's own readers, which
//! are not these crates' writers' only users: that is said in STATUS.md, since no scanner's own LAZ or E57 is to hand.

use std::io::{Cursor, Seek, SeekFrom};

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let [_, las, folder] = args.as_slice() else { return Err("made-up-scan-files <room.las> <folder>".into()) };
    let bytes = std::fs::read(las).map_err(|e| format!("{las}: {e}"))?;
    let word = |at: usize, n: usize| bytes[at..at + n].to_vec();
    let header = u16::from_le_bytes(word(94, 2).try_into().unwrap()) as usize;
    let start = u32::from_le_bytes(word(96, 4).try_into().unwrap()) as usize;
    let format = bytes[104];
    let len = u16::from_le_bytes(word(105, 2).try_into().unwrap()) as usize;
    let count = u32::from_le_bytes(word(107, 4).try_into().unwrap()) as usize;
    let f = |at: usize| f64::from_le_bytes(word(at, 8).try_into().unwrap());
    let (scale, offset) = ([f(131), f(139), f(147)], [f(155), f(163), f(171)]);

    // LAZ: the header with the packed mark, one "laszip encoded" record, then the points packed in chunks.
    let items = laz::LazItemRecordBuilder::default_for_point_format_id(format, 0).map_err(|e| e.to_string())?;
    let vlr = laz::LazVlrBuilder::new(items).build();
    let mut data = Vec::new();
    vlr.write_to(&mut data).map_err(|e| e.to_string())?;
    let mut file = bytes[..header].to_vec();
    file[104] = format | 0x80;
    file[100..104].copy_from_slice(&1u32.to_le_bytes());
    let mut record = vec![0u8; 54];
    record[2..16].copy_from_slice(b"laszip encoded");
    record[18..20].copy_from_slice(&22204u16.to_le_bytes());
    record[20..22].copy_from_slice(&(data.len() as u16).to_le_bytes());
    file.extend_from_slice(&record);
    file.extend_from_slice(&data);
    let at = file.len() as u32;
    file[96..100].copy_from_slice(&at.to_le_bytes());
    let mut out = Cursor::new(file);
    out.seek(SeekFrom::End(0)).map_err(|e| e.to_string())?;
    let mut packer = laz::LasZipCompressor::new(&mut out, vlr).map_err(|e| e.to_string())?;
    packer.compress_many(&bytes[start..start + count * len]).map_err(|e| e.to_string())?;
    packer.done().map_err(|e| e.to_string())?;
    drop(packer);
    std::fs::write(format!("{folder}/room.laz"), out.into_inner()).map_err(|e| e.to_string())?;

    // E57: the same points, in meters.
    let mut buf = Cursor::new(Vec::new());
    {
        use e57::{E57Writer, Record, RecordDataType, RecordName, RecordValue};
        let mut w = E57Writer::new(&mut buf, "made-up-room").map_err(|e| e.to_string())?;
        let axis = |name| Record { name, data_type: RecordDataType::Double { min: None, max: None } };
        let mut cloud = w.add_pointcloud("made-up-room-scan", vec![axis(RecordName::CartesianX), axis(RecordName::CartesianY), axis(RecordName::CartesianZ)]).map_err(|e| e.to_string())?;
        for r in bytes[start..start + count * len].chunks_exact(len) {
            let at = |k: usize| f64::from(i32::from_le_bytes(r[4 * k..4 * k + 4].try_into().unwrap())) * scale[k] + offset[k];
            cloud.add_point(vec![RecordValue::Double(at(0)), RecordValue::Double(at(1)), RecordValue::Double(at(2))]).map_err(|e| e.to_string())?;
        }
        cloud.finalize().map_err(|e| e.to_string())?;
        w.finalize().map_err(|e| e.to_string())?;
    }
    std::fs::write(format!("{folder}/room.e57"), buf.into_inner()).map_err(|e| e.to_string())?;
    Ok(())
}
