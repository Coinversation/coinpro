#![cfg_attr(not(feature = "std"), no_std)]

pub use self::math::Math;

pub use self::math::BONE;
pub use self::math::EXIT_FEE;
pub use self::math::MIN_FEE;
pub use self::math::MAX_FEE;
pub use self::math::MIN_BOUND_TOKENS;
pub use self::math::MAX_BOUND_TOKENS;
pub use self::math::INIT_POOL_SUPPLY;
pub use self::math::MIN_WEIGHT;
pub use self::math::MAX_WEIGHT;
pub use self::math::MIN_BALANCE;
pub use self::math::MAX_TOTAL_WEIGHT;
pub use self::math::MAX_OUT_RATIO;
pub use self::math::MAX_IN_RATIO;

use ink_lang as ink;

#[ink::contract]
mod math {
    pub const BONE: u128 = 10000000000;
    pub const MIN_BOUND_TOKENS: u128  = 2;
    pub const MAX_BOUND_TOKENS: u128  = 8;
    pub const MIN_FEE: u128           = BONE / 1000000;
    pub const MAX_FEE: u128           = BONE / 10;
    pub const EXIT_FEE: u128          = 0;

    pub const MIN_WEIGHT: u128        = BONE;
    pub const MAX_WEIGHT: u128        = BONE * 50;
    pub const MAX_TOTAL_WEIGHT: u128  = BONE * 50;
    pub const MIN_BALANCE: u128       = 10000;

    pub const INIT_POOL_SUPPLY: u128  = BONE * 100;

    pub const MIN_BPOW_BASE: u128     = 1;
    pub const MAX_BPOW_BASE: u128     = (2 * BONE) - 1;
    pub const BPOW_PRECISION: u128    = BONE / 100;

    pub const MAX_IN_RATIO: u128      = BONE / 2;
    pub const MAX_OUT_RATIO: u128     = (BONE / 3) + 1;

    #[ink(storage)]
    pub struct Math {
    }

    impl Math {
        #[ink(constructor)]
        pub fn new() -> Self { Self {} }

        #[ink(message)]
        pub fn btoi(&self, a : u128) -> u128 {
            return a / BONE;
        }

        #[ink(message)]
        pub fn bfloor(&self, a : u128) -> u128 {
            let b = self.btoi(a) * BONE;
            return b;
        }

        #[ink(message)]
        pub fn badd(&self, a : u128, b : u128) -> u128 {
            let c = a + b;
            assert!(c >= a, "ERR_ADD_OVERFLOW");
            return c;
        }

        #[ink(message)]
        pub fn bsub(&self, a : u128, b : u128) -> u128 {
            let (c, flag) = self.bsub_sign(a, b);
            assert!(!flag, "ERR_SUB_UNDERFLOW");
            return c;
        }

        #[ink(message)]
        pub fn bsub_sign(&self, a : u128, b : u128) -> (u128, bool) {
            return if a >= b {
                (a - b, false)
            } else {
                (b - a, true)
            }
        }

        #[ink(message)]
        pub fn bmul(&self, a : u128, b : u128) -> u128 {
            let c0 = a * b;
            assert!(a == 0 || c0 / a == b, "ERR_MUL_OVERFLOW");
            ink_env::debug_println!("assert 1");

            let c1 = c0 + (BONE / 2);
            assert!(c1 >= c0, "ERR_MUL_OVERFLOW");
            ink_env::debug_println!("assert 2");
            let c2 = c1 / BONE;
            return c2;
        }

        #[ink(message)]
        pub fn bdiv(&self, a : u128, b : u128) -> u128 {
            assert_ne!(b, 0, "ERR_DIV_ZERO");
            let c0 = a * BONE;
            assert!(a == 0 || c0 / a == BONE, "ERR_DIV_INTERNAL"); // bmul overflow
            let c1 = c0 + (b / 2);
            assert!(c1 >= c0, "ERR_DIV_INTERNAL"); //  badd require
            let c2 = c1 / b;
            return c2;
        }

        #[ink(message)]
        pub fn bpowi(&self, a : u128, n : u128) -> u128 {
            let mut z = a;
            if n % 2 == 0 {
                z = BONE;
            }

            let mut b = n;
            b = b / 2;
            let mut c = a;
            while  b != 0 {
                c = self.bmul(c, c);
                b = b / 2;
                if b % 2 != 0 {
                    z = self.bmul(z, c);
                }
            }
            return z;
        }

        #[ink(message)]
        pub fn bpow(&self, base : u128, exp : u128) -> u128 {
            assert!(base >= MIN_BPOW_BASE, "ERR_BPOW_BASE_TOO_LOW");
            assert!(base <= MAX_BPOW_BASE, "ERR_BPOW_BASE_TOO_HIGH");

            let whole  = self.bfloor(exp);
            let remain = self.bsub(exp, whole);

            let whole_pow = self.bpowi(base, self.btoi(whole));

            if remain == 0 {
                return whole_pow;
            }

            let partial_result = self.bpow_approx(base, remain, BPOW_PRECISION);
            return self.bmul(whole_pow, partial_result);
        }

        #[ink(message)]
        pub fn bpow_approx(&self, base : u128, exp : u128, precision : u128) -> u128 {
            let a= exp;
            let (x, xneg) = self.bsub_sign(base, BONE);
            let mut term = BONE;
            let mut sum = term;
            let mut negative = false;
            let mut i: u128 = 1;
            while term >= precision {
                let big_k = i * BONE;
                let (c, cneg) = self.bsub_sign(a, self.bsub(big_k, BONE));
                term = self.bmul(term, self.bmul(c, x));
                term = self.bdiv(term, big_k);
                if term == 0 {
                    break;
                }

                if xneg {
                    negative = !negative;
                }

                if cneg {
                    negative = !negative;
                }
                if negative {
                    sum = self.bsub(sum, term);
                } else {
                    sum = self.badd(sum, term);
                }

                i = i + 1;
            }
            return sum;
        }
    }
}
