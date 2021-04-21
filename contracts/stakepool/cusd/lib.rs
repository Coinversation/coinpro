#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod cusd {
    use ink_storage::{
        collections::{
            HashMap as StorageHashMap,
            Vec as StorageVec,
        },
        traits::{PackedLayout, SpreadLayout},
        Lazy,
    };

    use ink_prelude::{
        vec::Vec,
        format,
    };

    use ink_env::call::FromAccountId;
    use core::convert::TryInto;
    use ink_env::debug_println;

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
    pub struct Cusd {
        /// Stores a single `bool` value on the storage.
        value: bool,
        pub records: StorageHashMap<u32, Record>,
        pub tokens: StorageVec<u32>,
    }

    impl Cusd {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self {
                value: init_value,
                records: StorageHashMap::new(),
                tokens: StorageVec::new(),
            }
        }

        #[ink(message)]
        pub fn build_empty_record(&self) -> Record {
            Record {
                bound: false,   // is token bound to pool
                index: 0,   // private
                de_norm: 0,  // denormalized weight
                balance: 0,
            }
        }

        #[ink(message)]
        pub fn get_record(&self, token_id: u32) -> Option<Record> {
            let r = self.build_empty_record();
            let exist = self.records.contains_key(&token_id);
            if !exist {
                return Some(r)
            }

            return Some(self.records.get(&token_id).unwrap().clone());
        }

        #[ink(message)]
        pub fn join_pool(&mut self) {
            self.tokens.push(1);
            self.tokens.push(2);
            self.tokens.push(3);


        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        use super::*;

        use ink_lang as ink;

        #[ink::test]
        fn it_works() {
            let mut cusd = Cusd::new(false);
            // let r = Record {
            //     bound: true,   // is token bound to pool
            //     index: 1,   // private
            //     de_norm: 1,  // denormalized weight
            //     balance: 100,
            // };
            // cusd.records.insert(1, r);
            //
            // let cx = cusd.get_record(2);
            // assert_eq!(cx.unwrap().bound, false);
            //
            // let cx1 = cusd.get_record(1);
            // let r1 = cx1.unwrap();
            // assert_eq!(r1.bound, true);
            // assert_eq!(r1.index, 1);
            // assert_eq!(r1.balance, 100);
            // assert_eq!(r1.de_norm, 1);

            cusd.join_pool();

            let mut vec = Vec::new();
            vec.push(2);
            vec.push(3);
            vec.push(4);

            println!("Element at position");
            for (pos, e) in cusd.tokens.iter().enumerate() {
                let message = format!("Element at position {}: {:?}", pos, e);
                debug_println(&message);

                assert_eq!(vec[pos], pos + 1);


                println!("Element at position {}: {:?}", pos, e);
            }
        }
    }
}
