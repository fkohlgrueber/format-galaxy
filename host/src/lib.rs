use fg_index::FormatId;
pub use fg_index::Galaxy;
use std::io::{Read, Write};
pub use fg_plugin::GalaxyFormatPluginV1;
use fg_plugin::GalaxyFormatPluginV1_;

use anyhow::Result;
use wasmtime::*;

type WasmRes<T> = std::result::Result<T, wasmtime::Trap>;

pub struct WasmtimeGalaxyFormatPlugin {
    memory: Memory,
    present_fn: Box<dyn Fn(u32, u32) -> WasmRes<u32>>,
    store_fn: Box<dyn Fn(u32, u32) -> WasmRes<u32>>,
    alloc_fn: Box<dyn Fn(u32) -> WasmRes<u32>>,
    free_fn: Box<dyn Fn(u32) -> WasmRes<()>>,
    result_get_ptr_fn: Box<dyn Fn(u32) -> WasmRes<u32>>,
    result_get_len_fn: Box<dyn Fn(u32) -> WasmRes<u32>>,
    result_get_success_fn: Box<dyn Fn(u32) -> WasmRes<u32>>,
}

impl GalaxyFormatPluginV1_ for WasmtimeGalaxyFormatPlugin {
    fn alloc(&self, size: u32) -> Result<u32> {
        Ok((self.alloc_fn)(size)?)
    }

    fn free(&self, ptr: u32) -> Result<()> {
        Ok((self.free_fn)(ptr)?)
    }

    fn present(&self, ptr: u32, size: u32) -> Result<u32> {
        Ok((self.present_fn)(ptr, size)?)
    }
    
    fn store(&self, ptr: u32, size: u32) -> Result<u32> {
        Ok((self.store_fn)(ptr, size)?)
    }
    
    fn result_get_ptr(&self, res_ptr: u32) -> Result<u32> {
        Ok((self.result_get_ptr_fn)(res_ptr)?)
    }
    
    fn result_get_len(&self, res_ptr: u32) -> Result<u32> {
        Ok((self.result_get_len_fn)(res_ptr)?)
    }
    
    fn result_get_success(&self, res_ptr: u32) -> Result<bool> {
        Ok((self.result_get_success_fn)(res_ptr)? > 0)
    }
    
    
    fn memory_write(&self, ptr: u32, bytes: &[u8]) -> Result<()> {
        unsafe {
            self.memory.data_unchecked_mut()[ptr as usize .. ptr as usize + bytes.len()].clone_from_slice(bytes);
        }
        Ok(())
    }
    
    fn memory_read(&self, ptr: u32, len: u32) -> Result<Vec<u8>> {
        let bytes = unsafe {
            let s_slice = &self.memory.data_unchecked()[ptr as usize..][..len as usize];
            //let s_slice = &self.memory.data_unchecked()[x.ptr as usize..][..x.len as usize];
            s_slice.to_vec()
        };
        Ok(bytes)
    }
}

//static mut COUNTER: i32 = 0;

impl WasmtimeGalaxyFormatPlugin {

    pub fn new(path: &std::path::Path) -> Result<Self> {
        let engine = Engine::default();
        let store = Store::new(&engine);

        let bytes = std::fs::read(path)?;
        let hash = blake3::hash(&bytes);
        let module = if let Some(module) = Self::try_load_from_cache(&hash, &engine) {
            println!("using cached module");
            module
        } else {
            println!("cache miss. compiling module");
            let module = Module::from_file(&engine, path)?;
            println!("caching module");
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
        
        let instance = Instance::new(&store, &module, &[/*print_alloc.into()*/])?;
    
        let memory = instance
            .get_memory("memory")
            .ok_or(anyhow::format_err!("failed to find `memory` export"))?;
        
        let present = instance
            .get_func("present")
            .ok_or(anyhow::format_err!("failed to find `present` function export"))?
            .get2::<u32, u32, u32>()?;
        let store = instance
            .get_func("store")
            .ok_or(anyhow::format_err!("failed to find `store` function export"))?
            .get2::<u32, u32, u32>()?;
        let alloc = instance
            .get_func("alloc")
            .ok_or(anyhow::format_err!("failed to find `alloc` function export"))?
            .get1::<u32, u32>()?;
        let free = instance
            .get_func("free")
            .ok_or(anyhow::format_err!("failed to find `free` function export"))?
            .get1::<u32, ()>()?;
        let result_get_ptr = instance
            .get_func("result_get_ptr")
            .ok_or(anyhow::format_err!("failed to find `result_get_ptr` function export"))?
            .get1::<u32, u32>()?;
        let result_get_len = instance
            .get_func("result_get_len")
            .ok_or(anyhow::format_err!("failed to find `result_get_len` function export"))?
            .get1::<u32, u32>()?;
        let result_get_success = instance
            .get_func("result_get_success")
            .ok_or(anyhow::format_err!("failed to find `result_get_success` function export"))?
            .get1::<u32, u32>()?;

        Ok(WasmtimeGalaxyFormatPlugin {
            memory, 
            present_fn: Box::new(present), 
            store_fn: Box::new(store), 
            alloc_fn: Box::new(alloc), 
            free_fn: Box::new(free),
            result_get_ptr_fn: Box::new(result_get_ptr),
            result_get_len_fn: Box::new(result_get_len),
            result_get_success_fn: Box::new(result_get_success),
        })
    }

    fn try_load_from_cache(hash: &blake3::Hash, engine: &Engine) -> Option<Module> {
        std::fs::read(format!("cache/{}", hash.to_hex()))
            .ok()
            .and_then(|serialized| Module::deserialize(&engine, &serialized).ok())
    }
}

static PRELUDE: &[u8; 8] = b"FMTGALv1";

pub fn read_file(path: &std::path::Path) -> Result<(FormatId, Vec<u8>)> {
    let mut f = std::fs::File::open(path)?;
    let mut buf = [0u8; 8];
    f.read_exact(&mut buf)?;
    if buf != *PRELUDE {
        return Err(anyhow::anyhow!("Invalid prelude!"));
    }
    f.read_exact(&mut buf)?;
    let format_id = FormatId(u64::from_le_bytes(buf));
    let mut bytes = vec!();
    f.read_to_end(&mut bytes)?;
    Ok((format_id, bytes))
}

pub fn write_file(path: &std::path::Path, format_id: FormatId, bytes: &[u8]) -> Result<()> {
    let mut f = std::fs::File::open(path)?;
    f.write(PRELUDE)?;
    f.write(&format_id.0.to_le_bytes())?;
    f.write(bytes)?;
    Ok(())
}