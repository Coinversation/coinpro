#![cfg_attr(not(feature = "std"), no_std)]

pub use self::pool::Pool;
use ink_lang as ink;

#[ink::contract]
mod pool {
    use ink_prelude::vec::Vec;
    use ink_storage::{
        collections::{
            HashMap as StorageHashMap,
            Vec as StorageVec,
        },
        traits::{PackedLayout, SpreadLayout},
        Lazy,
    };
    use math::Math;
    use math::{
        EXIT_FEE,
        MIN_FEE,
        MAX_FEE,
        MIN_BOUND_TOKENS,
        MAX_BOUND_TOKENS,
        INIT_POOL_SUPPLY,
        MIN_WEIGHT,
        MAX_WEIGHT,
        MIN_BALANCE,
        MAX_TOTAL_WEIGHT,
        MAX_OUT_RATIO,
        MAX_IN_RATIO,
    };
    use base::Base;
    use token::Token;

    use ink_env::call::FromAccountId;
    use core::convert::TryInto;

    #[derive(
    Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode, SpreadLayout, PackedLayout,
    )]
    #[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct Record {
        pub bound: bool,   // is token bound to pool
        pub index: u128,   // private
        pub de_norm: u128,  // denormalized weight
        pub balance: u128,
    }

    #[ink(storage)]
    pub struct Pool {
        mutex: bool,
        factory: AccountId,
        controller: AccountId,
        public_swap: bool,
        swap_fee: u128,
        finalized: bool,
        tokens: StorageVec<AccountId>,
        records: StorageHashMap<AccountId, Record>,
        total_weight: u128,

        base: Lazy<Base>,
        math: Lazy<Math>,
        token:  Lazy<Token>,
    }

    #[ink(event)]
    pub struct LogSwap {
        #[ink(topic)]
        caller: Option<AccountId>,
        #[ink(topic)]
        token_in: Option<AccountId>,
        #[ink(topic)]
        token_out: Option<AccountId>,
        token_amount_in: u128,
        token_amount_out: u128,
    }

    #[ink(event)]
    pub struct LogJoin {
        #[ink(topic)]
        caller: Option<AccountId>,
        #[ink(topic)]
        token_in: Option<AccountId>,
        token_amount_in: u128,
    }

    #[ink(event)]
    pub struct LogExit {
        #[ink(topic)]
        caller: Option<AccountId>,
        #[ink(topic)]
        token_out: Option<AccountId>,
        token_amount_out: u128,
    }

    #[ink(event, anonymous)]
    pub struct LogCall {
        #[ink(topic)]
        sig: [u8; 4],
        #[ink(topic)]
        caller: Option<AccountId>,
        data: Vec<u8>,
    }

    impl Pool {
        #[ink(constructor)]
        pub fn new(math_address: AccountId,
                   base_address: AccountId,
                   token_address:  AccountId) -> Self {
            let caller = Self::env().caller();

            let base: Base = FromAccountId::from_account_id(base_address);
            let math: Math = FromAccountId::from_account_id(math_address);
            let token: Token = FromAccountId::from_account_id(token_address);

            let instance = Self {
                mutex: false,
                factory: caller,
                controller: caller,
                public_swap: false,
                swap_fee: MIN_FEE,
                finalized: false,
                tokens: StorageVec::new(),
                records: StorageHashMap::new(),
                total_weight: 0,

                base: Lazy::new(base),
                math: Lazy::new(math),
                token: Lazy::new(token),
            };
            instance
        }

        fn _logs_(&self) {
            let caller = Self::env().caller();
            self.env().emit_event(LogCall {
                // @todo lipu: fix this field
                sig: [0x00; 4],
                caller: Some(caller),
                // @todo lipu: fix this field
                data: [0x00; 4].to_vec(),
            });
        }

        fn _lock_(&mut self) {
            assert!(!self.mutex, "ERR_REENTRY");
            self.mutex = true;
        }

        fn _unlock_(&mut self) {
            self.mutex = false;
        }

        fn _view_lock_(&self) {
            assert!(!self.mutex, "ERR_REENTRY");
        }

        fn _get_sender(&self) -> AccountId {
            let sender = Self::env().caller();
            return sender;
        }

        fn _get_sender_and_this(&self) -> (AccountId, AccountId) {
            let sender = self._get_sender();
            let this = self.env().account_id();
            return (sender, this);
        }

        pub fn _pull_underlying(&self, erc20: AccountId, from: AccountId, to: AccountId, amount: u128) {
            let mut token: Token = FromAccountId::from_account_id(erc20);
            let fer = token.transfer_from(from, to, amount);
            assert!(fer);
        }

        pub fn _push_underlying(&self, erc20: AccountId, to: AccountId, amount: u128) {
            let mut token: Token = FromAccountId::from_account_id(erc20);
            let fer = token.transfer(to, amount);
            assert!(fer);
        }

        fn _pull_pool_share(&mut self, from: AccountId, amount: u128) {
            self.token.pull(from, amount);
        }

        fn _push_pool_share(&mut self, to: AccountId, amount: u128) {
            self.token.push(to, amount);
        }

        fn _mint_pool_share(&mut self, amount: u128) {
            self.token.mint(amount);
        }

        fn _burn_pool_share(&mut self, amount: u128) {
            self.token.burn(amount);
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> u128 {
            return self.token.balance_of(owner);
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: u128) -> bool {
            return self.token.transfer(to, value);
        }

        #[ink(message)]
        pub fn is_public_swap(&self) -> bool {
            return self.public_swap;
        }

        #[ink(message)]
        pub fn is_finalized(&self) -> bool {
            return self.finalized;
        }

        #[ink(message)]
        pub fn is_bound(&self, t: AccountId) -> bool {
            return self.records[&t].bound;
        }

        #[ink(message)]
        pub fn get_num_tokens(&self) -> u128 {
            return self.tokens.len().into();
        }

        #[ink(message)]
        pub fn get_current_tokens(&self) -> Vec<AccountId> {
            self._view_lock_();
            let ts: Vec<_> = self.tokens.iter().copied().collect();
            return ts;
        }

        #[ink(message)]
        pub fn get_final_tokens(&self) -> Vec<AccountId> {
            self._view_lock_();
            assert!(self.finalized, "ERR_NOT_FINALIZED");
            let ts: Vec<_> = self.tokens.iter().copied().collect();
            return ts;
        }

        #[ink(message)]
        pub fn get_denormalized_weight(&self, token: AccountId) -> u128 {
            self._view_lock_();
            assert!(self.records[&token].bound, "ERR_NOT_BOUND");
            return self.records[&token].de_norm;
        }

        #[ink(message)]
        pub fn get_total_denormalized_weight(&self) -> u128 {
            self._view_lock_();
            return self.total_weight;
        }

        #[ink(message)]
        pub fn get_normalized_weight(&self, token: AccountId) -> u128 {
            self._view_lock_();
            assert!(self.records[&token].bound, "ERR_NOT_BOUND");
            let denorm: u128 = self.records[&token].de_norm;
            let norm_weight: u128 = self.math.bdiv(denorm, self.total_weight);
            return norm_weight;
        }

        #[ink(message)]
        pub fn get_balance(&self, token: AccountId) -> u128 {
            self._view_lock_();
            assert!(self.records[&token].bound, "ERR_NOT_BOUND");
            return self.records[&token].balance;
        }

        #[ink(message)]
        pub fn get_swap_fee(&self) -> u128 {
            self._view_lock_();
            return self.swap_fee;
        }

        #[ink(message)]
        pub fn get_controller(&self) -> AccountId {
            self._view_lock_();
            return self.controller;
        }

        #[ink(message)]
        pub fn set_swap_fee(&mut self, fee:u128) {
            self._logs_();
            self._view_lock_();

            assert!(!self.finalized, "ERR_IS_FINALIZED");
            assert_eq!(self.controller, self._get_sender(), "ERR_NOT_CONTROLLER");
            assert!(self.swap_fee >= MIN_FEE, "ERR_MIN_FEE");
            assert!(self.swap_fee <= MAX_FEE, "ERR_MAX_FEE");

            self.swap_fee = fee;
        }

        #[ink(message)]
        pub fn set_controller(&mut self, manager:AccountId) {
            self._logs_();
            self._lock_();
            assert!(self.controller == self._get_sender(), "ERR_NOT_CONTROLLER");
            self.controller = manager;
            self._unlock_();
        }

        #[ink(message)]
        pub fn set_public_swap(&mut self, public:bool) {
            self._logs_();
            self._lock_();
            assert!(!self.finalized, "ERR_IS_FINALIZED");
            assert!(self.controller == self._get_sender(), "ERR_NOT_CONTROLLER");
            self.public_swap = public;
            self._unlock_();
        }

        #[ink(message)]
        pub fn finalize(&mut self) {
            self._logs_();
            self._lock_();
            let sender = self._get_sender();
            assert!(self.controller == sender, "ERR_NOT_CONTROLLER");
            assert!(!self.finalized, "ERR_IS_FINALIZED");
            assert!(u128::from(self.tokens.len()) >= MIN_BOUND_TOKENS, "ERR_MIN_TOKENS");
            self.finalized = true;
            self.public_swap = true;
            self._mint_pool_share(INIT_POOL_SUPPLY);
            self._push_pool_share(sender, INIT_POOL_SUPPLY);
            self._unlock_();
        }

        #[ink(message)]
        pub fn bind(&mut self, token: AccountId, balance: u128, denorm:u128) {
            self._logs_();
            assert!(self.controller == self._get_sender(), "ERR_NOT_CONTROLLER");
            assert!(!self.records[&token].bound, "ERR_IS_BOUND");
            assert!(!self.finalized, "ERR_IS_FINALIZED");
            assert!(u128::from(self.tokens.len()) < MAX_BOUND_TOKENS, "ERR_MAX_TOKENS");
            let r = Record {
                bound: true,
                index: self.tokens.len().into(),
                de_norm: 0,    // balance and denorm will be validated
                balance: 0   // and set by `rebind`
            };
            self.records.insert(token, r);
            self.tokens.push(token);

            self.rebind(token, balance, denorm);
        }

        fn _require_bound_finalized_controller(&self, token: AccountId) {
            let sender = self._get_sender();
            assert!(self.controller == sender, "ERR_NOT_CONTROLLER");
            assert!(!self.records[&token].bound, "ERR_NOT_BOUND");
            assert!(!self.finalized, "ERR_IS_FINALIZED");
        }

        fn _update_balance(&mut self, token: AccountId, balance: u128) {
            if let Some(record) = self.records.get_mut(&token) {
                record.balance = balance;
            }
        }

        #[ink(message)]
        pub fn rebind(&mut self, token: AccountId, balance: u128, denorm:u128) {
            self._logs_();
            self._lock_();

            let (sender, this) = self._get_sender_and_this();

            self._require_bound_finalized_controller(token);

            assert!(denorm >= MIN_WEIGHT, "ERR_MIN_WEIGHT");
            assert!(denorm <= MAX_WEIGHT, "ERR_MAX_WEIGHT");
            assert!(balance >= MIN_BALANCE, "ERR_MIN_BALANCE");

            // Adjust the denorm and totalWeight
            let old_weight = self.records[&token].de_norm;
            if denorm > old_weight {
                self.total_weight = self.math.badd(self.total_weight, self.math.bsub(denorm, old_weight));
                assert!(self.total_weight <= MAX_TOTAL_WEIGHT, "ERR_MAX_TOTAL_WEIGHT");
            } else if denorm < old_weight {
                self.total_weight = self.math.bsub(self.total_weight, self.math.bsub(old_weight, denorm));
            }

            // Adjust the balance record and actual token balance
            let old_balance = self.records[&token].balance;

            if let Some(record) = self.records.get_mut(&token) {
                record.balance = balance;
                record.de_norm = denorm;
            }

            if balance > old_balance {
                self._pull_underlying(token, sender, this, self.math.bsub(balance, old_balance));
            } else if balance < old_balance {
                // In this case liquidity is being withdrawn, so charge EXIT_FEE
                let token_balance_withdrawn = self.math.bsub(old_balance, balance);
                let token_exit_fee = self.math.bmul(token_balance_withdrawn, EXIT_FEE);
                self._push_underlying(token, sender, self.math.bsub(token_balance_withdrawn, token_exit_fee));
                self._push_underlying(token, self.factory, token_exit_fee);
            }
            self._unlock_();
        }

        #[ink(message)]
        pub fn unbind(&mut self, token: AccountId) {
            self._logs_();
            self._lock_();

            let sender = self._get_sender();

            self._require_bound_finalized_controller(token);

            let token_balance = self.records[&token].balance;
            let token_exit_fee = self.math.bmul(token_balance, EXIT_FEE);

            self.total_weight = self.math.bsub(self.total_weight, self.records[&token].de_norm);

            // Swap the token-to-unbind with the last token,
            // then delete the last token
            let index = self.records[&token].index;
            let last = self.tokens.len() - 1;
            self.tokens[index.try_into().unwrap()] = self.tokens[last];
            self.records[&self.tokens[index.try_into().unwrap()]].index = index;
            self.tokens.pop();

            let r = Record{
                bound: false,
                index: 0,
                de_norm: 0,
                balance: 0
            };

            self.records.insert(token, r);

            self._push_underlying(token, sender, self.math.bsub(token_balance, token_exit_fee));
            self._push_underlying(token, self.factory, token_exit_fee);
            self._unlock_();
        }

        // Absorb any tokens that have been sent to this contract into the pool
        #[ink(message)]
        pub fn gulp(&mut self, token: AccountId) {
            self._logs_();
            self._lock_();
            assert!(!self.records[&token].bound, "ERR_NOT_BOUND");
            let balance = self.token.balance_of(token);
            self._update_balance(token, balance);
            self._unlock_();
        }

        fn require_valid_bound(&self, token_in: AccountId, token_out: AccountId) {
            assert!(self.records[&token_in].bound, "ERR_NOT_BOUND");
            assert!(self.records[&token_out].bound, "ERR_NOT_BOUND");
        }

        #[ink(message)]
        pub fn get_spot_price(&mut self, token_in: AccountId, token_out: AccountId) -> u128 {
            self._view_lock_();
            self.require_valid_bound(token_in, token_out);

            let in_record = &self.records[&token_in];
            let out_record = &self.records[&token_out];
            return self.base.calc_spot_price(in_record.balance, in_record.de_norm,
                                             out_record.balance, out_record.de_norm,
                                             self.swap_fee);
        }

        #[ink(message)]
        pub fn get_spot_price_sans_fee(&mut self, token_in: AccountId, token_out: AccountId) -> u128 {
            self._view_lock_();
            self.require_valid_bound(token_in, token_out);
            let in_record = &self.records[&token_in];
            let out_record = &self.records[&token_out];
            return self.base.calc_spot_price(in_record.balance, in_record.de_norm,
                                             out_record.balance, out_record.de_norm, 0);
        }

        #[ink(message)]
        pub fn join_pool(&mut self, pool_amount_out: u128, max_amounts_in: Vec<u128>) {
            self._logs_();
            self._lock_();
            assert!(self.finalized, "ERR_NOT_FINALIZED");

            let pool_total = self.token.total_supply();
            let ratio = self.math.bdiv(pool_amount_out, pool_total);
            assert!(ratio != 0, "ERR_MATH_APPROX");

            let (sender, this) = self._get_sender_and_this();

            let mut i = 0;
            while i < self.tokens.len() {
                let t = self.tokens[i];
                let bal = self.records[&t].balance;
                let token_amount_in = self.math.bmul(ratio, bal);
                assert!(token_amount_in != 0, "ERR_MATH_APPROX");
                // @todo fix max_amounts_in[i]
                //assert!(token_amount_in <= max_amounts_in[i]);
                let mut balance = self.records[&t].balance;
                balance = self.math.badd(balance, token_amount_in);
                self._update_balance(t, balance);
                self.env().emit_event(LogJoin {
                    caller: Some(sender),
                    token_in: Some(t),
                    token_amount_in,
                });
                self._pull_underlying(t, sender, this, token_amount_in);

                i += 1;
            }


            self._mint_pool_share(pool_amount_out);
            self._push_pool_share(sender, pool_amount_out);

            self._unlock_();
        }

        #[ink(message)]
        pub fn exit_pool(&mut self, pool_amount_in: u128, min_amounts_out: Vec<u128>) {
            self._logs_();
            self._lock_();
            assert!(self.finalized, "ERR_NOT_FINALIZED");

            let pool_total = self.token.total_supply();
            let exit_fee = self.math.bmul(pool_amount_in, EXIT_FEE);
            let pai_after_exit_fee = self.math.bsub(pool_amount_in, exit_fee);
            let ratio = self.math.bdiv(pai_after_exit_fee, pool_total);
            assert!(ratio != 0, "ERR_MATH_APPROX");

            let sender = self._get_sender();

            self._pull_pool_share(sender, pool_amount_in);
            self._push_pool_share(self.factory, exit_fee);
            self._burn_pool_share(pai_after_exit_fee);

            let mut i = 0;
            while i < self.tokens.len() {
                let t = self.tokens[i];
                let bal = self.records[&t].balance;
                let token_amount_out = self.math.bmul(ratio, bal);
                assert!(token_amount_out != 0, "ERR_MATH_APPROX");
                // @todo fix min_amounts_out[i]
                //assert!(token_amount_out >= min_amounts_out[i]);
                let mut balance = self.records[&t].balance;
                balance = self.math.bsub(balance, token_amount_out);
                self._update_balance(t, balance);
                self.env().emit_event(LogExit {
                    caller: Some(sender),
                    token_out: Some(t),
                    token_amount_out,
                });
                self._push_underlying(t, sender, token_amount_out);

                i += 1;
            }
            self._unlock_();
        }

        fn require_valid_bound_swap(&self, token_in: AccountId, token_out: AccountId) {
            self.require_valid_bound(token_in, token_out);
            assert!(self.public_swap, "ERR_SWAP_NOT_PUBLIC");
        }

        #[ink(message)]
        pub fn swap_exact_amount_in(&mut self,
                                    token_in: AccountId,
                                    token_amount_in: u128,
                                    token_out: AccountId,
                                    min_amount_out: u128,
                                    max_price: u128) ->(u128, u128) {
            self._logs_();
            self._lock_();
            self.require_valid_bound_swap(token_in, token_out);

            // @todo fix storage
            let in_record_balance = self.records[&token_in].balance;
            let in_record_de_norm = self.records[&token_in].de_norm;

            let out_record_balance = self.records[&token_out].balance;
            let out_record_de_norm = self.records[&token_out].de_norm;

            assert!(token_amount_in <= self.math.bmul(in_record_balance, MAX_IN_RATIO), "ERR_MAX_IN_RATIO");

            let spot_price_before = self.base.calc_spot_price(in_record_balance,
                                                              in_record_de_norm,
                                                              out_record_balance,
                                                              out_record_de_norm,
                                                              self.swap_fee);
            assert!(spot_price_before <= max_price, "ERR_BAD_LIMIT_PRICE");

            let token_amount_out = self.base.calc_out_given_in(in_record_balance,
                                                               in_record_de_norm,
                                                               out_record_balance,
                                                               out_record_de_norm,
                                                               token_amount_in,
                                                               self.swap_fee);
            assert!(token_amount_out >= min_amount_out, "ERR_LIMIT_OUT");

            let new_in_balance = self.math.badd(in_record_balance, token_amount_in);
            let new_out_balance = self.math.badd(out_record_balance, token_amount_out);

            let spot_price_after = self.base.calc_spot_price(new_in_balance,
                                                             in_record_de_norm,
                                                             new_out_balance,
                                                             out_record_de_norm,
                                                             self.swap_fee);

            assert!(spot_price_after >= spot_price_before, "ERR_MATH_APPROX");
            assert!(spot_price_after <= max_price, "ERR_LIMIT_PRICE");
            assert!(spot_price_before <= self.math.bdiv(token_amount_in, token_amount_out), "ERR_MATH_APPROX");

            self._update_balance(token_in, new_in_balance);
            self._update_balance(token_out, new_out_balance);
            let (sender, this) = self._get_sender_and_this();

            self.env().emit_event(LogSwap {
                caller: Some(sender),
                token_in: Some(token_in),
                token_out: Some(token_out),
                token_amount_in,
                token_amount_out,
            });

            self._pull_underlying(token_in, sender, this, token_amount_in);
            self._push_underlying(token_out, sender, token_amount_out);
            self._unlock_();

            return (token_amount_out, spot_price_after);
        }

        #[ink(message)]
        pub fn swap_exact_amount_out(&mut self,
                                     token_in: AccountId,
                                     max_amount_in: u128,
                                     token_out: AccountId,
                                     token_amount_out: u128,
                                     max_price: u128) ->(u128, u128) {
            self._logs_();
            self._lock_();
            self.require_valid_bound_swap(token_in, token_out);

            // @todo fix storage
            let in_record_balance = self.records[&token_in].balance;
            let in_record_de_norm = self.records[&token_in].de_norm;

            let out_record_balance = self.records[&token_in].balance;
            let out_record_de_norm = self.records[&token_in].de_norm;

            assert!(token_amount_out <= self.math.bmul(out_record_balance, MAX_OUT_RATIO), "ERR_MAX_OUT_RATIO");

            let spot_price_before = self.base.calc_spot_price(in_record_balance,
                                                              in_record_de_norm,
                                                              out_record_balance,
                                                              out_record_de_norm,
                                                              self.swap_fee);

            assert!(spot_price_before <= max_price, "ERR_BAD_LIMIT_PRICE");

            let token_amount_in = self.base.calc_in_given_out(in_record_balance,
                                                              in_record_de_norm,
                                                              out_record_balance,
                                                              out_record_de_norm,
                                                              token_amount_out,
                                                              self.swap_fee);

            assert!(token_amount_in <= max_amount_in, "ERR_LIMIT_IN");

            let new_in_record_balance = self.math.badd(in_record_balance, token_amount_in);
            let new_out_record_balance = self.math.bsub(out_record_balance, token_amount_out);

            let spot_price_after = self.base.calc_spot_price(new_in_record_balance,
                                                             in_record_de_norm,
                                                             new_out_record_balance,
                                                             out_record_de_norm,
                                                             self.swap_fee);

            assert!(spot_price_after >= spot_price_before, "ERR_MATH_APPROX");
            assert!(spot_price_after <= max_price, "ERR_LIMIT_PRICE");
            assert!(spot_price_before <= self.math.bdiv(token_amount_in, token_amount_out), "ERR_MATH_APPROX");

            self._update_balance(token_in, new_in_record_balance);
            self._update_balance(token_out, new_out_record_balance);

            let (sender, this) = self._get_sender_and_this();

            self.env().emit_event(LogSwap {
                caller: Some(sender),
                token_in: Some(token_in),
                token_out: Some(token_out),
                token_amount_in,
                token_amount_out,
            });

            self._pull_underlying(token_in, sender, this, token_amount_in);
            self._push_underlying(token_out, sender, token_amount_out);

            self._unlock_();
            return (token_amount_in, spot_price_after);
        }

        fn require_finalize_bound(&self, token_in: AccountId) {
            assert!(self.finalized, "ERR_NOT_FINALIZED");
            assert!(self.records[&token_in].bound, "ERR_NOT_BOUND");
        }

        #[ink(message)]
        pub fn join_swap_extern_amount_in(&mut self,
                                          token_in: AccountId,
                                          token_amount_in: u128,
                                          min_pool_amount_out: u128) -> u128 {
            self._logs_();
            self._lock_();
            self.require_finalize_bound(token_in);
            assert!(token_amount_in <= self.math.bmul(self.records[&token_in].balance, MAX_IN_RATIO), "ERR_MAX_IN_RATIO");

            // @todo fix storage
            let in_record_balance = self.records[&token_in].balance;
            let in_record_de_norm = self.records[&token_in].de_norm;

            let total_supply = self.token.total_supply();
            let pool_amount_out = self.base.calc_pool_out_given_single_in(in_record_balance,
                                                                          in_record_de_norm,
                                                                          total_supply,
                                                                          self.total_weight,
                                                                          token_amount_in,
                                                                          self.swap_fee);

            assert!(pool_amount_out >= min_pool_amount_out, "ERR_LIMIT_OUT");
            self._update_balance(token_in, self.math.badd(in_record_balance, token_amount_in));
            let (sender, this) = self._get_sender_and_this();

            self.env().emit_event(LogJoin {
                caller: Some(sender),
                token_in: Some(token_in),
                token_amount_in
            });
            self._mint_pool_share(pool_amount_out);
            self._push_pool_share(sender, pool_amount_out);
            self._pull_underlying(token_in, sender, this, token_amount_in);
            self._unlock_();

            return pool_amount_out;
        }

        #[ink(message)]
        pub fn join_swap_pool_amount_out(&mut self,
                                         token_in: AccountId,
                                         pool_amount_out: u128,
                                         max_amount_in: u128) -> u128 {
            self._logs_();
            self._lock_();
            self.require_finalize_bound(token_in);
            let in_record_balance = self.records[&token_in].balance;
            let in_record_de_norm = self.records[&token_in].de_norm;

            let total_supply = self.token.total_supply();
            let token_amount_in = self.base.calc_single_in_given_pool_out(in_record_balance,
                                                                          in_record_de_norm,
                                                                          total_supply,
                                                                          self.total_weight,
                                                                          pool_amount_out,
                                                                          self.swap_fee);

            assert!(token_amount_in != 0, "ERR_MATH_APPROX");
            assert!(token_amount_in <= max_amount_in, "ERR_LIMIT_IN");

            assert!(token_amount_in <= self.math.bmul(in_record_balance, MAX_IN_RATIO), "ERR_MAX_IN_RATIO");
            self._update_balance(token_in, self.math.badd(in_record_balance, token_amount_in));
            let (sender, this) = self._get_sender_and_this();

            self.env().emit_event(LogJoin {
                caller: Some(sender),
                token_in: Some(token_in),
                token_amount_in
            });
            self._mint_pool_share(pool_amount_out);
            self._push_pool_share(sender, pool_amount_out);
            self._pull_underlying(token_in, sender, this, token_amount_in);

            self._unlock_();

            return token_amount_in;
        }

        #[ink(message)]
        pub fn exit_swap_pool_amount_in(&mut self,
                                        token_out: AccountId,
                                        pool_amount_in: u128,
                                        min_amount_out: u128) -> u128 {
            self._logs_();
            self._lock_();

            self.require_finalize_bound(token_out);
            // @todo fix storage
            let out_record_balance = self.records[&token_out].balance;
            let out_record_de_norm = self.records[&token_out].de_norm;

            let total_supply = self.token.total_supply();

            let token_amount_out = self.base.calc_single_out_given_pool_in(
                out_record_balance,
                out_record_de_norm,
                total_supply,
                self.total_weight,
                pool_amount_in,
                self.swap_fee);

            assert!(token_amount_out >= min_amount_out, "ERR_LIMIT_OUT");
            assert!(token_amount_out <= self.math.bmul(out_record_balance, MAX_OUT_RATIO), "ERR_MAX_OUT_RATIO");

            self._update_balance(token_out, self.math.bsub(out_record_balance, token_amount_out));
            let exit_fee = self.math.bmul(pool_amount_in, EXIT_FEE);

            let sender = self._get_sender();

            self.env().emit_event(LogExit {
                caller: Some(sender),
                token_out: Some(token_out),
                token_amount_out,
            });
            self._pull_pool_share(sender, pool_amount_in);
            self._burn_pool_share(self.math.bsub(pool_amount_in, exit_fee));
            self._push_pool_share(self.factory, exit_fee);
            self._push_underlying(token_out, sender, token_amount_out);

            self._unlock_();
            return token_amount_out;
        }

        #[ink(message)]
        pub fn exit_swap_extern_amount_out(&mut self,
                                           token_out: AccountId,
                                           token_amount_out: u128,
                                           max_pool_amount_in: u128) -> u128 {
            self._logs_();
            self._lock_();
            self.require_finalize_bound(token_out);
            assert!(token_amount_out <= self.math.bmul(self.records[&token_out].balance, MAX_OUT_RATIO), "ERR_MAX_OUT_RATIO");

            // @todo fix storage
            let out_record_balance = self.records[&token_out].balance;
            let out_record_de_norm = self.records[&token_out].de_norm;

            let total_supply = self.token.total_supply();
            let pool_amount_in = self.base.calc_pool_in_given_single_out(
                out_record_balance,
                out_record_de_norm,
                total_supply,
                self.total_weight,
                token_amount_out,
                self.swap_fee
            );

            assert!(pool_amount_in != 0, "ERR_MATH_APPROX");
            assert!(pool_amount_in <= max_pool_amount_in, "ERR_LIMIT_IN");

            self._update_balance(token_out, self.math.bsub(out_record_balance, token_amount_out));
            let sender = self._get_sender();

            let exit_fee = self.math.bmul(pool_amount_in, EXIT_FEE);
            self.env().emit_event(LogExit {
                caller: Some(sender),
                token_out: Some(token_out),
                token_amount_out,
            });
            self._pull_pool_share(sender, pool_amount_in);
            self._burn_pool_share(self.math.bsub(pool_amount_in, exit_fee));
            self._push_pool_share(self.factory, exit_fee);
            self._push_underlying(token_out, sender, token_amount_out);
            self._unlock_();

            return pool_amount_in;
        }
    }
}
