
use format_galaxy_core::gen_plugin;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct Impl {}

impl format_galaxy_core::GalaxyFormat for Impl {
    fn present(bytes: &[u8]) -> Result<String, String> {
        Ok(bytes.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","))
    }

    fn store(s: &str) -> Result<Vec<u8>, String> {
        s.split(',').map(|x| x.parse()).collect::<Result<Vec<u8>,_>>()
            .map_err(|_| "Could not convert text to byte sequence.".to_string())
    }
}

gen_plugin!{Impl}

#[test]
fn test_impl() {
    use format_galaxy_core::GalaxyFormat;
    assert_eq!(Impl::present(&[1,2,3]), Ok("1,2,3".to_string()));
    assert_eq!(Impl::present(&[]), Ok("".to_string()));
    assert_eq!(Impl::store("1,2,3"), Ok(vec!(1,2,3)));
}
