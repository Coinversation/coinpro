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
        pub fn create(&self,
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
            assert!(self.controller == sender, "ERR_NOT_CONTROLLER");

            assert!(tokens.len() == balances.len(), "ERR_LENGTH_MISMATCH");
            assert!(tokens.len() == weights.len(), "ERR_LENGTH_MISMATCH");

            let mut factory: Factory = FromAccountId::from_account_id(factory_address);
            let pool_address = factory.new_pool(salt, token_endowment, pool_endowment);

            let mut pool: Pool = FromAccountId::from_account_id(pool_address);
            pool.set_swap_fee(swap_fee);

            let this = self.env().account_id();
            let sender = Self::env().caller();

            let mut i = 0;
            while i < tokens.len() {
                let mut token: PAT = FromAccountId::from_account_id(tokens[i]);
                assert!(token.transfer_from(sender, this, balances[i]).is_ok(), "ERR_TRANSFER_FAILED");
                self._safe_approve(tokens[i], pool_address, balances[i]);
                pool.bind(tokens[i], balances[i], weights[i]);

                i += 1;
            }

            if finalize {
                pool.finalize();
                assert!(pool.transfer(sender, pool.balance_of(this)), "ERR_TRANSFER_FAILED");
            } else {
                pool.set_public_swap(true);
            }

            return pool_address;
        }

        #[ink(message)]
        pub fn join_pool(&self,
                         pool_address: AccountId,
                         pool_amount_out: u128,
                         max_amounts_in: Vec<u128>)  {
            let sender = Self::env().caller();
            assert!(self.controller == sender, "ERR_NOT_CONTROLLER");

            let pool: Pool = FromAccountId::from_account_id(pool_address);
            let tokens = pool.get_final_tokens();
            self._join(pool_address, tokens, pool_amount_out, max_amounts_in);
        }

        #[ink(message)]
        pub fn join_swap_extern_amount_in(&self,
                                          pool_address: AccountId,
                                          token_address: AccountId,
                                          token_amount_in: u128,
                                          min_pool_amount_out: u128)  {
            let sender = Self::env().caller();
            let this = self.env().account_id();
            assert!(self.controller == sender, "ERR_NOT_CONTROLLER");

            let mut token: PAT = FromAccountId::from_account_id(token_address);
            assert!(token.transfer_from(sender, this, token_amount_in).is_ok(), "ERR_TRANSFER_FAILED");

            self._safe_approve(token_address, pool_address, token_amount_in);

            let mut pool: Pool = FromAccountId::from_account_id(pool_address);
            let pool_amount_out = pool.join_swap_extern_amount_in(token_address, token_amount_in, min_pool_amount_out);

            assert!(pool.transfer(sender, pool_amount_out), "ERR_TRANSFER_FAILED");
        }

        #[ink(message)]
        pub fn set_public_swap(&self, pool_address: AccountId, public_swap: bool) {
            let sender = Self::env().caller();
            assert!(self.controller == sender, "ERR_NOT_CONTROLLER");

            let mut pool: Pool = FromAccountId::from_account_id(pool_address);
            pool.set_public_swap(public_swap);
        }

        #[ink(message)]
        pub fn set_swap_fee(&self, pool_address: AccountId, new_fee: u128) {
            let sender = Self::env().caller();
            assert!(self.controller == sender, "ERR_NOT_CONTROLLER");

            let mut pool: Pool = FromAccountId::from_account_id(pool_address);
            pool.set_swap_fee(new_fee);
        }

        #[ink(message)]
        pub fn set_controller(&self, pool_address: AccountId, new_controller: AccountId) {
            let sender = Self::env().caller();
            assert!(self.controller == sender, "ERR_NOT_CONTROLLER");

            let mut pool: Pool = FromAccountId::from_account_id(pool_address);
            pool.set_controller(new_controller);
        }

        #[ink(message)]
        pub fn set_tokens(&self, pool_address: AccountId,
                         tokens: Vec<AccountId>,
                         balances: Vec<u128>,
                         denorms: Vec<u128>) {
            let sender = Self::env().caller();
            assert!(self.controller == sender, "ERR_NOT_CONTROLLER");

            assert!(tokens.len() == balances.len(), "ERR_LENGTH_MISMATCH");
            assert!(tokens.len() == denorms.len(), "ERR_LENGTH_MISMATCH");

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

                    if balances[i] > 1000000 {
                        pool.rebind(tokens[i], balances[i], denorms[i]);
                    } else {
                        pool.unbind(tokens[i]);
                    }
                } else {
                    assert!(token.transfer_from(sender, this, balances[i]).is_ok(), "ERR_TRANSFER_FAILED");
                    self._safe_approve(tokens[i], pool_address, balances[i]);
                    pool.bind(tokens[i], balances[i], denorms[i]);
                }

                if token.balance_of(this) > 0 {
                    assert!(token.transfer(sender, token.balance_of(this)).is_ok(), "ERR_TRANSFER_FAILED");
                }

                i += 1;
            }
        }

        #[ink(message)]
        pub fn finalize(&self, pool_address: AccountId) {
            let sender = Self::env().caller();
            assert!(self.controller == sender, "ERR_NOT_CONTROLLER");

            let mut pool: Pool = FromAccountId::from_account_id(pool_address);
            let this = self.env().account_id();

            pool.finalize();
            assert!(pool.transfer(sender, pool.balance_of(this)), "ERR_TRANSFER_FAILED");
        }

        fn _safe_approve(&self, token_address: AccountId, spender: AccountId, amount: u128) {
            let mut token: PAT = FromAccountId::from_account_id(token_address);
            let this = self.env().account_id();

            if token.allowance(this, spender) > 0 {
                token.approve(spender, 0);
            }
            token.approve(spender, amount);
        }

        fn _join(&self, pool_address: AccountId, tokens: Vec<AccountId>, pool_amount_out: u128, max_amounts_in: Vec<u128>) {
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
            }

            pool.join_pool(pool_amount_out, max_amounts_in);

            let mut j = 0;
            while j < tokens.len() {
                let mut token: PAT = FromAccountId::from_account_id(tokens[j]);
                if token.balance_of(this) > 0 {
                    assert!(token.transfer(sender, token.balance_of(this)).is_ok(), "ERR_TRANSFER_FAILED");
                }
                j += 1;
            }
            assert!(pool.transfer(sender, pool.balance_of(this)), "ERR_TRANSFER_FAILED");
        }
    }
}
