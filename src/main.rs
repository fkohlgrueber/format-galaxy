fn main() {
    println!("Hello, world!");
}

pub trait GalaxyFormat
{
    fn present(bytes: &[u8]) -> Result<String, String>;

    fn store(s: &str) -> Result<Vec<u8>, String>;
}