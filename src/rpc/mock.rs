/// Mock RPC Client for Testing
/// 
/// Provides a mock implementation of RpcClient for unit and integration testing
/// without requiring actual Solana RPC endpoints.

use solana_client::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Mock RPC client for testing
pub struct MockRpcClient {
    accounts: Arc<RwLock<HashMap<Pubkey, Account>>>,
    latest_blockhash: Arc<RwLock<Hash>>,
}

impl MockRpcClient {
    /// Create a new mock RPC client
    pub fn new() -> Self {
        Self {
            accounts: Arc::new(RwLock::new(HashMap::new())),
            latest_blockhash: Arc::new(RwLock::new(Hash::default())),
        }
    }

    /// Add a mock account
    pub fn add_account(&self, pubkey: Pubkey, account: Account) {
        let mut accounts = self.accounts.write().unwrap();
        accounts.insert(pubkey, account);
    }

    /// Set the latest blockhash
    pub fn set_latest_blockhash(&self, hash: Hash) {
        let mut latest = self.latest_blockhash.write().unwrap();
        *latest = hash;
    }

    /// Get account (test helper)
    pub fn get_account(&self, pubkey: &Pubkey) -> Option<Account> {
        let accounts = self.accounts.read().unwrap();
        accounts.get(pubkey).cloned()
    }

    /// Get latest blockhash (test helper)
    pub fn get_latest_blockhash(&self) -> Hash {
        *self.latest_blockhash.read().unwrap()
    }

    /// Clear all mock data
    pub fn clear(&self) {
        let mut accounts = self.accounts.write().unwrap();
        accounts.clear();
    }
}

impl Default for MockRpcClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::native_token::LAMPORTS_PER_SOL;

    #[test]
    fn test_mock_rpc_client_accounts() {
        let mock = MockRpcClient::new();
        let pubkey = Pubkey::new_unique();
        
        let account = Account {
            lamports: LAMPORTS_PER_SOL,
            data: vec![1, 2, 3],
            owner: Pubkey::default(),
            executable: false,
            rent_epoch: 0,
        };

        mock.add_account(pubkey, account.clone());
        
        let retrieved = mock.get_account(&pubkey).unwrap();
        assert_eq!(retrieved.lamports, LAMPORTS_PER_SOL);
        assert_eq!(retrieved.data, vec![1, 2, 3]);
    }

    #[test]
    fn test_mock_rpc_client_blockhash() {
        let mock = MockRpcClient::new();
        let hash = Hash::new_unique();

        mock.set_latest_blockhash(hash);
        
        assert_eq!(mock.get_latest_blockhash(), hash);
    }

    #[test]
    fn test_mock_rpc_client_clear() {
        let mock = MockRpcClient::new();
        let pubkey = Pubkey::new_unique();
        
        mock.add_account(pubkey, Account::default());
        assert!(mock.get_account(&pubkey).is_some());

        mock.clear();
       assert!(mock.get_account(&pubkey).is_none());
    }
}
