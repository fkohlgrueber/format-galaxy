use lib::GalaxyFormatPlugin;
use anyhow::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    let base_path = "converters/target/wasm32-unknown-unknown/release/";
    let plugin = "test_client";
    let full_path: PathBuf = [base_path, &format!("{}{}", plugin, ".wasm")].iter().collect();
    let plugin = GalaxyFormatPlugin::new(&full_path)?;

    let args: Vec<_> = std::env::args().skip(1).collect();
    let file_name = args.get(0).expect("You need to provide a file path");
    let bytes = std::fs::read(file_name)?;
    
    match plugin.present(&bytes) {
        Ok(s) => println!("{}", s),
        Err(e) => eprintln!("{}", e),
    }
    
    Ok(())
}
