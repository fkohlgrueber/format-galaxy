use lib::GalaxyFormatPlugin;
use anyhow::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("Loading Galaxy...");
    let galaxy = lib::Galaxy::from_json(std::path::Path::new("test_index.json"))?;

    let args: Vec<_> = std::env::args().skip(1).collect();
    let file_name = args.get(0).expect("You need to provide a file path");
    
    println!("Loading file...");
    // read file
    let (format_id, bytes) = lib::read_file(&std::path::PathBuf::from(file_name))?;
    println!("Format_id: {:x}", format_id.0);

    let format = galaxy.formats.get(&format_id).unwrap();

    // TODO: selection of converters
    let converter = format.converters.iter().next().unwrap().1;
    let converter_hash = &converter.versions.first().unwrap().1.0;

    println!("Loading Plugin...");
    let base_path = "cache/plugins/";
    let full_path: PathBuf = [base_path, &format!("{}{}", converter_hash, ".wasm")].iter().collect();
    let plugin = GalaxyFormatPlugin::new(&full_path)?;

    match plugin.present(&bytes) {
        Ok(s) => println!("{}", s),
        Err(e) => eprintln!("{}", e),
    }
    
    Ok(())
}
