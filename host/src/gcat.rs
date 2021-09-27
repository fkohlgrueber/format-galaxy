use lib::WasmtimeGalaxyFormatPlugin;
use lib::GalaxyFormatPluginV1;
use anyhow::Result;
use std::path::PathBuf;

use lib::{
    FileType
};

fn download_index() -> Result<lib::Galaxy> {
    let path = std::path::Path::new("fg-index/test_index.json");
    let galaxy = lib::Galaxy::from_json(path)?;
    Ok(galaxy)
}

fn main() -> Result<()> {
    
    let galaxy = download_index()?;

    // parse file name from command line
    let args: Vec<_> = std::env::args().skip(1).collect();
    let file_name = args.get(0).expect("You need to provide a file path");
    let file_path = PathBuf::from(file_name);
    if !file_path.is_file() {
        eprintln!("File not found");
        return Ok(());
    }

    // check file type and whether it contains format_id
    let file_type = lib::get_file_type(&file_path);

    // ask user to select a converter
    let selection = match lib::select_plugin(&galaxy, &file_type) {
        Some(x) => x,
        None => {
            return Ok(()); // selection was cancelled by user
        },
    };
    
    // read file content
    // println!("Loading file...");
    let content_bytes = match file_type {
        FileType::Ext(_) => {
            std::fs::read(file_path)?
        }
        FileType::FormatId(_) => {
            let (format_id, bytes) = lib::read_file(&std::path::PathBuf::from(file_name))?;
            assert_eq!(format_id, selection.format_id);
            bytes
        }
    };
    
    let format = &galaxy.formats[&selection.format_id];
    let converter = &format.converters[&selection.converter_id];
    let converter_version = &converter.versions[selection.version_idx];
    let converter_hash = converter_version.1.0.as_str();

    // load plugin
    // println!("Loading Plugin...");
    let base_path = "cache/plugins/";
    let full_path: PathBuf = [base_path, &format!("{}{}", converter_hash, ".wasm")].iter().collect();
    let plugin = WasmtimeGalaxyFormatPlugin::new(&full_path)?;

    // use plugin to present the content
    match plugin.present(&content_bytes)? {
        Ok(s) => println!("{}", s),
        Err(e) => eprintln!("{}", e),
    }
    
    Ok(())
}
