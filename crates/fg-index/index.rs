
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct FormatId(pub u64);

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct ConverterId(pub u64);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConverterHash(pub String); // should be an IPFS Multihash

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Galaxy {
    pub formats: HashMap<FormatId, FileFormat>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileFormat {
    pub name: String,
    pub desc: String, // should be valid markdown
    pub converters: HashMap<ConverterId, Converter>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Converter {
    pub name: String,
    pub desc: String, // should be valid markdown
    pub versions: Vec<(String, ConverterHash)>,  // version string (e.g. "1.0.0-alpha") and hash of the wasm module
}

impl Galaxy {
    pub fn from_json(path: &std::path::Path) -> Result<Galaxy> {
        Ok(serde_json::from_slice(&std::fs::read(path)?)?)
    }

    pub fn from_json_str(s: &str) -> Result<Galaxy> {
        Ok(serde_json::from_str(s)?)
    }
}


#[test]
fn test() {
    let conv1 = Converter {
        name: "conv1".into(),
        desc: "....".into(),
        versions: vec!(
            ("0.1.0".to_string(), ConverterHash("blabla my hash".to_string())),
            ("0.1.1".to_string(), ConverterHash("blabla my hash 2".to_string())),
        )
    };
    let mut converters = HashMap::new();
    converters.insert(ConverterId(1), conv1.clone());
    converters.insert(ConverterId(2), conv1);

    let mut formats = HashMap::new();
    formats.insert(FormatId(123), FileFormat {
        name: "Foo".to_string(),
        desc: "...".to_string(),
        converters
    });
    formats.insert(FormatId(55555), FileFormat {
        name: "Bar".to_string(),
        desc: "...".to_string(),
        converters: HashMap::new()
    });
    let g = Galaxy {
        formats
    };

    let s = serde_json::to_string_pretty(&g).unwrap();
    println!("{}", s);
    //assert!(false)
}

#[test]
fn test_read() {
    let galaxy: Galaxy = serde_json::from_slice(&std::fs::read("../test_index.json").unwrap()).unwrap();

    println!("{:#?}", galaxy);
    //assert!(false);
}