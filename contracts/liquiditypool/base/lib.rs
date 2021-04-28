#![cfg_attr(not(feature = "std"), no_std)]

pub use self::base::Base;
use ink_lang as ink;

#[ink::contract]
mod base {
    use math::{
        Math,
        BONE,
        EXIT_FEE,
    };

    use ink_storage::Lazy;
    use ink_env::call::FromAccountId;

    #[ink(storage)]
    pub struct Base {
        math: Lazy<Math>,
    }

    impl Base {
        #[ink(constructor)]
        pub fn new(math_address: AccountId) -> Self {
            let math: Math = FromAccountId::from_account_id(math_address);
            Self {
                math: Lazy::new(math),
            }
        }

        /**********************************************************************************************
        // calc_spot_price                                                                             //
        // sP = spot_price                                                                            //
        // bI = token_balance_in                ( bI / wI )         1                                  //
        // bO = token_balance_out         sP =  -----------  *  ----------                             //
        // wI = token_weight_in                 ( bO / wO )     ( 1 - sF )                             //
        // wO = token_weight_out                                                                       //
        // sF = swap_fee                                                                              //
        **********************************************************************************************/
        #[ink(message)]
        pub fn calc_spot_price(&self,
                               token_balance_in: u128,
                               token_weight_in: u128,
                               token_balance_out: u128,
                               token_weight_out: u128,
                               swap_fee: u128) -> u128 {
            let numer = self.math.bdiv(token_balance_in, token_weight_in);
            let denom = self.math.bdiv(token_balance_out, token_weight_out);
            let ratio = self.math.bdiv(numer, denom);
            let scale = self.math.bdiv(BONE, self.math.bsub(BONE, swap_fee));
            let spot_price = self.math.bmul(ratio, scale);
            return  spot_price;
        }

        /**********************************************************************************************
        // calc_out_given_in                                                                            //
        // aO = token_amount_out                                                                       //
        // bO = token_balance_out                                                                      //
        // bI = token_balance_in              /      /            bI             \    (wI / wO) \      //
        // aI = token_amount_in    aO = bO * |  1 - | --------------------------  | ^            |     //
        // wI = token_weight_in               \      \ ( bI + ( aI * ( 1 - sF )) /              /      //
        // wO = token_weight_out                                                                       //
        // sF = swap_fee                                                                              //
        **********************************************************************************************/
        #[ink(message)]
        pub fn calc_out_given_in(&self,
                                 token_balance_in: u128,
                                 token_weight_in: u128,
                                 token_balance_out: u128,
                                 token_weight_out: u128,
                                 token_amount_in: u128,
                                 swap_fee: u128) -> u128 {
            let weight_ratio = self.math.bdiv(token_weight_in, token_weight_out);
            let fee_in = self.math.bsub(BONE, swap_fee);
            let adjusted_in = self.math.bmul(token_amount_in, fee_in);
            let y = self.math.bdiv(token_balance_in, self.math.badd(token_balance_in, adjusted_in));
            let foo = self.math.bpow(y, weight_ratio);
            let bar = self.math.bsub(BONE, foo);
            let token_amount_out = self.math.bmul(token_balance_out, bar);
            return token_amount_out;
        }

        /**********************************************************************************************
        // calc_in_given_out                                                                            //
        // aI = token_amount_in                                                                        //
        // bO = token_balance_out               /  /     bO      \    (wO / wI)      \                 //
        // bI = token_balance_in          bI * |  | ------------  | ^            - 1  |                //
        // aO = token_amount_out    aI =        \  \ ( bO - aO ) /                   /                 //
        // wI = token_weight_in           --------------------------------------------                 //
        // wO = token_weight_out                          ( 1 - sF )                                   //
        // sF = swap_fee                                                                              //
        **********************************************************************************************/
        #[ink(message)]
        pub fn calc_in_given_out(&self,
                                 token_balance_in: u128,
                                 token_weight_in: u128,
                                 token_balance_out: u128,
                                 token_weight_out: u128,
                                 token_amount_out: u128,
                                 swap_fee: u128) -> u128 {
            let weight_ratio = self.math.bdiv(token_weight_out, token_weight_in);
            let diff = self.math.bsub(token_balance_out, token_amount_out);
            let y = self.math.bdiv(token_balance_out, diff);
            let mut foo = self.math.bpow(y, weight_ratio);
            foo = self.math.bsub(foo, BONE);
            let amount_in = self.math.bsub(BONE, swap_fee);
            let token_amount_in = self.math.bdiv(self.math.bmul(token_balance_in, foo), amount_in);
            return token_amount_in;
        }

        /**********************************************************************************************
        // calc_pool_out_given_single_in                                                                  //
        // pAo = pool_amount_out         /                                              \              //
        // tAi = token_amount_in        ///      /     //    wI \      \\       \     wI \             //
        // wI = token_weight_in        //| tAi *| 1 - || 1 - --  | * sF || + tBi \    --  \            //
        // tW = total_weight     pAo=||  \      \     \\    tW /      //         | ^ tW   | * pS - pS //
        // tBi = token_balance_in      \\  ------------------------------------- /        /            //
        // pS = pool_supply            \\                    tBi               /        /             //
        // sF = swap_fee                \                                              /              //
        **********************************************************************************************/
        #[ink(message)]
        pub fn calc_pool_out_given_single_in(&self,
                                             token_balance_in: u128,
                                             token_weight_in: u128,
                                             pool_supply: u128,
                                             total_weight: u128,
                                             token_amount_in: u128,
                                             swap_fee: u128) -> u128 {
            // Charge the trading fee for the proportion of tokenAi
            // That proportion is (1- weightTokenIn)
            // tokenAiAfterFee = tAi * (1 - (1-weightTi) * poolFee);
            let normalized_weight = self.math.bdiv(token_weight_in, total_weight);
            let zaz = self.math.bmul(self.math.bsub(BONE, normalized_weight), swap_fee);
            let token_amount_in_after_fee = self.math.bmul(token_amount_in, self.math.bsub(BONE, zaz));

            let new_token_balance_in = self.math.badd(token_balance_in, token_amount_in_after_fee);
            let token_in_ratio = self.math.bdiv(new_token_balance_in, token_balance_in);

            // uint newPoolSupply = (ratioTi ^ weightTi) * poolSupply;
            let pool_ratio = self.math.bpow(token_in_ratio, normalized_weight);
            let new_pool_supply = self.math.bmul(pool_ratio, pool_supply);
            let pool_amount_out = self.math.bsub(new_pool_supply, pool_supply);
            return pool_amount_out;
        }

        /**********************************************************************************************
        // calc_single_in_given_pool_out                                                                  //
        // tAi = token_amount_in              //(pS + pAo)\     /    1    \\                           //
        // pS = pool_supply                 || ---------  | ^ | --------- || * bI - bI                //
        // pAo = pool_amount_out              \\    pS    /     \(wI / tW)//                           //
        // bI = balance_in          tAi =  --------------------------------------------               //
        // wI = weight_in                              /      wI  \                                   //
        // tW = total_weight                          |  1 - ----  |  * sF                            //
        // sF = swap_fee                               \      tW  /                                   //
        **********************************************************************************************/
        #[ink(message)]
        pub fn calc_single_in_given_pool_out(&self,
                                             token_balance_in: u128,
                                             token_weight_in: u128,
                                             pool_supply: u128,
                                             total_weight: u128,
                                             pool_amount_out: u128,
                                             swap_fee: u128) -> u128 {
            let normalized_weight = self.math.bdiv(token_weight_in, total_weight);
            let new_pool_supply = self.math.badd(pool_supply, pool_amount_out);
            let pool_ratio = self.math.bdiv(new_pool_supply, pool_supply);

            //uint newBalTi = poolRatio^(1/weightTi) * balTi;
            let boo = self.math.bdiv(BONE, normalized_weight);
            let token_in_ratio = self.math.bpow(pool_ratio, boo);
            let new_token_balance_in = self.math.bmul(token_in_ratio, token_balance_in);
            let token_amount_in_after_fee = self.math.bsub(new_token_balance_in, token_balance_in);
            // Do reverse order of fees charged in joinswap_ExternAmountIn, this way
            //     ``` pAo == joinswap_ExternAmountIn(Ti, joinswap_PoolAmountOut(pAo, Ti)) ```
            //uint tAi = tAiAfterFee / (1 - (1-weightTi) * swapFee) ;
            let zar = self.math.bmul(self.math.bsub(BONE, normalized_weight), swap_fee);
            let token_amount_in = self.math.bdiv(token_amount_in_after_fee, self.math.bsub(BONE, zar));
            return token_amount_in;
        }

        /**********************************************************************************************
        // calc_single_out_given_pool_in                                                                  //
        // tAo = token_amount_out            /      /                                             \\   //
        // bO = token_balance_out           /      // pS - (pAi * (1 - eF)) \     /    1    \      \\  //
        // pAi = pool_amount_in            | bO - || ----------------------- | ^ | --------- | * b0 || //
        // ps = pool_supply                \      \\          pS           /     \(wO / tW)/      //  //
        // wI = token_weight_in      tAo =   \      \                                             //   //
        // tW = total_weight                    /     /      wO \       \                             //
        // sF = swap_fee                    *  | 1 - |  1 - ---- | * sF  |                            //
        // eF = exit_fee                        \     \      tW /       /                             //
        **********************************************************************************************/
        #[ink(message)]
        pub fn calc_single_out_given_pool_in(&self,
                                             token_balance_out: u128,
                                             token_weight_out: u128,
                                             pool_supply: u128,
                                             total_weight: u128,
                                             pool_amount_in: u128,
                                             swap_fee: u128) -> u128 {
            let normalized_weight = self.math.bdiv(token_weight_out, total_weight);
            // charge exit fee on the pool token side
            // pAiAfterExitFee = pAi*(1-exitFee)
            let pool_amount_in_after_exit_fee = self.math.bmul(pool_amount_in, self.math.bsub(BONE, EXIT_FEE));
            let new_pool_supply = self.math.bsub(pool_supply, pool_amount_in_after_exit_fee);
            let pool_ratio = self.math.bdiv(new_pool_supply, pool_supply);

            // newBalTo = poolRatio^(1/weightTo) * balTo;
            let token_out_ratio = self.math.bpow(pool_ratio, self.math.bdiv(BONE, normalized_weight));
            let new_token_balance_out = self.math.bmul(token_out_ratio, token_balance_out);

            let token_amount_out_before_swap_fee = self.math.bsub(token_balance_out, new_token_balance_out);

            // charge swap fee on the output token side
            //uint tAo = tAoBeforeSwapFee * (1 - (1-weightTo) * swapFee)
            let zaz = self.math.bmul(self.math.bsub(BONE, normalized_weight), swap_fee);
            let token_amount_out = self.math.bmul(token_amount_out_before_swap_fee, self.math.bsub(BONE, zaz));
            return token_amount_out;
        }

        /**********************************************************************************************
        // calc_pool_in_given_single_out                                                                  //
        // pAi = pool_amount_in               // /               tAo             \\     / wO \     \   //
        // bO = token_balance_out            // | bO - -------------------------- |\   | ---- |     \  //
        // tAo = token_amount_out      pS - ||   \     1 - ((1 - (tO / tW)) * sF)/  | ^ \ tW /  * pS | //
        // ps = pool_supply                 \\ -----------------------------------/                /  //
        // wO = token_weight_out  pAi =       \\               bO                 /                /   //
        // tW = total_weight           -------------------------------------------------------------  //
        // sF = swap_fee                                        ( 1 - eF )                            //
        // eF = exit_fee                                                                              //
        **********************************************************************************************/
        #[ink(message)]
        pub fn calc_pool_in_given_single_out(&self,
                                             token_balance_out: u128,
                                             token_weight_out: u128,
                                             pool_supply: u128,
                                             total_weight: u128,
                                             token_amount_out: u128,
                                             swap_fee: u128) -> u128 {
            // charge swap fee on the output token side
            let normalized_weight = self.math.bdiv(token_weight_out, total_weight);
            //uint tAoBeforeSwapFee = tAo / (1 - (1-weightTo) * swapFee) ;
            let zoo = self.math.bsub(BONE, normalized_weight);
            let zar = self.math.bmul(zoo, swap_fee);
            let token_amount_out_before_swap_fee = self.math.bdiv(token_amount_out, self.math.bsub(BONE, zar));

            let new_token_balance_out = self.math.bsub(token_balance_out, token_amount_out_before_swap_fee);
            let token_out_ratio = self.math.bdiv(new_token_balance_out, token_balance_out);

            //uint newPoolSupply = (ratioTo ^ weightTo) * poolSupply;
            let pool_ratio = self.math.bpow(token_out_ratio, normalized_weight);
            let new_pool_supply = self.math.bmul(pool_ratio, pool_supply);
            let pool_amount_in_after_exit_fee = self.math.bsub(pool_supply, new_pool_supply);

            // charge exit fee on the pool token side
            // pAi = pAiAfterExitFee/(1-exitFee)
            let pool_amount_in = self.math.bdiv(pool_amount_in_after_exit_fee, self.math.bsub(BONE, EXIT_FEE));
            return pool_amount_in;
        }
    }
}
