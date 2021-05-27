#![cfg_attr(not(feature = "std"), no_std)]
use ink_lang as ink;
#[ink::contract]
mod exchangeproxy {
    use cdot::PAT;
    // use poolproxy::PoolInterface;
    use ink_env::call::FromAccountId;
    use ink_env::debug_println;
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
    pub struct Test {
        value : bool,
        // xxx: Lazy<PAT>,
        // yyy: Lazy<PAT>,
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

    impl Test {
        /// Constructor that initializes the `bool` value to `false`.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }
        pub fn default() -> Self { Self::new(Default::default()) }

        #[ink(message)]
        pub fn swap_test(
            &mut self,
            token_in: AccountId,
            token_out: AccountId,
            total_in: u128,
        ) {
            let caller = self.env().caller();
            let this  = self.env().account_id();

            //log print
            self._trans_(token_in,caller,this,total_in);
            self._get_balance_(token_in, this);

            debug_println("………………………… _trans_ 2222  begin……………………………………");
            self._trans_(token_out,this, caller, 2222);
            self._get_balance_(token_in, this);
            debug_println("…………………………… _trans_ 2222 end …………………………………………");

            debug_println("……………………………next …………………………………………");
            self._trans_(token_in,this,caller,888);
            debug_println("FINISH ...............");

        }



        pub fn _trans_(&self, token: AccountId, from: AccountId, to: AccountId, value:Balance ) {
            let message = ink_prelude::format!("1^^^^^^  from is {:?} , value is {:?}, to is {:?}",
                                               from, value, to);
            debug_println(&message);
            let token: PAT = FromAccountId::from_account_id(token);
            let mut token_contract = Lazy::new(token);
            let fer = token_contract.transfer_from(from, to, value).is_ok();
            assert!(fer);
            debug_println("1^^^^^^  _trans_ finish ^^^^^^^^^^^^^^^");
        }
        pub fn _get_balance_(&self, token: AccountId, this : AccountId ) -> Balance {
            let token: PAT = FromAccountId::from_account_id(token);
            let token_contract = Lazy::new(token);
            let this_balance : Balance = token_contract.balance_of(this);
            let message = ink_prelude::format!("this is {:?} , value is {:?}", this, this_balance);
            debug_println(&message);
            this_balance
        }
    }
}
