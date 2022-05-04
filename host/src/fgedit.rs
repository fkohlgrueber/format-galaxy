use lib::FileType;
use lib::WasmtimeGalaxyFormatPlugin;
use lib::GalaxyFormatPluginV1;
use anyhow::Result;
use lib::file_extension;
use lib::is_fg_file;
use std::path::PathBuf;

fn download_index() -> Result<lib::Galaxy> {
    let path = std::path::Path::new("fg-index/test_index.json");
    let galaxy = lib::Galaxy::from_json(path)?;
    Ok(galaxy)
}

fn ask_yes_no(prompt: &str) -> bool {
    loop {
        println!("{}", prompt);

        let mut buf = String::new();
        let _ = std::io::stdin().read_line(&mut buf).unwrap();
        match buf.as_str().trim() {
            "y" | "yes" => {
                return true;
            }
            "n" | "no" => {
                return false;
            }
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    
    let galaxy = download_index()?;

    // parse file name from command line
    let args: Vec<_> = std::env::args().skip(1).collect();
    let file_name = args.get(0).expect("You need to provide a file path");
    let file_path = PathBuf::from(file_name);
    
    let tmp_filename = format!("{}{}", file_name, ".tmp");

    let (file_type, store_in_container_format) = if file_path.is_file() {
        // check file type and whether it contains format_id
        let file_type = lib::get_file_type(&file_path)?;
        // print warning when using file that doesn't use the fmtgal container format
        if let FileType::Ext(_) = &file_type {
            eprintln!("WARNING: The file doesn't use the format galaxy container format. The exact format of the file is not known and needs to be selected manually.")
        }
        let store_in_container_format = matches!(&file_type, FileType::FormatId(_));
        (file_type, store_in_container_format)
    } else {
        // if file doesn't exist, offer all file formats and ask whether to store in the container format on save
        if is_fg_file(&file_path) {
            (FileType::Ext(None), true)
        } else {
            (FileType::Ext(file_extension(&file_path).map(String::from)), false)
        }
    };


    // ask user to select a converter
    let selection = match lib::select_plugin(&galaxy, &file_type) {
        Some(x) => x,
        None => {
            return Ok(()); // selection was cancelled by user
        },
    };

    let format = &galaxy.formats[&selection.format_id];
    let converter = &format.converters[&selection.converter_id];
    let converter_version = &converter.versions[selection.version_idx];
    let converter_hash = converter_version.1.0.as_str();

    // load plugin
    // println!("Loading Plugin...");
    let base_path = "fg-index/converters/";
    let full_path: PathBuf = [base_path, &format!("{}{}", converter_hash, ".wasm")].iter().collect();
    let mut plugin = WasmtimeGalaxyFormatPlugin::new(&full_path)?;


    // convert existing file
    if file_path.is_file() {
        // read file content
        let content_bytes = match file_type {
            FileType::Ext(_) => {
                std::fs::read(&file_path)?
            }
            FileType::FormatId(_) => {
                let (format_id, bytes) = lib::read_file(&std::path::PathBuf::from(file_name))?;
                assert_eq!(format_id, selection.format_id);
                bytes
            }
        };

        // present
        let s = match plugin.present(&content_bytes)? {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}", e);
                return Ok(());
            },
        };

        // write to tmp file
        std::fs::write(&tmp_filename, s)?;
    }

    let bytes = loop {
        // call editor, wait 'till it completes
        let _ = std::process::Command::new("vim")
            .arg(&tmp_filename)
            .status()
            .expect("Failed to execute command");

        // read tmp file
        let s = std::fs::read_to_string(&tmp_filename).expect("Couldn't read tmp file");

        // store
        match plugin.store(&s)? {
            Ok(b) => {
                break b;
            }
            Err(e) => {
                eprintln!("Storing the content yielded the following error:\n");
                eprintln!("{}\n", e);
                let res = ask_yes_no("Do you want to open the editor again? [y/n]");
                if !res {
                    return Ok(());
                }
            },
        }
    };


    if store_in_container_format {
        lib::write_file(&file_path, selection.format_id.clone(), &bytes).expect("Couldn't write result file");
    } else {
        std::fs::write(&file_name, bytes).expect("Couldn't write result file (non-container)");
    }

    // delete tmp file
    std::fs::remove_file(tmp_filename).expect("Couldn't delete tmp file");
    
    Ok(())
}
