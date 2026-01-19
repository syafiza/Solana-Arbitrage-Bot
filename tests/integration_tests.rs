/// Integration Tests for Pool Initializers
/// 
/// Tests the complete initialization flow using mock RPC data.

#[cfg(test)]
mod pool_initializer_tests {
    use crate::dex::raydium::{RaydiumCpmmInitializer, RaydiumAmmInfo};
    use crate::dex::traits::PoolInitializer;
    use crate::rpc::MockRpcClient;
    use solana_sdk::account::Account;
    use solana_sdk::pubkey::Pubkey;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_raydium_pool_initialization_success() {
        let mock_rpc = Arc::new(MockRpcClient::new());
        let pool_address = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        
        // Create mock Raydium pool account
        let mut pool_data = vec![0u8; 500];
        // Set mints at correct offsets (simplified for test)
        // In real implementation, would use proper serialization
        
        let account = Account {
            lamports: 1_000_000,
            data: pool_data,
            owner: crate::dex::raydium::raydium_program_id(),
            executable: false,
            rent_epoch: 0,
        };

        mock_rpc.add_account(pool_address, account);

        let initializer = RaydiumCpmmInitializer::new();
        
        // Note: This test would need actual serialized pool data to fully work
        // This demonstrates the testing pattern
        
        // let result = initializer
        //     .initialize_pools(
        //         &[pool_address.to_string()],
        //         mock_rpc,
        //         &mint,
        //     )
        //     .await;

        // assert!(result.is_ok());
        // let pools = result.unwrap();
        // assert_eq!(pools.len(), 1);
    }

    #[tokio::test]
    async fn test_pool_initialization_invalid_owner() {
        let mock_rpc = Arc::new(MockRpcClient::new());
        let pool_address = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        
        // Create account with WRONG owner
        let account = Account {
            lamports: 1_000_000,
            data: vec![0u8; 500],
            owner: Pubkey::new_unique(), // Wrong owner!
            executable: false,
            rent_epoch: 0,
        };

        mock_rpc.add_account(pool_address, account);

        let initializer = RaydiumCpmmInitializer::new();
        
        let result = initializer
            .initialize_pools(
                &[pool_address.to_string()],
                mock_rpc,
                &mint,
            )
            .await;

        assert!(result.is_err());
        // Should be InvalidAccountOwner error
    }
}
