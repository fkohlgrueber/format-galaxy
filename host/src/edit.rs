use lib::GalaxyFormatPlugin;
use anyhow::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("loading...");
    let base_path = "target/wasm32-unknown-unknown/release/";
    let plugin = "wasm";
    let full_path: PathBuf = [base_path, &format!("{}{}", plugin, ".wasm")].iter().collect();
    let plugin = GalaxyFormatPlugin::new(&full_path)?;

    let args: Vec<_> = std::env::args().skip(1).collect();
    let file_name = args.get(0).expect("You need to provide a file path");
    
    let tmp_filename = format!("{}{}", file_name, ".tmp");
    if let Ok(bytes) = std::fs::read(file_name) {
        // present
        let s = match plugin.present(&bytes) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}", e);
                return Ok(());
            },
        };

        // write to tmp file
        std::fs::write(&tmp_filename, s)?;

    }
    
    // call editor, wait 'till it completes
    println!("open editor");
    let _ = std::process::Command::new("vim")
        .arg(&tmp_filename)
        .status()
        .expect("Failed to execute command");

    // read tmp file
    let s = std::fs::read_to_string(&tmp_filename)?;

    // store
    let bytes = match plugin.store(&s) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{}", e);
            return Ok(());
        },
    };
    std::fs::write(&file_name, bytes)?;

    // delete tmp file
    std::fs::remove_file(tmp_filename)?;
    
    Ok(())
}
