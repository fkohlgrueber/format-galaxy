
use format_galaxy_core::gen_plugin;
use anyhow::anyhow;

struct Impl {}

fn present_inner(bytes: &[u8]) -> Result<String, anyhow::Error> {
    wasmprinter::print_bytes(bytes)
}

fn store_inner(s: &str) -> Result<Vec<u8>, anyhow::Error> {
    wat::parse_str(s).map_err(|e| anyhow!("This didn't work: {}", e))
}

impl format_galaxy_core::GalaxyFormat for Impl {
    fn present(bytes: &[u8]) -> Result<String, String> {
        present_inner(bytes).map_err(|e| e.to_string())
    }

    fn store(s: &str) -> Result<Vec<u8>, String> {
        store_inner(s).map_err(|e| e.to_string())
    }
}

gen_plugin!{Impl}
