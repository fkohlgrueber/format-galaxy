pub use fg_plugin::GalaxyFormatPluginV1;
use fg_plugin::GalaxyFormatPluginV1_;
use anyhow::Result;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys::{Function, Object, Reflect, WebAssembly};
use js_sys::WebAssembly::Memory;
use wasm_bindgen_futures::JsFuture;

use anyhow::anyhow;


pub struct WebGalaxyFormatPlugin {
    memory: Memory,
    alloc_fn: Function,
    free_fn: Function,
    present_fn: Function,
    store_fn: Function,
    get_result_ptr_fn: Function,
    get_result_len_fn: Function,
    get_result_success_fn: Function,
}

fn get_fn(exports: &Object, name: &str) -> Result<Function, JsValue> {
    Ok(Reflect::get(exports, &name.into())?
        .dyn_into::<Function>()
        .expect(&format!("{} export wasn't a function", name)))
}

impl WebGalaxyFormatPlugin {
    pub async fn from_slice(bytes: &[u8]) -> Result<Self, JsValue> {
        let wasm = JsFuture::from(WebAssembly::instantiate_buffer(bytes, &Object::new())).await?;
        let wasm_instance: WebAssembly::Instance = Reflect::get(&wasm, &"instance".into())?.dyn_into()?;

        let c = wasm_instance.exports();

        let memory = Reflect::get(c.as_ref(), &"memory".into())?
            .dyn_into::<WebAssembly::Memory>()
            .expect("memory export wasn't a `WebAssembly.Memory`");
        let present_fn = get_fn(c.as_ref(), "present")?;
        let store_fn = get_fn(c.as_ref(), "store")?;
        let alloc_fn = get_fn(c.as_ref(), "alloc")?;
        let free_fn = get_fn(c.as_ref(), "free")?;
        let get_result_ptr_fn = get_fn(c.as_ref(), "result_get_ptr")?;
        let get_result_len_fn = get_fn(c.as_ref(), "result_get_len")?;
        let get_result_success_fn = get_fn(c.as_ref(), "result_get_success")?;

        Ok(WebGalaxyFormatPlugin {
            memory,
            present_fn,
            store_fn,
            alloc_fn,
            free_fn,
            get_result_ptr_fn,
            get_result_len_fn,
            get_result_success_fn
        })
    }
}

fn to_u32(v: JsValue) -> Result<u32> {
    match v.as_f64() {
        Some(f) => Ok(f as u32),
        None => Err(anyhow!("Unexpected value!"))
    }
}

fn call1(f: &Function, a: u32) -> Result<JsValue> {
    match f.call1(&JsValue::undefined(), &a.into()) {
        Ok(v) => Ok(v),
        Err(v) => Err(anyhow!("Error calling wasm function: {:?}", v))
    }
}

fn call2(f: &Function, a: u32, b: u32) -> Result<JsValue> {
    match f.call2(&JsValue::undefined(), &a.into(), &b.into()) {
        Ok(v) => Ok(v),
        Err(v) => Err(anyhow!("Error calling wasm function: {:?}", v))
    }
}

impl GalaxyFormatPluginV1_ for WebGalaxyFormatPlugin {
    fn alloc(&mut self, size: u32) -> Result<u32> {
        to_u32(call1(&self.alloc_fn, size)?)
    }

    fn free(&mut self, ptr: u32) -> Result<()> {
        call1(&self.free_fn, ptr)?;
        Ok(())
    }

    fn present(&mut self, ptr: u32, size: u32) -> Result<u32> {
        to_u32(call2(&self.present_fn, ptr, size)?)
    }
    
    fn store(&mut self, ptr: u32, size: u32) -> Result<u32> {
        to_u32(call2(&self.store_fn, ptr, size)?)
    }
    
    fn result_get_ptr(&mut self, res_ptr: u32) -> Result<u32> {
        to_u32(call1(&self.get_result_ptr_fn, res_ptr)?)
    }
    
    fn result_get_len(&mut self, res_ptr: u32) -> Result<u32> {
        to_u32(call1(&self.get_result_len_fn, res_ptr)?)
    }
    
    fn result_get_success(&mut self, res_ptr: u32) -> Result<bool> {
        Ok(to_u32(call1(&self.get_result_success_fn, res_ptr)?)? > 0)
    }
    
    
    fn memory_write(&mut self, ptr: u32, bytes: &[u8]) -> Result<()> {
        let array = js_sys::Uint8Array::new(&self.memory.buffer());
        for i in 0..bytes.len() as u32 {
            array.set_index(ptr + i, bytes[i as usize]);
        }
        Ok(())
    }
    
    fn memory_read(&mut self, ptr: u32, len: u32) -> Result<Vec<u8>> {
        let mut ret_bytes: Vec<u8> = Vec::with_capacity(len as usize);
        let array = js_sys::Uint8Array::new(&self.memory.buffer());
        for i in 0..len {
            ret_bytes.push(array.get_index(ptr + i));
        }
        Ok(ret_bytes)
    }
}