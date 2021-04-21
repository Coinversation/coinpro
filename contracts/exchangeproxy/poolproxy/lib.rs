#![cfg_attr(not(feature = "std"), no_std)]
pub use self::poolproxy::PoolInterface;
use ink_lang as ink;

#[ink::contract]
mod poolproxy {
    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct PoolInterface {
        /// Stores a single `bool` value on the storage.
        value: bool,
    }

    impl PoolInterface {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        #[ink(message)]
        pub fn swap_exact_amount_in(&self,
                                    token_in: AccountId,
                                    token_amount_in: u128,
                                    token_out: AccountId,
                                    min_amount_out: u128,
                                    max_price: u128,
        ) -> u128 {unimplemented!()}

        #[ink(message)]
        pub fn swap_exact_amount_out(&self,
                                     token_in: AccountId,
                                     token_amount_in: u128,
                                     token_out: AccountId,
                                     min_amount_out: u128,
                                     max_price: u128,
        ) -> u128 {unimplemented!()}
        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn flip(&mut self) {
            self.value = !self.value;
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }
    }
}
