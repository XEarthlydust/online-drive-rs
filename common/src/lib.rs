pub mod config;
pub mod context;
pub mod handler;
pub mod module;
pub mod util;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
