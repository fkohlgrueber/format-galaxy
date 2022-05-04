use fg_index::FormatId;
pub use fg_index::Galaxy;
use std::{io::{Read, Write}, path::Path};
pub use fg_plugin::GalaxyFormatPluginV1;
use fg_plugin::GalaxyFormatPluginV1_;

use anyhow::Result;
use wasmtime::*;

mod select;

pub use select::{
    ConverterSelection, select_plugin
};


pub struct WasmtimeGalaxyFormatPlugin {
    memory: Memory,
    store: Store<()>,
    present_fn: TypedFunc<(u32, u32), u32>,
    store_fn: TypedFunc<(u32, u32), u32>,
    alloc_fn: TypedFunc<u32, u32>,
    free_fn: TypedFunc<u32, ()>,
    result_get_ptr_fn: TypedFunc<u32, u32>,
    result_get_len_fn: TypedFunc<u32, u32>,
    result_get_success_fn: TypedFunc<u32, u32>,
}

impl GalaxyFormatPluginV1_ for WasmtimeGalaxyFormatPlugin {
    fn alloc(&mut self, size: u32) -> Result<u32> {
        Ok(self.alloc_fn.call(&mut self.store, size)?)
    }

    fn free(&mut self, ptr: u32) -> Result<()> {
        Ok(self.free_fn.call(&mut self.store, ptr)?)
    }

    fn present(&mut self, ptr: u32, size: u32) -> Result<u32> {
        Ok(self.present_fn.call(&mut self.store, (ptr, size))?)
    }
    
    fn store(&mut self, ptr: u32, size: u32) -> Result<u32> {
        Ok(self.store_fn.call(&mut self.store, (ptr, size))?)
    }
    
    fn result_get_ptr(&mut self, res_ptr: u32) -> Result<u32> {
        Ok(self.result_get_ptr_fn.call(&mut self.store, res_ptr)?)
    }
    
    fn result_get_len(&mut self, res_ptr: u32) -> Result<u32> {
        Ok(self.result_get_len_fn.call(&mut self.store, res_ptr)?)
    }
    
    fn result_get_success(&mut self, res_ptr: u32) -> Result<bool> {
        Ok(self.result_get_success_fn.call(&mut self.store, res_ptr)? > 0)
    }
    
    
    fn memory_write(&mut self, ptr: u32, bytes: &[u8]) -> Result<()> {
        self.memory.data_mut(&mut self.store)[ptr as usize .. ptr as usize + bytes.len()].clone_from_slice(bytes);
        Ok(())
    }
    
    fn memory_read(&mut self, ptr: u32, len: u32) -> Result<Vec<u8>> {
        let s_slice = &self.memory.data(&mut self.store)[ptr as usize..][..len as usize];
        //let s_slice = &self.memory.data_unchecked()[x.ptr as usize..][..x.len as usize];
        Ok(s_slice.to_vec())
    }
}

//static mut COUNTER: i32 = 0;

impl WasmtimeGalaxyFormatPlugin {

    pub fn new(path: &Path) -> Result<Self> {
        let engine = Engine::default();
        let mut store = Store::new(&engine, ());

        let bytes = std::fs::read(path)?;
        let hash = blake3::hash(&bytes);
        let module = if let Some(module) = Self::try_load_from_cache(&hash, &engine) {
            // println!("using cached module");
            module
        } else {
            // println!("cache miss. compiling module");
            let module = Module::from_file(&engine, path)?;
            // println!("caching module");
            let serialized = module.serialize()?;
            std::fs::create_dir_all("cache/compiled/")?;
            std::fs::write(format!("cache/compiled/{}", hash.to_hex()), &serialized)?;
            module
        };

        // uncomment to track allocations
        /*
        let print_alloc = Func::wrap(&store, |is_alloc: u32, ptr: u32, size: u32| {
            unsafe {
                if is_alloc>0 {
                    COUNTER += size as i32;
                } else {
                    COUNTER -= size as i32;
                }
                println!("{}: {} ({}) --- {}", if is_alloc>0 { "Alloc" } else { "Dealloc" }, ptr, size, COUNTER);
            }
        });
        */
        
        let instance = Instance::new(&mut store, &module, &[/*print_alloc.into()*/])?;
    
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or(anyhow::format_err!("failed to find `memory` export"))?;
        
        Ok(WasmtimeGalaxyFormatPlugin {
            memory, 
            present_fn: instance.get_typed_func(&mut store, "present")?,
            store_fn: instance.get_typed_func(&mut store, "store")?,
            alloc_fn: instance.get_typed_func(&mut store, "alloc")?,
            free_fn: instance.get_typed_func(&mut store, "free")?,
            result_get_ptr_fn: instance.get_typed_func(&mut store, "result_get_ptr")?,
            result_get_len_fn: instance.get_typed_func(&mut store, "result_get_len")?,
            result_get_success_fn: instance.get_typed_func(&mut store, "result_get_success")?,
            store,
        })
    }

    fn try_load_from_cache(hash: &blake3::Hash, engine: &Engine) -> Option<Module> {
        std::fs::read(format!("cache/{}", hash.to_hex()))
            .ok()
            .and_then(|serialized| unsafe { Module::deserialize(&engine, &serialized).ok() } )
    }
}

static PRELUDE: &[u8; 8] = b"FMTGALv1";

pub fn read_format_id(path: &Path) -> Result<FormatId> {
    let mut f = std::fs::File::open(path)?;
    parse_format_id(&mut f)
}

pub fn read_file(path: &Path) -> Result<(FormatId, Vec<u8>)> {
    let mut f = std::fs::File::open(path)?;
    let format_id = parse_format_id(&mut f)?;
    let mut bytes = vec!();
    f.read_to_end(&mut bytes)?;
    Ok((format_id, bytes))
}

fn parse_format_id<R: Read>(reader: &mut R) -> Result<FormatId> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    if buf != *PRELUDE {
        return Err(anyhow::anyhow!("Invalid prelude!"));
    }
    reader.read_exact(&mut buf)?;
    let format_id = FormatId(u64::from_le_bytes(buf));
    Ok(format_id)
}

pub fn write_file(path: &Path, format_id: FormatId, bytes: &[u8]) -> Result<()> {
    let mut f = std::fs::File::create(path)?;
    f.write(PRELUDE)?;
    f.write(&format_id.0.to_le_bytes())?;
    f.write(bytes)?;
    Ok(())
}

pub enum FileType {
    Ext(Option<String>),
    FormatId(FormatId),
}

pub fn get_file_type(path: &Path) -> Result<FileType> {
    let ext = file_extension(path);
    match ext {
        Some("fg") => {
            read_format_id(path).map(FileType::FormatId)
        }
        ext => Ok(FileType::Ext(ext.map(String::from)))
    }
}

pub fn is_fg_file(path: &Path) -> bool {
    matches!(file_extension(path), Some("fg"))
}

pub fn file_extension(path: &Path) -> Option<&str> {
    path.extension()
        .and_then(|s| s.to_str())
}