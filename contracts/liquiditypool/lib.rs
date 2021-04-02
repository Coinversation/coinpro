// Copyright 2018-2021 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod poolfactory {
    use ink_storage::collections::HashMap as StorageHashMap;
    use ink_lang::ToAccountId;
    use ink_env::call::FromAccountId;

    use math::Math;
    use base::Base;
    use token::Token;
    use pool::Pool;

    #[ink(storage)]
    pub struct PoolFactory {
        version: u32,
        math_address: AccountId,
        base_address: AccountId,

        token_code_hash: Hash,
        pool_code_hash: Hash,

        is_pool: StorageHashMap<AccountId, bool>,
        labs: AccountId,
    }

    #[ink(event)]
    pub struct LogNewPool {
        #[ink(topic)]
        caller: Option<AccountId>,
        #[ink(topic)]
        pool: Option<AccountId>,
    }

    #[ink(event)]
    pub struct LogLabs {
        #[ink(topic)]
        caller: Option<AccountId>,
        #[ink(topic)]
        labs: Option<AccountId>,
    }

    impl PoolFactory {
        #[ink(constructor)]
        pub fn new(version: u32,
                   math_address: AccountId,
                   base_address: AccountId,
                   token_code_hash: Hash,
                   pool_code_hash: Hash) -> Self {
            let rs = StorageHashMap::new();
            let lab = Self::env().caller();
            Self {
                version,
                math_address,
                base_address,

                token_code_hash,
                pool_code_hash,

                is_pool: rs,
                labs: lab,
            }
        }

        #[ink(message)]
        pub fn is_pool(&self, b: AccountId) -> bool {
            return self.is_pool[&b];
        }

        #[ink(message)]
        pub fn new_pool(&self) -> Pool {
            let salt = self.version.to_le_bytes();
            let token_params = Token::new(self.math_address)
                .endowment(0)
                .code_hash(self.token_code_hash)
                .salt_bytes(salt)
                .params();

            let token_address = self
                .env()
                .instantiate_contract(&token_params)
                .expect("failed at instantiating the `Token` contract");

            let pool_params = Pool::new(self.math_address, self.base_address, token_address)
                .endowment(0)
                .code_hash(self.pool_code_hash)
                .salt_bytes(salt)
                .params();

            let pool_address = self
                .env()
                .instantiate_contract(&pool_params)
                .expect("failed at instantiating the `pool` contract");

            let sender = Self::env().caller();
            self.env().emit_event(LogNewPool {
                caller: Some(sender),
                pool: Some(pool_address),
            });

            let mut p: Pool = FromAccountId::from_account_id(pool_address);
            p.set_controller(sender);
            return p;
        }

        #[ink(message)]
        pub fn get_labs(&self) -> AccountId {
            self.labs
        }

        #[ink(message)]
        pub fn set_labs(&mut self, b: AccountId) {
            let sender = Self::env().caller();
            assert!(sender == self.labs, "ERR_NOT_CONVLABS");
            self.env().emit_event(LogLabs {
                caller: Some(sender),
                labs: Some(b),
            });

            self.labs = b;
        }

        #[ink(message)]
        pub fn collect(&mut self, pool_address: AccountId) {
            assert!(Self::env().caller() == self.labs, "ERR_NOT_CONVLABS");
            let this = self.env().account_id();
            let mut p: Pool = FromAccountId::from_account_id(pool_address);
            let collected = p.balance_of(this);
            let r = p.transfer(self.labs, collected);
            assert!(r, "ERR_TOKEN_FAILED");
        }
    }
}
