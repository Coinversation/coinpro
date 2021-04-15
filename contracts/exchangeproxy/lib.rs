#![cfg_attr(not(feature = "std"), no_std)]
use ink_lang as ink;
#[ink::contract]
mod exchangeproxy {
    use cdot::PAT;
    use pool::PoolInterface;
    use ink_env::call::FromAccountId;
    use ink_lang::ToAccountId;
    use ink_prelude::vec::Vec;
    use ink_storage::{
        // collections::{HashMap as StorageHashMap, Vec as StorageVec},
        traits::{PackedLayout, SpreadLayout},
        Lazy,
    };

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct ExchangeProxy {
        /// Stores a single `bool` value on the storage.
        _mutex: bool,
        cdot: Lazy<PAT>,
    }

    #[derive(
    Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode, SpreadLayout, PackedLayout,
    )]
    #[cfg_attr(
    feature = "std", derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct Swap {
        pool: AccountId,
        token_in_param: u128,
        // tokenInAmount / maxAmountIn / limitAmountIn
        token_out_param: u128,
        // mßinAmountOut / token_amount_out / limitAmountOut
        max_price: u128,
    }

    #[ink(event)]
    pub struct LOGCALL {
        // #[ink(topic)]
        // sig: [u8; 4],
        #[ink(topic)]
        caller: Option<AccountId>,
        // data: [u8; 32],
    }

    impl ExchangeProxy {
        /// Constructor that initializes the `bool` value to `false`.
        #[ink(constructor)]
        pub fn new(cdot_contract: AccountId) -> Self {
            assert_ne!(cdot_contract, Default::default());
            let cdot_token: PAT = FromAccountId::from_account_id(cdot_contract);
            Self {
                _mutex: false,
                cdot: Lazy::new(cdot_token),
            }
        }
        pub fn default() -> Self { Self::new(Default::default()) }
        #[ink(message)]
        pub fn batch_swap_exact_in(
            &mut self,
            swaps: Vec<Swap>,
            token_in: AccountId,
            token_out: AccountId,
            total_amount_in: u128,
            min_total_amount_out: u128,
        ) -> u128 {
            self._logs_();
            self._locks_();
            let mut ti: PAT = FromAccountId::from_account_id(token_in);
            let mut to: PAT = FromAccountId::from_account_id(token_out);
            let mut total_amount_out: u128 = 0;
            assert!(ti.transfer_from(self.env().caller(), self.env().account_id(), total_amount_in).is_ok());
            for x in swaps {
                let pool: PoolInterface = FromAccountId::from_account_id(x.pool);
                if ti.allowance(self.env().account_id(), x.pool) < total_amount_in {
                    ti.approve(x.pool, u128::MAX);
                }
                let token_amount_out = pool.swap_exact_amount_in(
                    token_in,
                    x.token_in_param,
                    token_out,
                    x.token_out_param,
                    x.max_price,
                );
                total_amount_out = self.add(token_amount_out, total_amount_out);
            }
            assert!(total_amount_out>=min_total_amount_out);
            assert!(to.transfer(self.env().caller(),to.balance_of(self.env().account_id())).is_ok());
            assert!(ti.transfer(self.env().caller(),ti.balance_of(self.env().account_id())).is_ok());
            self._unlocks_();
            total_amount_out
        }


        #[ink(message)]
        pub fn batch_swap_exact_out(
            &mut self,
            swaps: Vec<Swap>,
            token_in: AccountId,
            token_out: AccountId,
            max_total_amount_in: u128,
        ) -> u128 {
            self._logs_();
            self._locks_();
            let mut total_amount_in: u128 = 0;
            let mut ti: PAT = FromAccountId::from_account_id(token_in);
            let mut to: PAT = FromAccountId::from_account_id(token_out);
            assert!(ti.transfer_from(self.env().caller(), self.env().account_id(), max_total_amount_in).is_ok());
            for x in swaps {
                let pool: PoolInterface = FromAccountId::from_account_id(x.pool);
                if ti.allowance(self.env().account_id(), x.pool) < max_total_amount_in {
                    ti.approve(x.pool, u128::MAX);
                }
                let token_amount_in = pool.swap_exact_amount_in(
                    token_in,
                    x.token_in_param,
                    token_out,
                    x.token_out_param,
                    x.max_price,
                );
                total_amount_in = self.add(token_amount_in, total_amount_in);
            }
            assert!(total_amount_in<=max_total_amount_in);
            assert!(to.transfer(self.env().caller(),to.balance_of(self.env().account_id())).is_ok());
            assert!(ti.transfer(self.env().caller(),ti.balance_of(self.env().account_id())).is_ok());
            self._unlocks_();
            total_amount_in
        }

        #[ink(message)]
        pub fn batch_dot_in_swap_exact_in(
            &mut self,
            swaps: Vec<Swap>,
            token_out: AccountId,
            min_total_amount_out: u128,
        ) -> u128 {
            self._logs_();
            self._locks_();
            let mut total_amount_out: u128 = 0;
            let mut to: PAT = FromAccountId::from_account_id(token_out);
            self.cdot.deposit();
            for x in swaps {
                let pool: PoolInterface = FromAccountId::from_account_id(x.pool);
                if self.cdot.allowance(self.env().account_id(), x.pool) < self.env().balance() {
                    self.cdot.approve(x.pool, u128::MAX);
                }
                let token_amount_out = pool.swap_exact_amount_in(
                    self.cdot.to_account_id(),
                    x.token_in_param,
                    token_out,
                    x.token_out_param,
                    x.max_price,
                );
                total_amount_out = self.add(token_amount_out, total_amount_out);
            }
            assert!(total_amount_out >= min_total_amount_out);
            assert!(to.transfer(self.env().caller(), to.balance_of(self.env().account_id())).is_ok());
            let cdot_balance = self.cdot.balance_of(self.env().account_id());
            if cdot_balance > 0 {
                self.cdot.withdraw(cdot_balance);
                // (bool xfer,) = msg.sender.call.value(cdot_balance)("");
                // require(xfer, "ERR_ETH_FAILED");
            }
            self._unlocks_();
            total_amount_out
        }

        #[ink(message)]
        pub fn batch_dot_out_swap_exact_in(
            &mut self,
            swaps: Vec<Swap>,
            token_in: AccountId,
            total_amount_in: u128,
            min_total_amount_out: u128,
        ) -> u128 {
            self._logs_();
            self._locks_();
            let mut total_amount_out: u128 = 0;
            let mut ti: PAT = FromAccountId::from_account_id(token_in);
            assert!(ti.transfer_from(self.env().caller(), self.env().account_id(), total_amount_in).is_ok());
            for x in swaps {
                let pool: PoolInterface = FromAccountId::from_account_id(x.pool);
                if ti.allowance(self.env().account_id(), x.pool) < total_amount_in {
                    ti.approve(x.pool, u128::MAX);
                }
                let token_amount_out = pool.swap_exact_amount_in(
                    token_in,
                    x.token_in_param,
                    self.cdot.to_account_id(),
                    x.token_out_param,
                    x.max_price,
                );
                total_amount_out = self.add(token_amount_out, total_amount_out);
            }
            assert!(total_amount_out >= min_total_amount_out);
            let cdot_balance = self.cdot.balance_of(self.env().account_id());
            self.cdot.withdraw(cdot_balance);
            // (bool xfer,) = msg.sender.call.value(cdot_balance)("");
            // require(xfer, "ERR_ETH_FAILED");
            assert!(ti.transfer(self.env().caller(), ti.balance_of(self.env().account_id())).is_ok());
            self._unlocks_();
            total_amount_out
        }
        #[ink(message)]
        pub fn batch_dot_in_swap_exact_out(
            &mut self,
            swaps: Vec<Swap>,
            token_out: AccountId,
        ) -> u128 {
            self._logs_();
            self._locks_();
            let mut total_amount_in: u128 = 0;
            let mut to: PAT = FromAccountId::from_account_id(token_out);
            self.cdot.deposit();
            for x in swaps {
                let pool: PoolInterface = FromAccountId::from_account_id(x.pool);
                if to.allowance(self.env().account_id(), x.pool) < self.env().balance() {
                    to.approve(x.pool, u128::MAX);
                }
                let token_amount_in = pool.swap_exact_amount_out(
                    self.cdot.to_account_id(),
                    x.token_in_param,
                    token_out,
                    x.token_out_param,
                    x.max_price,
                );
                total_amount_in = self.add(token_amount_in, total_amount_in);
                assert!(to.transfer(self.env().caller(), to.balance_of(self.env().account_id())).is_ok());
                let cdot_balance = self.cdot.balance_of(self.env().account_id());
                if cdot_balance > 0 {
                    self.cdot.withdraw(cdot_balance);
                    // (bool xfer,) = msg.sender.call.value(cdot_balance)("");
                    // assert_eq!(xfer,false);
                }
            }
            self._unlocks_();
            total_amount_in
        }

        #[ink(message)]
        pub fn batch_dot_out_swap_exact_out(
            &mut self,
            swaps: Vec<Swap>,
            token_in: AccountId,
            max_total_amount_in: u128,
        ) -> u128 {
            self._logs_();
            self._locks_();
            let mut total_amount_in: u128 = 0;
            let mut ti: PAT = FromAccountId::from_account_id(token_in);
            assert!(ti.transfer_from(self.env().caller(), self.env().account_id(), max_total_amount_in).is_ok());
            // let swap: Vec<_> = swaps.iter().copied().collect();
            // for x in swap.clone().into_iter() {
            for x in swaps {
                let pool: PoolInterface = FromAccountId::from_account_id(x.pool);
                if ti.allowance(self.env().account_id(), x.pool) < max_total_amount_in {
                    ti.approve(x.pool, u128::MAX);
                }
                let token_amount_in = pool.swap_exact_amount_out(
                    token_in,
                    x.token_in_param,
                    self.cdot.to_account_id(),
                    x.token_out_param,
                    x.max_price,
                );
                total_amount_in = self.add(token_amount_in, total_amount_in);
            }
            assert!(max_total_amount_in <= max_total_amount_in);
            assert!(ti.transfer(self.env().caller(), ti.balance_of(self.env().account_id())).is_ok());
            let cdot_balance = self.cdot.balance_of(self.env().account_id());
            self.cdot.withdraw(cdot_balance);
            // (bool xfer,) = msg.sender.call.value(cdot_balance)("");
            // assert!(xfer);
            self._unlocks_();
            total_amount_in
        }


        //............................................
        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn get(&self) -> bool {
            self._mutex
        }

        fn add(&self, a: u128, b: u128) -> u128 {
            let c = a + b;
            assert!(c >= a);
            c
        }

        fn _logs_(&mut self) {
            // emit LOG_CALL(msg.sig, msg.sender, msg.data);
            let sender = self.env().caller();
            self.env().emit_event(LOGCALL {
                // sig: Default::default(),
                caller: Some(sender),
                // data: Default::default(),
            });
        }

        fn _locks_(&mut self) {
            assert!(self._mutex);
            self._mutex = true;
        }

        fn _unlocks_(&mut self) {
            self._mutex = false;
        }
    }
}
