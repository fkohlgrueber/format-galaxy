/* Interactive selection of converters on the command line
*/

use fg_index::Converter;
use fg_index::ConverterId;
use fg_index::FileFormat;
use fg_index::FormatId;
use fg_index::Galaxy;
use terminal_menu::{menu, label, button, run, mut_menu};

use super::FileType;

enum Answer<T> {
    Selected(T),
    Back,
    Exit,
}

pub struct ConverterSelection {
    pub format_id: FormatId,
    pub converter_id: ConverterId,
    pub version_idx: usize,
}

fn ask(formats: &[(FormatId, FileFormat)], allow_format_selection: bool) -> Option<ConverterSelection> {
    let mut format_state = None;
    let mut converter_state = None;

    if !allow_format_selection {
        assert_eq!(formats.len(), 1);
        format_state = Some(&formats[0]);
    }

    loop {
        match (&format_state, &converter_state) {
            (None, None) => {
                // ask for format
                match ask_format(formats) {
                    Answer::Selected(format) => {
                        format_state = Some(format);
                    }
                    Answer::Exit => return None,
                    Answer::Back => unreachable!()
                }
            }
            (Some((_format_id, format)), None) => {
                // ask for converter
                let mut converters: Vec<_> = format.converters.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                converters.sort_by_key(|c| c.1.name.to_string());
                match ask_converter(converters.as_slice(), allow_format_selection, &format.name) {
                    Answer::Selected(converter) => {
                        converter_state = Some(converter.clone());
                    }
                    Answer::Exit => return None,
                    Answer::Back => {
                        format_state = None;
                    }
                }
            }
            (Some(format), Some(converter)) => {
                // ask for version
                let versions: Vec<&String> = converter.1.versions.iter().map(|x| &x.0).collect();
                match ask_version(versions.as_slice()) {
                    Answer::Selected(version_idx) => {
                        return Some(ConverterSelection {
                            format_id: format.0.clone(), 
                            converter_id: converter.0.clone(), 
                            version_idx
                        });
                    }
                    Answer::Exit => return None,
                    Answer::Back => {
                        converter_state = None;
                    }
                }
            }
            _ => unreachable!()
        }
    }

}

fn ask_format(formats: &[(FormatId, FileFormat)]) -> Answer<&(FormatId, FileFormat)> {
    let mut items = vec!(
        label("Please select a format:"),
    );
    let num_labels = items.len();

    items.extend(formats.iter().map(|x| button(&x.1.name)));
    
    items.push(button("(exit)"));
    
    let menu = menu(items);
    run(&menu);
    if mut_menu(&menu).canceled() {
        return Answer::Exit;
    }
    let idx = mut_menu(&menu).selected_item_index() - num_labels;
    
    if idx < formats.len() {
        let format_id = formats.iter().nth(idx).unwrap();
        return Answer::Selected(format_id);
    }

    if idx == formats.len() {
        return Answer::Exit;
    }

    unreachable!("This is an internal error");
}

fn ask_converter<'a>(converters: &'a[(ConverterId, Converter)], offer_back: bool, format_name: &str) -> Answer<&'a (ConverterId, Converter)> {
    let mut items = vec!(
        label(&format!("File format: {}", format_name)),
        label("Please select a converter:"),
    );
    let num_labels = items.len();

    items.extend(converters.iter().map(|x| button(&x.1.name)));

    if offer_back {
        items.push(button("(back)"));
    }
    items.push(button("(exit)"));
    
    
    let menu = menu(items);
    
    run(&menu);
    if mut_menu(&menu).canceled() {
        return Answer::Exit;
    }
    let idx = mut_menu(&menu).selected_item_index() - num_labels;
    
    if idx < converters.len() {
        let converter_id = converters.iter().nth(idx).unwrap();
        return Answer::Selected(converter_id);
    }

    if offer_back {
        if idx == converters.len() {
            return Answer::Back;
        } else if idx == converters.len() + 1 {
            return Answer::Exit;
        }
    } else if idx == converters.len() {
        return Answer::Exit;
    }

    unreachable!("This is an internal error");
}

fn ask_version(versions: &[&String]) -> Answer<usize> {
    let mut items = vec!(
        label("Please select a version:"),
    );
    let num_labels = items.len();

    // versions should be shown newest to oldest, so reverse here
    items.extend(versions.iter().rev().map(|x| button(x.as_str())));

    
    items.push(button("(back)"));
    items.push(button("(exit)"));
    
    
    let menu = menu(items);
    
    run(&menu);
    if mut_menu(&menu).canceled() {
        return Answer::Exit;
    }
    let idx = mut_menu(&menu).selected_item_index() - num_labels;
    
    if idx < versions.len() {
        return Answer::Selected(versions.len()-1-idx);  // calculation neede because of reverse display of versions
    } else if idx == versions.len() {
        return Answer::Back;
    } else if idx == versions.len() + 1 {
        return Answer::Exit;
    }

    unreachable!("This is an internal error");
}


pub fn select_plugin(galaxy: &Galaxy, file_type: &FileType) -> Option<ConverterSelection> {
    // ask user to select a converter plugin
    match file_type {
        FileType::FormatId(fid) => {
            let formats = vec!((fid.clone(), galaxy.formats[&fid].clone()));
            ask(formats.as_slice(), false)
        }
        FileType::Ext(opt_ext) => {
            // only list plugins that are compatible with the given file extension
            let mut formats: Vec<(FormatId, FileFormat)> = if let Some(ext) = opt_ext {
                let filtered_formats: Vec<_> = galaxy.formats.iter()
                    .filter(|(_id, ff)| ff.extensions.iter().any(|s| s==ext))
                    .map(|(id, format)| (id.clone(), format.clone())).collect();
                
                if filtered_formats.is_empty() {
                    println!("No matching format found for file with extension \'{}\'", ext);
                    galaxy.formats.iter().map(|(id, format)| (id.clone(), format.clone())).collect()
                } else {
                    filtered_formats
                }
            } else {
                galaxy.formats.iter().map(|(id, format)| (id.clone(), format.clone())).collect()
            };
        
            formats.sort_by_key(|x| x.1.name.to_string());
        
            // ask user
            ask(formats.as_slice(), true)
        }
    }
}