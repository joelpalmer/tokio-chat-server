pub mod client;
pub mod protocol;
pub mod runtime;
pub mod server;

// Re-export public items for convenience
pub use runtime::run_server;
pub use server::ChatServer;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
