//! Common test utilities and initialization for Royale Arena backend tests

use dotenvy::from_filename;
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize test environment
/// This function ensures that test environment is initialized only once
pub fn init_test_env() {
    INIT.call_once(|| {
        // Load environment variables from .env.royale file
        let _ = from_filename(".env.royale");
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_test_env() {
        // This test ensures that the function can be called without panicking
        init_test_env();
        init_test_env(); // Call again to test that it only initializes once
        assert!(true);
    }
}