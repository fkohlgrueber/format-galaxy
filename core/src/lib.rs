#![feature(vec_into_raw_parts)]

pub trait GalaxyFormat
{
    fn present(bytes: &[u8]) -> Result<String, String>;

    fn store(s: &str) -> Result<Vec<u8>, String>;
}

pub mod __mi {
    use super::*;

    #[repr(C)]
    #[derive(Debug)]
    pub struct ReturnData {
        pub ptr: u32,
        pub len: u32,
        pub capacity: u32,
        pub success: bool
    }

    pub fn present<T: GalaxyFormat>(ptr: *mut u8, len: u32) -> *mut ReturnData {
        // input is a byte slice
        let bytes = unsafe { Vec::from_raw_parts(ptr, len as usize, len as usize) };
        
        let res = <T as GalaxyFormat>::present(&bytes);
        let success = res.is_ok();
        let bytes = match res {
            Ok(s) => s.into_bytes(),
            Err(s) => s.into_bytes(),
        };
        alloc_result(bytes, success)
    }

    pub fn store<T: GalaxyFormat>(ptr: *mut u8, len: u32) -> *mut ReturnData {
        // input is a utf-8 string
        let s = unsafe { String::from_raw_parts(ptr, len as usize, len as usize) };
        let res = <T as GalaxyFormat>::store(&s);
        let success = res.is_ok();
        let bytes = match res {
            Ok(bytes) => bytes,
            Err(s) => s.into_bytes(),
        };
        alloc_result(bytes, success)
    }
    
    pub fn alloc_result(data: Vec<u8>, success: bool) -> *mut ReturnData {
        let (ptr, len, capacity) = data.into_raw_parts();
        let res_data = ReturnData {
            ptr: ptr as u32, 
            len: len as u32, 
            capacity: capacity as u32, 
            success
        };
        let res_box = Box::new(res_data);
        std::boxed::Box::into_raw(res_box)
    }

    pub fn result_get_ptr(ptr: *mut ReturnData) -> u32 {
        return unsafe { (*ptr).ptr };
    }

    pub fn result_get_len(ptr: *mut ReturnData) -> u32 {
        return unsafe { (*ptr).len };
    }

    pub fn result_get_success(ptr: *mut ReturnData) -> u32 {
        return unsafe { (*ptr).success as u32 };
    }
    
    pub fn alloc(n: u32) -> *mut u8 {
        let v: Vec<u8> = Vec::with_capacity(n as usize);
    
        let (ptr, _len, capacity) = v.into_raw_parts();
        assert!(capacity == n as usize);
        ptr
    }
    
    pub fn free(ptr: *mut ReturnData) {
        unsafe {
            let ret_box = std::boxed::Box::from_raw(ptr);
            let _v = Vec::from_raw_parts(
                ret_box.ptr as *mut u8, 
                ret_box.len as usize, 
                ret_box.capacity as usize
            );
        }
    }   
}


#[macro_export]
macro_rules! gen_plugin {
    ($impl_type:ty) => {
        #[cfg(target_arch="wasm32")]
        mod plugin {
            use super::*;

            #[no_mangle]
            pub extern "C" fn present(ptr: *mut u8, len: u32) -> *mut format_galaxy_core::__mi::ReturnData {
                format_galaxy_core::__mi::present::<$impl_type>(ptr, len)
            }
            
            #[no_mangle]
            pub extern "C" fn store(ptr: *mut u8, len: u32) -> *mut format_galaxy_core::__mi::ReturnData {
                format_galaxy_core::__mi::store::<$impl_type>(ptr, len)
            }
            
            #[no_mangle]
            pub extern "C" fn alloc(n: u32) -> *mut u8 {
                format_galaxy_core::__mi::alloc(n)
            }
            
            #[no_mangle]
            pub extern "C" fn free(ptr: *mut format_galaxy_core::__mi::ReturnData) {
                format_galaxy_core::__mi::free(ptr)
            }

            #[no_mangle]
            pub extern "C" fn result_get_ptr(ptr: *mut format_galaxy_core::__mi::ReturnData) -> u32 {
                format_galaxy_core::__mi::result_get_ptr(ptr)
            }

            #[no_mangle]
            pub extern "C" fn result_get_len(ptr: *mut format_galaxy_core::__mi::ReturnData) -> u32 {
                format_galaxy_core::__mi::result_get_len(ptr)
            }

            #[no_mangle]
            pub extern "C" fn result_get_success(ptr: *mut format_galaxy_core::__mi::ReturnData) -> u32 {
                format_galaxy_core::__mi::result_get_success(ptr)
            }

        }
    };
}