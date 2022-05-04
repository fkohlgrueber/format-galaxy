use yew::prelude::*;
use yew::services::storage::{Area, StorageService};
use yew::services::fetch::{FetchService, FetchTask, Request, Response};

use yew::services::reader::{File, FileChunk, FileData, ReaderService, ReaderTask};
use yew::{html, ChangeData, Component, ComponentLink, Html, ShouldRender};


use yewtil::future::LinkFuture;

use fg_index::Galaxy;
use fg_index::FormatId;
use fg_index::ConverterId;

use crate::plugin::GalaxyFormatPluginV1;
use crate::plugin::WebGalaxyFormatPlugin;

use yew::format::Nothing;

const INDEX_URL: &str = "https://raw.githubusercontent.com/fkohlgrueber/format-galaxy/main/fg-index/test_index.json";
const PLUGIN_URL: &str = "https://raw.githubusercontent.com/fkohlgrueber/format-galaxy/main/fg-index/converters/";

enum Selection {
    None,
    Format(FormatId),
    Converter(FormatId, ConverterId),
    Version(FormatId, ConverterId, String)
}

impl Selection {
    fn get_format(&self) -> Option<&FormatId> {
        match self {
            Selection::None => None,
            Selection::Format(f) => Some(f),
            Selection::Converter(f, _) => Some(f),
            Selection::Version(f, _, _) => Some(f),
        }
    }
    fn get_converter(&self) -> Option<&ConverterId> {
        match self {
            Selection::None => None,
            Selection::Format(_) => None,
            Selection::Converter(_, c) => Some(c),
            Selection::Version(_, c, _) => Some(c),
        }
    }
    fn get_version(&self) -> Option<&str> {
        match self {
            Selection::None => None,
            Selection::Format(_) => None,
            Selection::Converter(_, _) => None,
            Selection::Version(_, _, v) => Some(v.as_ref()),
        }
    }


    fn is_none(&self) -> bool {
        matches!(self, Selection::None)
    }
}

pub struct App {
    link: ComponentLink<Self>,
    galaxy: Option<Galaxy>,
    formats: Vec<(FormatId, String)>,
    ft: Option<FetchTask>,
    selection: Selection,
    plugin: Option<WebGalaxyFormatPlugin>,
    status: String,
    input_text: String,
    bytes: Option<Vec<u8>>,
    reader_task: Option<ReaderTask>,
    reader_service: ReaderService,
}

pub enum Msg {
    FormatChange(ChangeData),
    ConverterChange(ChangeData),
    VersionChange(ChangeData),
    FetchReady(String),
    PluginFetchReady(Vec<u8>),
    PluginReady(WebGalaxyFormatPlugin),
    InputChanged(String),
    Nothing,
    OpenFile,
    FormatSource,
    Download,
    OpenFileObj(File),
    FileLoaded(FileData)
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        
        let request = Request::get(INDEX_URL)
            .body(Nothing)
            .expect("Could not build that request");
        let callback = link.callback(
            move |response: Response<yew::format::Text>| {
                let (meta, data) = response.into_parts();
                if meta.status.is_success() {
                    Msg::FetchReady(data.unwrap())
                } else {
                    Msg::Nothing
                }
            },
        );
        let ft = FetchService::fetch(request, callback).unwrap();

        let app = App {
            link,
            galaxy: None,
            ft: Some(ft),
            formats: vec!(),
            selection: Selection::None,
            plugin: None,
            status: String::new(),
            input_text: "type here".to_string(),
            bytes: None,
            reader_task: None,
            reader_service: ReaderService::new(),
        };

        app
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FetchReady(s) => {
                match Galaxy::from_json_str(&s) {
                    Ok(g) => {
                        self.formats = g.formats.iter().map(|(k, v)| (k.clone(), v.name.clone())).collect::<Vec<_>>();
                        self.galaxy = Some(g);
                        
                    }
                    Err(_e) => {
                        yew::services::ConsoleService::log("Couldn't fetch galaxy index.");
                    }
                }
            }
            Msg::PluginFetchReady(bytes) => {
                self.link.send_future(async move {
                    match WebGalaxyFormatPlugin::from_slice(&bytes).await {
                        Ok(plugin) => Msg::PluginReady(plugin),
                        Err(_) => Msg::Nothing
                    }
                })
            }
            Msg::PluginReady(mut plugin) => {
                if let Some(bytes) = &mut self.bytes {
                    if let Ok(Ok(s)) = plugin.present(&bytes) {
                        self.input_text = s;
                    }
                }
                self.plugin = Some(plugin);
                self.status = "Plugin loaded, ready to go!".to_string();

                
            }
            Msg::InputChanged(s) => {
                if let Some(plugin) = &mut self.plugin {
                    match plugin.store(&s) {
                        Err(e) => {
                            self.status = format!("Fatal error: {}", e);
                            self.bytes = None;
                        },
                        Ok(Err(e)) => {
                            self.status = format!("Err: {}", e);
                            self.bytes = None;
                        },
                        Ok(Ok(bytes)) => {
                            self.status = format!("Ok");
                            self.bytes = Some(bytes);
                        },
                    }
                }
                self.input_text = s;
            }
            Msg::OpenFile => {
                
            }
            Msg::OpenFileObj(file) => {
                // TODO!
                yew::services::ConsoleService::log(&format!("Filename: {}", &file.name()));

                let callback = self.link.callback(Msg::FileLoaded);
                let task = self.reader_service.read_file(file, callback).unwrap();

                self.reader_task = Some(task);

            }
            Msg::FileLoaded(data) => {
                let bytes = data.content;
                // TODO: check that file conforms to fmtgal format, select a converter and present the content
            }
            Msg::FormatSource => {
                if let Some(bytes) = &self.bytes {
                    if let Some(plugin) = &mut self.plugin {
                        match plugin.present(bytes) {
                            Err(e) => {
                                self.status = format!("Fatal error: {}", e);
                            },
                            Ok(Err(e)) => {
                                self.status = format!("Err: {}", e);
                            },
                            Ok(Ok(s)) => {
                                //self.status = format!("Ok");
                                self.input_text = s;
                            },
                        }
                    }
                    
                }
            }
            Msg::Download => {
                // TODO: wrap self.bytes in fmtgal format and trigger download
            }
            Msg::FormatChange(cd) => {
                if let ChangeData::Select(elmt) = cd {
                    self.selection = match elmt.value().parse() {
                        Ok(fid) => Selection::Format(FormatId(fid)),
                        Err(_) => Selection::None
                    };
                }
            }
            Msg::ConverterChange(cd) => {
                if let ChangeData::Select(elmt) = cd {
                    if let Ok(id) = elmt.value().parse() {
                        let cid = ConverterId(id);
                        if let Some(fid) = self.selection.get_format().cloned() {
                            self.selection = Selection::Converter(fid, cid);
                        }
                    }
                }
            }
            Msg::VersionChange(cd) => {
                if let ChangeData::Select(elmt) = cd {
                    let version = elmt.value();
                    if let Some(fid) = self.selection.get_format().cloned() {
                        if let Some(cid) = self.selection.get_converter().cloned() {
                            self.selection = Selection::Version(fid, cid, version);

                            self.update_plugin();
                        }
                    }
                }
            }
            Msg::Nothing => {}
        }
        true
    }

    fn view(&self) -> Html {
        let formats = if let Some(g) = &self.galaxy {
            g.formats.iter().map(
                |(fid, f)| html!(<option value=fid.0 selected=self.selection.get_format()==Some(fid)>{&f.name}</option>)
            ).collect()
        } else {
            vec!()
        };
        
        let converters = match (&self.galaxy, self.selection.get_format()) {
            (Some(g), Some(fid)) => {
                g.formats.get(fid).unwrap().converters.iter().map(
                    |(cid, c)| html!(<option value=cid.0 selected=self.selection.get_converter()==Some(cid)>{&c.name}</option>)
                ).collect()
            }
            _ => vec!()
        };

        let versions = match (&self.galaxy, self.selection.get_format(), self.selection.get_converter()) {
            (Some(g), Some(fid), Some(cid)) => {
                g.formats.get(fid).unwrap().converters.get(cid).unwrap().versions.iter().map(
                    |(version, _)| html!(<option value=version selected=self.selection.get_version()==Some(version)>{&version}</option>)
                ).collect()
            }
            _ => vec!()
        };


        html! {
            <>
            <div style="padding: 10px;">
                
                <span>{"Format: "}</span><select disabled=self.galaxy.is_none() onchange=self.link.callback(|e| Msg::FormatChange(e))>
                    // display a default element when no selection has been made yet
                    {if self.selection.is_none() {
                        html!(<option value="none" selected=true>{"Select format"}</option>) 
                    } else {
                        html!()
                    }}
                    {formats}
                </select>
                <span>{"Converter: "}</span><select disabled=self.selection.get_format().is_none() onchange=self.link.callback(|e| Msg::ConverterChange(e))>
                    // display a default element when no selection has been made yet
                    {if self.selection.get_converter().is_none() {
                        html!(<option value="none" selected=true>{"Select converter"}</option>) 
                    } else {
                        html!()
                    }}
                    {converters}
                </select>
                <span>{"Version: "}</span><select  disabled=self.selection.get_converter().is_none() onchange=self.link.callback(|e| Msg::VersionChange(e))>
                    // display a default element when no selection has been made yet
                    {if self.selection.get_version().is_none() {
                        html!(<option value="none" selected=true>{"Select version"}</option>) 
                    } else {
                        html!()
                    }}
                    {versions}
                </select>
                <br />
                <button onclick=self.link.callback(|_| Msg::OpenFile)>{"Open File"}</button>
                <button disabled=self.bytes.is_none() onclick=self.link.callback(|_| Msg::FormatSource)>{"Format source"}</button>
                <button disabled=self.bytes.is_none() onclick=self.link.callback(|_| Msg::Download)>{"Download"}</button>
                <input type="file" multiple=false onchange=self.link.callback(move |value| {
                        let file = if let ChangeData::Files(files) = value {
                            files.get(0)
                        } else { None };
                        if let Some(f) = file {
                            Msg::OpenFileObj(f)
                        } else {
                            Msg::Nothing
                        }
                    })
                />
                <br />
                <textarea disabled=self.plugin.is_none() oninput=self.link.callback(|s: InputData| Msg::InputChanged(s.value)) value={&self.input_text}>{&self.input_text}</textarea>
                <textarea disabled=true value={&self.status}>{&self.status}</textarea>
            </div>
            </>
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }
}

impl App {
    fn update_plugin(&mut self) {
        // fetch plugin from index
        let hash_str = self.get_selected_plugin_hash().unwrap();

        let request = Request::get(format!("{}{}.wasm", PLUGIN_URL, hash_str))
            .body(Nothing)
            .expect("Could not build that request");
        let callback = self.link.callback(
            move |response: Response<yew::format::Binary>| {
                let (meta, data) = response.into_parts();
                if meta.status.is_success() {
                    Msg::PluginFetchReady(data.unwrap())
                } else {
                    Msg::Nothing
                }
            },
        );
        self.ft = Some(FetchService::fetch_binary(request, callback).unwrap());

    }

    fn get_selected_plugin_hash(&self) -> Option<String> {
        match &self.selection {
            Selection::Version(fid, cid, version) => {
                return Some(self.galaxy.as_ref()?.formats.get(fid)?.converters.get(cid)?.versions.iter().find(|(v, _)| v == version)?.1.0.clone());
            }
            _ => { return None; }
        }
        
    }
}