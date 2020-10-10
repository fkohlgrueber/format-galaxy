
use format_galaxy_core::gen_plugin;

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
