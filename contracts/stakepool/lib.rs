#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod stakepool {
    use cusd::Pat;
    use ink_env::call::FromAccountId;
    use ink_env::debug_println;
    use ink_prelude::vec::Vec;
    use ink_storage::{
        collections::HashMap as StorageMap,
        traits::{PackedLayout, SpreadLayout},
        Lazy,
    };
    use ownership::Ownable;
    use primitive_types::U256;

    //c_id  abbreviation for instance of  CollateralId
    pub type CollateralId = u32;
    pub type USD = u32;

    // pub const PATS: Balance = 10_000_000_000;
    pub const PAT_PRICE_DECIMALS: u32 = 100;

    #[ink(event)]
    pub struct IssueCusd {
        #[ink(topic)]
        c_id: CollateralId,
        #[ink(topic)]
        collateral: Balance,
        #[ink(topic)]
        cusd: Balance,
    }

    #[ink(event)]
    pub struct AddCollateral {
        #[ink(topic)]
        c_id: CollateralId,
        #[ink(topic)]
        add_collateral: Balance,
        #[ink(topic)]
        collateral_ratio: u32,
    }

    #[ink(event)]
    pub struct MinusCollateral {
        #[ink(topic)]
        c_id: CollateralId,
        #[ink(topic)]
        minus_collateral: Balance,
        #[ink(topic)]
        collateral_ratio: u32,
    }

    #[ink(event)]
    pub struct Withdraw {
        #[ink(topic)]
        c_id: CollateralId,
        #[ink(topic)]
        collateral: Balance,
        #[ink(topic)]
        cusd: Balance,
    }

    #[ink(event)]
    pub struct Liquidate {
        #[ink(topic)]
        c_id: CollateralId,
        #[ink(topic)]
        collateral: Balance,
        #[ink(topic)]
        keeper_reward: Balance,
    }

    #[derive(
    Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode, SpreadLayout, PackedLayout,
    )]
    #[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct CollateralState {
        pub issuer: AccountId,
        pub collateral_pat: Balance,
        // 1 Cusd = 1 USD
        pub issue_cusd: Balance,
        pub create_date: Timestamp,
    }

    #[ink(storage)]
    pub struct CollateralManager {
        cusd_token: Lazy<Pat>,
        pat_token: Lazy<Pat>,
        cstate: StorageMap<CollateralId, CollateralState>,
        cstate_count: u32,
        min_c_ratio: u32,               //min collateral ratio
        min_liquidation_ratio: u32,
        liquidater_reward_ratio: u32,
        pat_price: USD,
        owner: AccountId,
    }

    impl Ownable for CollateralManager {
        #[ink(constructor)]
        fn new() -> Self {
            unimplemented!()
        }

        #[ink(message)]
        fn owner(&self) -> Option<AccountId> {
            Some(self.owner)
        }

        /// transfer contract ownership to new owner.
        #[ink(message)]
        fn transfer_ownership(&mut self, new_owner: Option<AccountId>) {
            assert_eq!(self.owner(), Some(self.env().caller()));
            if let Some(new_one) = new_owner {
                self.owner = new_one;
            }
        }
    }

    impl CollateralManager {
        #[ink(constructor)]
        pub fn new(cusd_contract: AccountId, pat_contract: AccountId) -> Self {
            assert_ne!(cusd_contract, Default::default());
            let caller = Self::env().caller();
            let cusd_token: Pat = FromAccountId::from_account_id(cusd_contract);
            let pat_token: Pat = FromAccountId::from_account_id(pat_contract);
            Self {
                cusd_token: Lazy::new(cusd_token),
                pat_token: Lazy::new(pat_token),
                cstate: StorageMap::new(),
                cstate_count: 0,
                min_c_ratio: 150,
                min_liquidation_ratio: 100,
                liquidater_reward_ratio: 150,
                pat_price: 4000,
                owner: caller,
            }
        }

        /// Adjust Min Collateral Ratio only admin
        #[ink(message)]
        pub fn set_min_cratio(&mut self, mcr: u32) {
            self.only_owner();
            self.min_c_ratio = mcr;
        }

        // Set Min Liquidation Ratio only admin
        #[ink(message)]
        pub fn set_min_lratio(&mut self, mlr: u32) {
            self.only_owner();
            self.min_liquidation_ratio = mlr;
        }

        /// Set Liquidater Reward Ratio only admin
        #[ink(message)]
        pub fn set_lrr(&mut self, lrr: u32) {
            self.only_owner();
            self.liquidater_reward_ratio = lrr;
        }

        /// Set pat price only admin
        #[ink(message)]
        pub fn set_pat_price(&mut self, price: USD) {
            self.only_owner();
            self.pat_price = price;
        }

        /// Get cstate by id
        #[ink(message)]
        pub fn get_cstate(&self, c_id: CollateralId) -> Option<CollateralState> {
            self.cstate.get(&c_id).cloned().and_then(|cstate| Some(cstate))
        }

        /// Stake collateral and issue cusd
        #[ink(message, payable)]
        pub fn issue_cusd(&mut self, cratio: u32) -> (CollateralId, Balance) {

            assert!(cratio >= self.min_c_ratio,"ERR_CRATION");
            ink_env::debug_println!("xxxxx_issue_cusd begin.");
            let caller = self.env().caller();
            let collateral = self.env().transferred_balance();
            let cusd_decimals = 10u128.saturating_pow(self.cusd_token.token_decimals().unwrap() as u32);
            let pat_decimals = 10u128.saturating_pow(self.pat_token.token_decimals().unwrap() as u32);

            let message = ink_prelude::format!("cusd_decimals is {:?}, pat_decimals is {:?}, collateral is {:?}", cusd_decimals, pat_decimals, collateral);
            debug_println!("{}",&message);

            let collateral_value =((collateral/pat_decimals) * (self.pat_price as u128)) / (PAT_PRICE_DECIMALS as u128);

            let cusd =  (collateral_value * cusd_decimals  * 100) / (cratio as u128);
            let message = ink_prelude::format!("collateral_value is {:?}, self.pat_price is {:?}, cusd is {:?}", collateral_value,self.pat_price,  cusd);
            debug_println!("{}",&message);

            let cstate = CollateralState {
                issuer:  caller,
                collateral_pat: collateral,
                issue_cusd: cusd,
                create_date: self.env().block_timestamp(),
            };
            self.cstate_count += 1;
            self.cstate.insert(self.cstate_count, cstate);
            debug_println!("xxxxx_end");
            self.cusd_token.mint(caller, cusd).unwrap();
            self.env().emit_event(IssueCusd {
                c_id: self.cstate_count,
                collateral,
                cusd,
            });
            (self.cstate_count, cusd)
        }

        /// Only issuer can add collateral and update collateral ratio
        #[ink(message, payable)]
        pub fn add_collateral(&mut self, c_id: CollateralId) {
            assert!(self.cstate.contains_key(&c_id));
            let caller = self.env().caller();
            let collateral = self.env().transferred_balance();
            let cstate = self.cstate.get_mut(&c_id).unwrap();
            assert!(cstate.issuer == caller);
            // let cratio = (collateral + cstate.collateral_pat as u128) * self.pat_price as u128 * 100
            //     / cstate.issue_cusd;
            let cusd_decimals =
                10u128.saturating_pow(self.cusd_token.token_decimals().unwrap() as u32);
            let pat_decimals = 10u128.saturating_pow(self.pat_token.token_decimals().unwrap() as u32);

            let cratio = (collateral + cstate.collateral_pat as u128)
                * self.pat_price as u128
                * 100
                * cusd_decimals
                / (cstate.issue_cusd * pat_decimals * PAT_PRICE_DECIMALS as u128);

            // assert!(cratio >= self.min_c_ratio.into());
            cstate.collateral_pat += collateral;
            self.env().emit_event(AddCollateral {
                c_id,
                add_collateral: collateral,
                collateral_ratio: cratio as u32,
            });
        }

        /// Only issuer can minus collateral and update collateral ratio
        #[ink(message)]
        pub fn minus_collateral(&mut self, c_id: CollateralId, collateral: Balance) {
            assert!(self.cstate.contains_key(&c_id));
            let caller = self.env().caller();
            let cstate = self.cstate.get_mut(&c_id).unwrap();
            assert!(cstate.issuer == caller);
            // let cratio =
            //     (cstate.collateral_pat - collateral) * self.pat_price as u128 * 100 / cstate.issue_cusd;
            let cusd_decimals =
                10u128.saturating_pow(self.cusd_token.token_decimals().unwrap() as u32);
            let pat_decimals = 10u128.saturating_pow(self.pat_token.token_decimals().unwrap() as u32);

            let cratio =
                (cstate.collateral_pat - collateral) * self.pat_price as u128 * 100 * cusd_decimals
                    / (cstate.issue_cusd * pat_decimals * PAT_PRICE_DECIMALS as u128);

            // assert!(cratio >= self.min_c_ratio.into());
            cstate.collateral_pat -= collateral;
            self.env().transfer(caller, collateral).unwrap();
            self.env().emit_event(MinusCollateral {
                c_id,
                minus_collateral: collateral,
                collateral_ratio: cratio as u32,
            });
        }

        /// Only issuer can redeem
        #[ink(message)]
        pub fn redeem_pat(&mut self, c_id: CollateralId, cusd: Balance) -> Balance {
            assert!(self.cstate.contains_key(&c_id));
            let caller = self.env().caller();
            let cstate = self.cstate.get_mut(&c_id).unwrap();
            assert!(cstate.issuer == caller);
            // let cratio = (cstate.collateral_pat * self.pat_price as u128 * 100 / cstate.issue_cusd) as u32;
            // assert!(cratio >= self.min_c_ratio);
            assert!(cusd <= cstate.issue_cusd);

            let bt: U256 = cstate.collateral_pat.into();
            let bi: U256 = cusd.into();
            let ui: U256 = cstate.issue_cusd.into();
            let r = bt * bi / ui;
            let pat = r.as_u128();

            cstate.collateral_pat -= pat;
            cstate.issue_cusd -= cusd;
            self.env().transfer(caller, pat).unwrap();
            self.cusd_token.burn(caller, cusd).unwrap();
            self.env().emit_event(Withdraw {
                c_id,
                collateral: pat,
                cusd,
            });
            pat
        }

        /// Anyone can invoke collateral liquidation if current collateral ratio lower than minimum
        #[ink(message)]
        pub fn liquidate_collateral(&mut self, c_id: CollateralId, cusd: Balance) {
            assert!(self.cstate.contains_key(&c_id));
            let cstate = self.cstate.get_mut(&c_id).unwrap();
            // let cratio = (cstate.collateral_pat * self.pat_price as u128 * 100 / cstate.issue_cusd) as u32;
            let cusd_decimals =
                10u128.saturating_pow(self.cusd_token.token_decimals().unwrap() as u32);
            let pat_decimals = 10u128.saturating_pow(self.pat_token.token_decimals().unwrap() as u32);

            let cratio = (cstate.collateral_pat * self.pat_price as u128 * 100 * cusd_decimals
                / (cstate.issue_cusd * pat_decimals * PAT_PRICE_DECIMALS as u128)) as u32;
            assert!(cratio <= self.min_liquidation_ratio);
            let owner = cstate.issuer;
            let pat =
                cusd * pat_decimals * PAT_PRICE_DECIMALS as u128 / (self.pat_price as u128 * cusd_decimals);
            cstate.issue_cusd = cstate.issue_cusd.saturating_sub(cusd);
            // let keeper_reward =
            //     cusd * PATS * self.liquidater_reward_ratio as u128 * PAT_PRICE_DECIMALS as u128
            //         / (100 * self.pat_price as u128 * cusd_decimals);
            let keeper_reward = cusd * pat_decimals * self.liquidater_reward_ratio as u128
                / (self.pat_price as u128 * cusd_decimals);
            assert!(pat + keeper_reward <= cstate.collateral_pat);

            cstate.collateral_pat = cstate.collateral_pat - pat - keeper_reward;
            let mut rest_pat = 0_u128;
            if cstate.issue_cusd == 0 && cstate.collateral_pat > 0 {
                rest_pat = cstate.collateral_pat;
                cstate.collateral_pat = 0;
            }
            let caller = self.env().caller();
            assert!(self.env().transfer(caller, pat + keeper_reward).is_ok());
            assert!(self.cusd_token.burn(caller, cusd).is_ok());
            if rest_pat > 0 {
                assert!(self.env().transfer(owner, rest_pat).is_ok());
            }
            self.env().emit_event(Liquidate {
                c_id,
                collateral: pat,
                keeper_reward,
            });
        }

        /// Returns the total issuers、total collateral、total issue cusd.
        #[ink(message)]
        pub fn total_supply(&self) -> (u32, Balance, Balance) {
            let mut issuers = Vec::new();
            let total_collateral: Balance = self.env().balance();
            let total_issue_cusd: Balance = self.cusd_token.total_supply();
            for (_k, v) in self.cstate.iter() {
                if !issuers.contains(&v.issuer) {
                    issuers.push(v.issuer);
                }
                // total_collateral += v.collateral_pat;
                // total_issue_cusd += v.issue_cusd;
            }
            (issuers.len() as u32, total_collateral, total_issue_cusd)
        }

        /// Returns the total cstate amount.
        #[ink(message)]
        pub fn cstate_count(&self) -> u32 {
            self.cstate_count
        }

        fn only_owner(&self) {
            assert_eq!(self.env().caller(), self.owner);
        }
        /// System params
        #[ink(message)]
        pub fn system_params(&self) -> (u32, u32, u32, u32) {
            (
                self.min_c_ratio,
                self.min_liquidation_ratio,
                self.liquidater_reward_ratio,
                self.pat_price,
            )
        }
    }
}
