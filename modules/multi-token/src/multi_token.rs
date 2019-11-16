use codec::FullCodec;
use core::fmt::Debug;
use rstd::prelude::*;
use sr_primitives::traits::{CheckedAdd, CheckedSub, SimpleArithmetic};
use support::{decl_event, decl_module, decl_storage, dispatch::Result};
use system::{self, ensure_signed};

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    /// Numerical type for storing balance
    type TokenBalance: Debug + SimpleArithmetic + FullCodec + Default + Copy;
    /// Token id
    type Discriminant: Debug + PartialEq + FullCodec + Copy + Default;
}

// public interface for this runtime module
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // initialize the default event for this module
        fn deposit_event() = default;

        // transfer tokens from one account to another
        // origin is assumed as sender
        fn transfer(
            origin,
            to: <T as system::Trait>::AccountId,
            token_id: T::Discriminant,
            value: T::TokenBalance
        ) -> Result {
            let sender = ensure_signed(origin)?;
            let sender_bal = Self::balance_of((token_id, sender.clone()))
                .checked_sub(&value)
                .ok_or("Not enough balance.")?;
            let receiver_bal = Self::balance_of((token_id, to.clone()))
                .checked_add(&value)
                .ok_or("Balance overflow in receiver account.")?;
            if sender != to {
                <BalanceOf<T>>::insert((token_id, sender.clone()), sender_bal);
                <BalanceOf<T>>::insert((token_id, to.clone()), receiver_bal);
            }
            Self::deposit_event(RawEvent::Transfer(sender, to, token_id, value));
            Ok(())
        }
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Erc20 {
        BalanceOf
            get(balance_of)
            build(|config: &GenesisConfig<T>| config.balances.clone())
            : map (T::Discriminant, T::AccountId) => T::TokenBalance;
    }
    add_extra_genesis {
        config(balances): Vec<((T::Discriminant, T::AccountId), T::TokenBalance)>;
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)] // https://github.com/paritytech/substrate/issues/2114
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Discriminant = <T as Trait>::Discriminant,
        TokenBalance = <T as Trait>::TokenBalance,
    {
        Transfer(
            AccountId,    // from
            AccountId,    // to
            Discriminant, // token_id
            TokenBalance  // value
        ),
    }
);

#[cfg(test)]
mod test {
    use super::*;
    use codec::{Decode, Encode};
    use primitives::H256;
    use sr_primitives::weights::Weight;
    use sr_primitives::Perbill;
    use sr_primitives::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
    };
    use support::{impl_outer_origin, parameter_types};

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }
    impl system::Trait for Test {
        type Origin = Origin;
        type Call = ();
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
    }
    impl Trait for Test {
        type Event = ();
        type TokenBalance = u128;
        type Discriminant = TokenType;
    }
    type TemplateModule = Module<Test>;

    #[derive(Debug, PartialEq, Eq, Clone, Copy, Decode, Encode)]
    pub enum TokenType {
        A,
        B,
    }

    impl Default for TokenType {
        fn default() -> Self {
            Self::A
        }
    }

    /// test accounts
    const A: u64 = 0;
    const B: u64 = 1;
    const C: u64 = 2;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities {
        system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap()
            .into()
    }

    // set account balance of bene for token_id to total_supply
    fn cheat_in(bene: u64, token_id: TokenType, total_supply: u128) {
        <BalanceOf<Test>>::insert((token_id, bene), total_supply);
    }

    /// send tokens from A to B
    #[test]
    fn xfer() {
        new_test_ext().execute_with(|| {
            cheat_in(A, TokenType::A, 10);

            // transfer to B
            TemplateModule::transfer(Origin::signed(A), B, TokenType::A, 4).unwrap();

            // A has 6
            assert_eq!(TemplateModule::balance_of((TokenType::A, A)), 6);

            // B has 4
            assert_eq!(TemplateModule::balance_of((TokenType::A, B)), 4);
        });
    }

    #[test]
    fn cheat_in_meta() {
        new_test_ext().execute_with(|| {
            assert_eq!(TemplateModule::balance_of((TokenType::A, A)), 0);
            assert_eq!(TemplateModule::balance_of((TokenType::B, A)), 0);
            cheat_in(A, TokenType::A, 10);
            assert_eq!(TemplateModule::balance_of((TokenType::A, A)), 10);
            assert_eq!(TemplateModule::balance_of((TokenType::B, A)), 0);
            cheat_in(A, TokenType::B, 10);
            assert_eq!(TemplateModule::balance_of((TokenType::A, A)), 10);
            assert_eq!(TemplateModule::balance_of((TokenType::B, A)), 10);
        });
    }

    #[test]
    fn transfer_pong() {
        new_test_ext().execute_with(|| {
            cheat_in(A, TokenType::A, 10);
            assert_eq!(TemplateModule::balance_of((TokenType::A, A)), 10);
            assert_eq!(TemplateModule::balance_of((TokenType::A, B)), 0);
            TemplateModule::transfer(Origin::signed(A), B, TokenType::A, 1).unwrap();
            assert_eq!(TemplateModule::balance_of((TokenType::A, A)), 9);
            assert_eq!(TemplateModule::balance_of((TokenType::A, B)), 1);
            TemplateModule::transfer(Origin::signed(B), A, TokenType::A, 1).unwrap();
            assert_eq!(TemplateModule::balance_of((TokenType::A, A)), 10);
            assert_eq!(TemplateModule::balance_of((TokenType::A, B)), 0);
            TemplateModule::transfer(Origin::signed(A), B, TokenType::A, 1).unwrap();
            assert_eq!(TemplateModule::balance_of((TokenType::A, A)), 9);
            assert_eq!(TemplateModule::balance_of((TokenType::A, B)), 1);
            TemplateModule::transfer(Origin::signed(B), A, TokenType::A, 1).unwrap();
            assert_eq!(TemplateModule::balance_of((TokenType::A, A)), 10);
            assert_eq!(TemplateModule::balance_of((TokenType::A, B)), 0);
        });
    }

    #[test]
    fn transfer_before_create() {
        new_test_ext().execute_with(|| {
            TemplateModule::transfer(Origin::signed(A), B, TokenType::A, 1).unwrap_err();
            TemplateModule::transfer(Origin::signed(B), A, TokenType::A, 1).unwrap_err();
            TemplateModule::transfer(Origin::signed(A), B, TokenType::B, 1).unwrap_err();
        });
    }

    #[test]
    fn transfer_none() {
        new_test_ext().execute_with(|| {
            cheat_in(A, TokenType::A, 10);
            TemplateModule::transfer(Origin::signed(A), B, TokenType::A, 0).unwrap();
        });
    }

    #[test]
    fn transfer_twice() {
        new_test_ext().execute_with(|| {
            cheat_in(A, TokenType::A, 10);
            TemplateModule::transfer(Origin::signed(A), B, TokenType::A, 5).unwrap();
            TemplateModule::transfer(Origin::signed(A), B, TokenType::A, 5).unwrap();
            TemplateModule::transfer(Origin::signed(A), B, TokenType::A, 5).unwrap_err();
        });
    }

    #[test]
    fn transfer_overflow() {
        new_test_ext().execute_with(|| {
            cheat_in(A, TokenType::A, u128::max_value());
            TemplateModule::transfer(Origin::signed(A), B, TokenType::A, u128::max_value())
                .unwrap();
            assert_eq!(TemplateModule::balance_of((TokenType::A, A)), 0);
            assert_eq!(
                TemplateModule::balance_of((TokenType::A, B)),
                u128::max_value()
            );
        });
    }

    #[test]
    fn transfer_too_much() {
        new_test_ext().execute_with(|| {
            cheat_in(A, TokenType::A, 10);
            TemplateModule::transfer(Origin::signed(A), B, TokenType::A, 11).unwrap_err();
            assert_eq!(TemplateModule::balance_of((TokenType::A, A)), 10);
            assert_eq!(TemplateModule::balance_of((TokenType::A, B)), 0);
        });
    }

    #[test]
    fn transfer_to_self() {
        new_test_ext().execute_with(|| {
            cheat_in(A, TokenType::A, 10);
            TemplateModule::transfer(Origin::signed(A), A, TokenType::A, 10).unwrap();
            assert_eq!(TemplateModule::balance_of((TokenType::A, A)), 10);
        });
    }

    #[test]
    fn transfer_too_much_to_self() {
        new_test_ext().execute_with(|| {
            cheat_in(A, TokenType::A, 10);
            TemplateModule::transfer(Origin::signed(A), A, TokenType::A, 11).unwrap_err();
            assert_eq!(TemplateModule::balance_of((TokenType::A, A)), 10);
        });
    }

    #[test]
    fn transfer_zero_to_self() {
        new_test_ext().execute_with(|| {
            cheat_in(A, TokenType::A, 10);
            TemplateModule::transfer(Origin::signed(A), A, TokenType::A, 0).unwrap();
            assert_eq!(TemplateModule::balance_of((TokenType::A, A)), 10);
        });
    }

    #[test]
    fn default_balance_zero() {
        new_test_ext().execute_with(|| {
            assert_eq!(TemplateModule::balance_of((TokenType::A, A)), 0);
            assert_eq!(TemplateModule::balance_of((TokenType::A, B)), 0);
            assert_eq!(TemplateModule::balance_of((TokenType::A, C)), 0);
            assert_eq!(TemplateModule::balance_of((TokenType::B, A)), 0);
            assert_eq!(TemplateModule::balance_of((TokenType::B, B)), 0);
            assert_eq!(TemplateModule::balance_of((TokenType::B, C)), 0);
        });
    }
}
