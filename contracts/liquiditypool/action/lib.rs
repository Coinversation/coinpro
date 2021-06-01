#![cfg_attr(not(feature = "std"), no_std)]

pub use self::action::Action;
use ink_lang as ink;

#[ink::contract]
mod action {
    use ink_env::call::FromAccountId;
    use ink_env::debug_println;
    use ink_prelude::{
        vec::Vec,
    };

    use cdot::PAT;
    use pool::Pool;
    use factory::Factory;

    #[ink(storage)]
    pub struct Action {
        controller: AccountId,
    }

    impl Action {
        #[ink(constructor)]
        pub fn new(controller: AccountId) -> Self {
            Self {
                controller,
            }
        }

        #[ink(message)]
        pub fn get_controller(&self) -> AccountId {
                return self.controller;
        }

        fn _pull_underlying(&self, erc20: AccountId, from: AccountId, to: AccountId, amount: u128) {
            let message1 = ink_prelude::format!("_pull_underlying amount is {:?}", amount);
            ink_env::debug_println(&message1);

            let mut erc: PAT = FromAccountId::from_account_id(erc20);
            let fer = erc.transfer_from(from, to, amount).is_ok();
            assert!(fer);

            let b = erc.balance_of(to);
            let message1 = ink_prelude::format!("after _pull_underlying to amount is {:?}", b);
            ink_env::debug_println(&message1);
        }

        fn _bind(&mut self, token: AccountId, pool_address: AccountId, balance: u128, weight: u128) {
            self._safe_approve(token, pool_address, balance);

            let mut pool: Pool = FromAccountId::from_account_id(pool_address);
            pool.bind(token, balance, weight);
        }

        #[ink(message)]
        pub fn init_tokens(&mut self,
                        pool_address: AccountId,
                        tokens: Vec<AccountId>,
                        balances: Vec<u128>,
                        weights: Vec<u128>,) {
            let sender = Self::env().caller();
            let this = self.env().account_id();

            let mut i = 0;
            while i < tokens.len() {
                self._pull_underlying(tokens[i], sender, this, balances[i]);
                self._bind(tokens[i], pool_address, balances[i], weights[i]);
                debug_println("bind finish!");
                i += 1;
            }
        }


        #[ink(message)]
        pub fn create(&mut self,
                      salt: u32,
                      token_endowment: u128,
                      pool_endowment: u128,
                      factory_address: AccountId,
                      tokens: Vec<AccountId>,
                      balances: Vec<u128>,
                      weights: Vec<u128>,
                      swap_fee: u128,
                      finalize: bool) -> AccountId {
            let sender = Self::env().caller();
            let this = self.env().account_id();

            debug_println("enter create");

            assert!(self.controller == sender, "ERR_NOT_CONTROLLER");

            assert!(tokens.len() == balances.len(), "ERR_LENGTH_MISMATCH");
            assert!(tokens.len() == weights.len(), "ERR_LENGTH_MISMATCH");

            debug_println("tokens valid");

            let mut factory: Factory = FromAccountId::from_account_id(factory_address);
            let pool_address = factory.new_pool(salt, token_endowment, pool_endowment);
            debug_println("new pool finish");

            // self.init_tokens(pool_address, tokens, balances, weights);

            // let mut i = 0;
            // while i < tokens.len() {
            //     self._pull_underlying(tokens[i], sender, this, balances[i]);
            //     // let mut token: PAT = FromAccountId::from_account_id(tokens[i]);
            //     // assert!(token.transfer_from(sender, this, balances[i]).is_ok(), "ERR_TRANSFER_FAILED");
            //     self._bind(tokens[i], pool_address, balances[i], weights[i]);
            //     // self._safe_approve(tokens[i], pool_address, balances[i]);
            //     // pool.bind(tokens[i], balances[i], weights[i]);
            //     debug_println("bind finish!");
            //     i += 1;
            // }

            debug_println("all binds finish!");

            debug_println("set_swap_fee finish");

            // if finalize {
            //     debug_println("ready finalize!");
            //     pool.finalize();
            //     debug_println("finalize finish!");
            //     assert!(pool.transfer(sender, pool.balance_of(this)), "ERR_TRANSFER_FAILED");
            //
            //     debug_println("finalize finish!");
            // } else {
            //     debug_println("ready set_public_swap!");
            //     pool.set_public_swap(true);
            //     debug_println("set_public_swap! finish");
            // }

            return pool_address;
        }

        #[ink(message)]
        pub fn join_pool(&mut self,
                         pool_address: AccountId,
                         pool_amount_out: u128,
                         max_amounts_in: Vec<u128>)  {
            debug_println("enter");
            let sender = Self::env().caller();
            assert!(self.controller == sender, "ERR_NOT_CONTROLLER");

            debug_println("caller valid");
            let pool: Pool = FromAccountId::from_account_id(pool_address);
            let tokens = pool.get_final_tokens();
            debug_println("get_final_tokens finish");
            self._join(pool_address, tokens, pool_amount_out, max_amounts_in);
            debug_println("_join finish");
        }

        #[ink(message)]
        pub fn join_swap_extern_amount_in(&mut self,
                                          pool_address: AccountId,
                                          token_address: AccountId,
                                          token_amount_in: u128,
                                          min_pool_amount_out: u128)  {
            debug_println("enter");
            let sender = Self::env().caller();
            let this = self.env().account_id();
            assert!(self.controller == sender, "ERR_NOT_CONTROLLER");

            debug_println("params valid");
            let mut token: PAT = FromAccountId::from_account_id(token_address);
            assert!(token.transfer_from(sender, this, token_amount_in).is_ok(), "ERR_TRANSFER_FAILED");

            debug_println("transfer_from finish");

            self._safe_approve(token_address, pool_address, token_amount_in);

            debug_println("_safe_approve finish");

            let mut pool: Pool = FromAccountId::from_account_id(pool_address);
            let pool_amount_out = pool.join_swap_extern_amount_in(token_address, token_amount_in, min_pool_amount_out);

            assert!(pool.transfer(sender, pool_amount_out), "ERR_TRANSFER_FAILED");
            debug_println("pool.transfer finish");
        }

        #[ink(message)]
        pub fn set_public_swap(&mut self, pool_address: AccountId, public_swap: bool) {
            let sender = Self::env().caller();
            assert!(self.controller == sender, "ERR_NOT_CONTROLLER");

            let mut pool: Pool = FromAccountId::from_account_id(pool_address);
            pool.set_public_swap(public_swap);
        }

        #[ink(message)]
        pub fn set_swap_fee(&mut self, pool_address: AccountId, new_fee: u128) {
            let sender = Self::env().caller();
            assert!(self.controller == sender, "ERR_NOT_CONTROLLER");

            let mut pool: Pool = FromAccountId::from_account_id(pool_address);
            pool.set_swap_fee(new_fee);
        }

        #[ink(message)]
        pub fn set_controller(&mut self, pool_address: AccountId, new_controller: AccountId) {
            let sender = Self::env().caller();
            assert!(self.controller == sender, "ERR_NOT_CONTROLLER");

            let mut pool: Pool = FromAccountId::from_account_id(pool_address);
            pool.set_controller(new_controller);
        }

        #[ink(message)]
        pub fn set_tokens(&mut self, pool_address: AccountId,
                         tokens: Vec<AccountId>,
                         balances: Vec<u128>,
                         denorms: Vec<u128>) {
            debug_println("enter");
            let sender = Self::env().caller();
            assert!(self.controller == sender, "ERR_NOT_CONTROLLER");

            assert!(tokens.len() == balances.len(), "ERR_LENGTH_MISMATCH");
            assert!(tokens.len() == denorms.len(), "ERR_LENGTH_MISMATCH");

            debug_println("params valid");

            let mut pool: Pool = FromAccountId::from_account_id(pool_address);

            let this = self.env().account_id();

            let mut i = 0;
            while i < tokens.len() {
                let mut token: PAT = FromAccountId::from_account_id(tokens[i]);
                if pool.is_bound(tokens[i]) {
                    if balances[i] > pool.get_balance(tokens[i]) {
                        assert!(token.transfer_from(sender, this, balances[i] - pool.get_balance(tokens[i])).is_ok(), "ERR_TRANSFER_FAILED");
                        self._safe_approve(tokens[i], pool_address, balances[i] - pool.get_balance(tokens[i]));
                    }

                    if balances[i] >= 10000 {
                        pool.rebind(tokens[i], balances[i], denorms[i]);
                    } else {
                        pool.unbind(tokens[i]);
                    }

                    debug_println("pool.is_bound");
                } else {
                    assert!(token.transfer_from(sender, this, balances[i]).is_ok(), "ERR_TRANSFER_FAILED");
                    self._safe_approve(tokens[i], pool_address, balances[i]);
                    pool.bind(tokens[i], balances[i], denorms[i]);

                    debug_println("pool.bind");
                }

                if token.balance_of(this) > 0 {
                    assert!(token.transfer(sender, token.balance_of(this)).is_ok(), "ERR_TRANSFER_FAILED");
                    debug_println("token.transfer finish");
                }

                i += 1;
            }
        }

        #[ink(message)]
        pub fn finalize(&mut self, pool_address: AccountId) {
            let sender = Self::env().caller();
            assert!(self.controller == sender, "ERR_NOT_CONTROLLER");
            debug_println("enter");

            let mut pool: Pool = FromAccountId::from_account_id(pool_address);
            let this = self.env().account_id();

            pool.finalize();

            debug_println("pool.finalize finish");
            assert!(pool.transfer(sender, pool.balance_of(this)), "ERR_TRANSFER_FAILED");
            debug_println("pool.transfer finish");
        }

        fn _safe_approve(&mut self, token_address: AccountId, spender: AccountId, amount: u128) {
            debug_println("enter");
            let mut token: PAT = FromAccountId::from_account_id(token_address);
            let this = self.env().account_id();

            if token.allowance(this, spender) > 0 {
                token.approve(spender, 0);
            }
            token.approve(spender, amount);
            debug_println("approve finish");
        }

        fn _join(&mut self, pool_address: AccountId, tokens: Vec<AccountId>, pool_amount_out: u128, max_amounts_in: Vec<u128>) {
            debug_println("enter");
            assert!(max_amounts_in.len() == tokens.len(), "ERR_LENGTH_MISMATCH");

            let mut pool: Pool = FromAccountId::from_account_id(pool_address);

            let this = self.env().account_id();
            let sender = Self::env().caller();

            let mut i = 0;
            while i < tokens.len() {
                let mut token: PAT = FromAccountId::from_account_id(tokens[i]);
                assert!(token.transfer_from(sender, this, max_amounts_in[i]).is_ok(), "ERR_TRANSFER_FAILED");
                self._safe_approve(tokens[i], pool_address, max_amounts_in[i]);
                i += 1;

                debug_println("self._safe_approve!!");
            }

            pool.join_pool(pool_amount_out, max_amounts_in);
            debug_println("pool.join_pool finish!!");

            let mut j = 0;
            while j < tokens.len() {
                let mut token: PAT = FromAccountId::from_account_id(tokens[j]);
                if token.balance_of(this) > 0 {
                    assert!(token.transfer(sender, token.balance_of(this)).is_ok(), "ERR_TRANSFER_FAILED");
                }
                j += 1;

                debug_println("token.transfer finish!!");
            }
            assert!(pool.transfer(sender, pool.balance_of(this)), "ERR_TRANSFER_FAILED");
            debug_println("pool.transfer finish!!");
        }
    }
}
