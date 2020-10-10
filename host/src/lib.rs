use format_galaxy_core::ReturnData;

use anyhow::Result;
use wasmtime::*;

type WasmRes<T> = std::result::Result<T, wasmtime::Trap>;

pub struct GalaxyFormatPlugin {
    memory: Memory,
    present: Box<dyn Fn(u32, u32) -> WasmRes<u32>>,
    store: Box<dyn Fn(u32, u32) -> WasmRes<u32>>,
    alloc: Box<dyn Fn(u32) -> WasmRes<u32>>,
    free: Box<dyn Fn(u32) -> WasmRes<()>>,
}

impl GalaxyFormatPlugin {

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
            std::fs::write(format!("cache/{}", hash.to_hex()), &serialized)?;
            module
        };
        
        let instance = Instance::new(&store, &module, &[])?;
    
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

        Ok(GalaxyFormatPlugin {
            memory, 
            present: Box::new(present), 
            store: Box::new(store), 
            alloc: Box::new(alloc), 
            free: Box::new(free),
        })
    }

    pub fn present(&self, bytes: &[u8]) -> Result<String, String> {
        self.handle_call(bytes, &self.present).map(|x| String::from_utf8(x).unwrap())
    }

    pub fn store(&self, s: &str) -> Result<Vec<u8>, String> {
        self.handle_call(s.as_bytes(), &self.store)
    }

    fn try_load_from_cache(hash: &blake3::Hash, engine: &Engine) -> Option<Module> {
        std::fs::read(format!("cache/{}", hash.to_hex()))
            .ok()
            .and_then(|serialized| Module::deserialize(&engine, &serialized).ok())
    }

    fn handle_call<T: Fn(u32, u32) -> WasmRes<u32>>(&self, bytes: &[u8], f: &T) -> Result<Vec<u8>, String> {
        // allocate memory and store bytes
        let len =bytes.len();
        let ptr = (self.alloc)(len as u32).unwrap();
        unsafe {
            self.memory.data_unchecked_mut()[ptr as usize .. ptr as usize + len].clone_from_slice(bytes);
        }

        // main call
        let res_ptr = f(ptr as u32, len as u32).unwrap();

        // get result
        let x: &ReturnData = unsafe {
            let y: *const u8 = &self.memory.data_unchecked_mut()[res_ptr as usize];
            let cast_res: *const ReturnData = y.cast();
            &*cast_res
        };
        let success = x.success;
        let v = unsafe {
            let s_slice = &self.memory.data_unchecked()[x.ptr as usize..][..x.len as usize];
            s_slice.to_vec()
        };

        // free result memory
        (self.free)(res_ptr).unwrap();


        if success {
            Ok(v)
        } else {
            Err(String::from_utf8(v).unwrap())
        }
    }
}
