use anyhow::Result;

pub trait GalaxyFormatPluginV1_ {
    fn alloc(&mut self, size: u32) -> Result<u32>;
    fn free(&mut self, ptr: u32) -> Result<()>;
    fn present(&mut self, ptr: u32, size: u32) -> Result<u32>;
    fn store(&mut self, ptr: u32, size: u32) -> Result<u32>;
    fn result_get_ptr(&mut self, res_ptr: u32) -> Result<u32>;
    fn result_get_len(&mut self, res_ptr: u32) -> Result<u32>;
    fn result_get_success(&mut self, res_ptr: u32) -> Result<bool>;
    
    fn memory_write(&mut self, ptr: u32, bytes: &[u8]) -> Result<()>;
    fn memory_read(&mut self, ptr: u32, len: u32) -> Result<Vec<u8>>;

    fn handle_call<T: FnMut(&mut Self, u32, u32) -> Result<u32>>(&mut self, bytes: &[u8], f: &mut T) -> anyhow::Result<Result<Vec<u8>, String>> {
        // allocate memory and store bytes
        let len =bytes.len();
        let ptr = self.alloc(len as u32)?;
        self.memory_write(ptr, bytes)?;

        // main call
        let res_ptr = f(self, ptr as u32, len as u32)?;

        // get result
        let ptr = self.result_get_ptr(res_ptr)?;
        let len = self.result_get_len(res_ptr)?;
        let success = self.result_get_success(res_ptr)?;

        let v = self.memory_read(ptr, len)?;

        // free result memory
        self.free(res_ptr)?;

        Ok(if success {
            Ok(v)
        } else {
            Err(String::from_utf8(v)?)
        })
    }
}

pub trait GalaxyFormatPluginV1 : GalaxyFormatPluginV1_ {
    fn present(&mut self, bytes: &[u8]) -> Result<Result<String, String>> {
        Ok(match self.handle_call(bytes, &mut <Self as GalaxyFormatPluginV1_>::present)? {
            Ok(bytes) => Ok(String::from_utf8(bytes)?),
            Err(s) => Err(s)
        })
    }

    fn store(&mut self, s: &str) -> Result<Result<Vec<u8>, String>> {
        self.handle_call(s.as_bytes(), &mut <Self as GalaxyFormatPluginV1_>::store)
    }
}

impl<T> GalaxyFormatPluginV1 for T
where T: GalaxyFormatPluginV1_ {}
