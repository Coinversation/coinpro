#![cfg_attr(not(feature = "std"), no_std)]

pub use self::token::Token;
use ink_lang as ink;

#[ink::contract]
mod token {
    use ink_prelude::string::String;
    use math::Math;

    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_storage::{
        collections::HashMap as StorageHashMap,
        Lazy
    };

    use ink_env::call::FromAccountId;
    use ink_prelude::string::ToString;

    #[ink(storage)]
    pub struct Token {
        math: Lazy<Math>,
        total_supply: u128,
        balances: StorageHashMap<AccountId, u128>,
        allowances: StorageHashMap<(AccountId, AccountId), u128>,
        name: Option<String>,
        symbol: Option<String>,
        decimals: Option<u8>,
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        value: u128,
    }

    /// Event emitted when an approval occurs that `spender` is allowed to withdraw
    /// up to the amount of `value` tokens from `owner`.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        #[ink(topic)]
        value: u128,
    }

    impl Token {
        #[ink(constructor)]
        pub fn new(math_address: AccountId) -> Self {
            let math: Math = FromAccountId::from_account_id(math_address);
            Self {
                math: Lazy::new(math),
                total_supply: 0,
                balances: StorageHashMap::new(),
                allowances: StorageHashMap::new(),

                name: Some("Conversation Pool Token".to_string()),
                symbol: Some("CPT".to_string()),
                decimals: Some(10),
            }
        }

        #[ink(message)]
        pub fn mint(&mut self, amt:u128) {
            let from = self.env().caller();
            let balance = self.balance_of(from);
            let balance = self.math.badd(balance, amt);
            self.balances.insert(from, balance);
            self.total_supply = self.math.badd(self.total_supply, amt);

            self.env().emit_event(Transfer {
                from: None,
                to: Some(from),
                value: amt,
            });
        }

        #[ink(message)]
        pub fn burn(&mut self, amt:u128) {
            let from = self.env().caller();
            let balance = self.balance_of(from);
            assert!(balance >= amt, "ERR_INSUFFICIENT_BAL");
            let balance = self.math.bsub(balance, amt);
            self.balances.insert(from, balance);
            self.total_supply = self.math.bsub(self.total_supply, amt);
            self.env().emit_event(Transfer {
                from: Some(from),
                to: None,
                value: amt,
            });
        }

        #[ink(message)]
        pub fn trans(&mut self, from:AccountId, to:AccountId, amt:u128) {
            let from_balance = self.balance_of(from);
            assert!(from_balance >= amt, "ERR_INSUFFICIENT_BAL");
            let from_balance = self.math.bsub(from_balance, amt);
            self.balances.insert(from, from_balance);

            let to_balance = self.balance_of(to);
            let to_balance = self.math.badd(to_balance, amt);
            self.balances.insert(to, to_balance);

            self.env().emit_event(Transfer {
                from: Some(from),
                to: Some(to),
                value: amt,
            });
        }

        #[ink(message)]
        pub fn push(&mut self, to : AccountId, amt:u128) {
            let from = self.env().caller();
            self.trans(from, to, amt);
        }

        #[ink(message)]
        pub fn pull(&mut self, from : AccountId, amt:u128) {
            let to = self.env().caller();
            self.trans(from, to, amt);
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> u128 {
            self.balances.get(&owner).copied().unwrap_or(0)
        }

        #[ink(message)]
        pub fn name(&self) -> Option<String> {
            self.name.clone()
        }

        #[ink(message)]
        pub fn symbol(&self) -> Option<String> {
            self.symbol.clone()
        }

        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> u128 {
            self.allowances.get(&(owner, spender)).copied().unwrap_or(0)
        }

        #[ink(message)]
        pub fn decimals(&self) -> Option<u8> {
            self.decimals
        }

        #[ink(message)]
        pub fn total_supply(&self) -> u128 {
            self.total_supply
        }

        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: u128)  {
            let owner = self.env().caller();
            self.allowances.insert((owner, spender), value);
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });
        }

        #[ink(message)]
        pub fn increase_approval(&mut self, spender: AccountId, value: u128) {
            let owner = self.env().caller();
            let balance = self.allowance(owner, spender);
            let balance = self.math.badd(balance, value);
            self.allowances.insert((owner, spender), balance);
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });
        }

        #[ink(message)]
        pub fn decrease_approval(&mut self, spender: AccountId, value: u128) {
            let owner = self.env().caller();
            let old_value = self.allowance(owner, spender);
            if value > old_value {
                self.allowances.insert((owner, spender), 0);
            } else {
                let new_value = self.math.bsub(old_value, value);
                self.allowances.insert((owner, spender), new_value);
            }
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: u128) -> bool {
            let owner = self.env().caller();
            self.trans(owner, to, value);
            return true;
        }

        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, value: u128) -> bool {
            let owner = self.env().caller();
            let allow = self.allowance(from, owner);
            assert!(owner == from || value < allow, "ERR_BTOKEN_BAD_CALLER");
            self.trans(from, to, value);

            if owner != from && allow != u128::MAX {
                let balance = self.math.bsub(allow, value);
                self.allowances.insert((from, owner), balance);
                self.env().emit_event(Approval {
                    owner,
                    spender: to,
                    value: balance,
                });
            }
            return true;
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[test]
        fn default_works() {
        }

        /// We test a simple use case of our contract.
        #[test]
        fn it_works() {
        }
    }
}
