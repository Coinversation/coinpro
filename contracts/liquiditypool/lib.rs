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
mod register {
    use ink_lang::ToAccountId;
    use ink_env::call::FromAccountId;
    use ink_env::debug_println;

    use action::Action;

    #[ink(storage)]
    pub struct Register {
        action_code_hash: Hash,
    }

    #[ink(event)]
    pub struct LogNewAction {
        #[ink(topic)]
        caller: Option<AccountId>,
        #[ink(topic)]
        pool: Option<AccountId>,
    }

    impl Register {
        #[ink(constructor)]
        pub fn new(action_code_hash: Hash) -> Self {
            Self {
                action_code_hash,
            }
        }

        #[ink(message)]
        pub fn new_action(&mut self, salt: u32, action_endowment: u128) -> AccountId {
            let salt_bytes = salt.to_le_bytes();
            debug_println("enter ");
            assert_ne!(self.action_code_hash, Hash::from([0; 32]));
            debug_println("action code hash valid ");

            let sender = Self::env().caller();
            let action_params = Action::new(sender)
                .endowment(action_endowment)
                .code_hash(self.action_code_hash)
                .salt_bytes(salt_bytes)
                .params();

            debug_println("build action contract params finish");

            let action_address = self
                .env()
                .instantiate_contract(&action_params)
                .expect("failed at instantiating the `Action` contract");

            debug_println("instantiate action succeed");

            self.env().emit_event(LogNewAction {
                caller: Some(sender),
                pool: Some(action_address),
            });

            debug_println("new action succeed");
            return action_address
        }
    }
}
