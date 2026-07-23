pub mod config;
pub mod error;
pub mod types;

// Re-export yang paling umum digunakan agar lebih mudah diakses
pub use config::Config;
pub use error::{BotError, BotResult};
pub use types::*;

pub fn add(left: usize, right: usize) -> usize {
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
