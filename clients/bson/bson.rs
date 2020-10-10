
use format_galaxy_core::gen_plugin;
use std::convert::TryFrom;
use anyhow::anyhow;

struct Impl {}

fn present_inner(bytes: &[u8]) -> Result<String, anyhow::Error> {
    let mut cur = std::io::Cursor::new(bytes);
    let doc = bson::Document::from_reader(&mut cur)?;
    let map: serde_json::Map<String, serde_json::Value> = doc.into_iter().map(|(k, v)| (k, v.into_canonical_extjson())).collect();
    let val = serde_json::Value::from(map);
    Ok(serde_json::to_string_pretty(&val)?)
}

fn store_inner(s: &str) -> Result<Vec<u8>, anyhow::Error> {
    let val: serde_json::Value = serde_json::from_str(s)?;
    if let serde_json::Value::Object(map) = val {
        let doc: Result<bson::Document, anyhow::Error> = map.into_iter().map(|(k, v)| 
            Ok((k, bson::Bson::try_from(v)?))).collect();
        let mut writer = vec!();
        doc?.to_writer(&mut writer)?;
        return Ok(writer);
    }
    Err(anyhow!("Expected top-level object to be an object."))
}

impl format_galaxy_core::GalaxyFormat for Impl {
    fn present(bytes: &[u8]) -> Result<String, String> {
        present_inner(bytes).map_err(|e| e.to_string())
    }

    fn store(s: &str) -> Result<Vec<u8>, String> {
        store_inner(s).map_err(|e| e.to_string())
    }
}

gen_plugin!{Impl}
