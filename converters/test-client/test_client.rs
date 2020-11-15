
use format_galaxy_core::gen_plugin;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// uncomment to trace allocations
/*
use std::alloc::{GlobalAlloc, System, Layout};

extern "C" {
    fn print_alloc(is_alloc: u32, ptr: u32, size: u32);
}

struct MyAllocator;

unsafe impl GlobalAlloc for MyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        print_alloc(1, ptr as u32, layout.size() as u32);
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        print_alloc(0, ptr as u32, layout.size() as u32);
    }
}

#[global_allocator]
static GLOBAL: MyAllocator = MyAllocator;
*/

struct Impl {}

impl format_galaxy_core::GalaxyFormat for Impl {
    fn present(bytes: &[u8]) -> Result<String, String> {
        // TODO: provide real implementation
        if bytes.is_empty() {
            Err("I don't like empty Strings!".to_string())
        } else {
            let s = bytes.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",");
            Ok(s)
        }
    }

    fn store(s: &str) -> Result<Vec<u8>, String> {
        // TODO: provide real implementation
        match s.split(',').map(|x| x.parse()).collect::<Result<Vec<u8>,_>>() {
            Ok(v) => Ok(v),
            Err(_) => Err("I don't store this!".to_string())
        }
    }
}

gen_plugin!{Impl}

#[test]
fn test_impl() {
    use format_galaxy_core::GalaxyFormat;
    assert_eq!(Impl::present(&[1,2,3]), Ok("1,2,3".to_string()));
    assert!(Impl::present(&[]).is_err());
    assert_eq!(Impl::store("1,2,3"), Ok(vec!(1,2,3)));
}
