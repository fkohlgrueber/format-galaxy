use anyhow::Result;

pub trait GalaxyFormatPluginV1_ {
    fn alloc(&self, size: u32) -> Result<u32>;
    fn free(&self, ptr: u32) -> Result<()>;
    fn present(&self, ptr: u32, size: u32) -> Result<u32>;
    fn store(&self, ptr: u32, size: u32) -> Result<u32>;
    fn result_get_ptr(&self, res_ptr: u32) -> Result<u32>;
    fn result_get_len(&self, res_ptr: u32) -> Result<u32>;
    fn result_get_success(&self, res_ptr: u32) -> Result<bool>;
    
    fn memory_write(&self, ptr: u32, bytes: &[u8]) -> Result<()>;
    fn memory_read(&self, ptr: u32, len: u32) -> Result<Vec<u8>>;

    fn handle_call<T: Fn(u32, u32) -> Result<u32>>(&self, bytes: &[u8], f: &T) -> anyhow::Result<Result<Vec<u8>, String>> {
        // allocate memory and store bytes
        let len =bytes.len();
        let ptr = self.alloc(len as u32)?;
        self.memory_write(ptr, bytes)?;

        // main call
        let res_ptr = f(ptr as u32, len as u32)?;

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
    fn present(&self, bytes: &[u8]) -> Result<Result<String, String>> {
        let f = |ptr, len| <Self as GalaxyFormatPluginV1_>::present(self, ptr, len);
        Ok(match self.handle_call(bytes, &f)? {
            Ok(bytes) => Ok(String::from_utf8(bytes)?),
            Err(s) => Err(s)
        })
    }

    fn store(&self, s: &str) -> Result<Result<Vec<u8>, String>> {
        let f = |ptr, len| <Self as GalaxyFormatPluginV1_>::store(self, ptr, len);
        self.handle_call(s.as_bytes(), &f)
    }
}

impl<T> GalaxyFormatPluginV1 for T
where T: GalaxyFormatPluginV1_ {}
