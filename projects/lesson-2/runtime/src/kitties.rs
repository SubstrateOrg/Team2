/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references


/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs

use support::{decl_module, decl_storage, decl_event, StorageValue, StorageMap, dispatch::Result};
use system::ensure_signed;
use codec::{Encode, Decode};
use sr_primitives::traits::Hash;
use rstd::prelude::Vec;
use byteorder::{ByteOrder, LittleEndian};

/// The module's configuration trait.
pub trait Trait: system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct Kitty<AccountId> {
    id: u128,
    dna: u128,
    owner: AccountId,
}
// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as KittyModule {
		Something get(something): Option<u32>;
		KittyId: u128;
		KittyById get(kitty_by_id): map u128 => Option<Kitty<T::AccountId>>;
		KittyByOwner get(kitty_by_owner): map (T::AccountId) =>  Option<Vec<u128>>;
	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event() = default;

		// 创建猫咪
		pub fn create_kitty(origin,owner: T::AccountId) -> Result {
			let user = ensure_signed(origin)?;
			let kitty_last_id = KittyId::get();
			let hash = (<system::Module<T>>::random_seed(), user.clone(), kitty_last_id).using_encoded(<T as system::Trait>::Hashing::hash);
			let hash = hash.as_ref();

      		let dna = LittleEndian::read_u128(&hash[0..16]);
			let kitty = Kitty {
				id: kitty_last_id,
				dna: dna,
				owner: owner.clone(),
			};
			KittyId::mutate(|n| *n += 1);
			<KittyById<T>>::insert(kitty_last_id,kitty.clone());
			let owner_kitties;
			if let  Some(mut owner_kitties_tmp) = <KittyByOwner<T>>::get(owner.clone()) {
				owner_kitties_tmp.push(kitty.id);
				owner_kitties = owner_kitties_tmp;
			}else{
				let mut v = Vec::new();
				v.push(kitty.id);
				owner_kitties = v;
			}
			<KittyByOwner<T>>::insert(owner.clone(),owner_kitties);

			// 创建kitty
			// 随机dna
			// here we are raising the Something event
			Self::deposit_event(RawEvent::KittyCreated(kitty_last_id,user,owner));
			Ok(())
		}
		// takes a parameter of the type `AccountId`, stores it and emits an event
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		// Just a dummy event.
		// Event `Something` is declared with a parameter of the type `u32` and `AccountId`
		// To emit this event, we call the deposit funtion, from our runtime funtions
		KittyCreated(u128,AccountId,AccountId),
		SomethingStored(u32, AccountId),
	}
);

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use runtime_io::with_externalities;
    use primitives::{H256, Blake2Hasher};
    use support::{impl_outer_origin, assert_ok, parameter_types};
    use sr_primitives::{traits::{BlakeTwo256, IdentityLookup}, testing::Header};
    use sr_primitives::weights::Weight;
    use sr_primitives::Perbill;

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
        type WeightMultiplierUpdate = ();
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
    }

    impl Trait for Test {
        type Event = ();
    }

    type KittyModule = Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
    }

    #[test]
    fn it_works_for_default_value() {
        with_externalities(&mut new_test_ext(), || {
            // Just a dummy test for the dummy funtion `do_something`
            // calling the `do_something` function with a value 42
            assert_ok!(KittyModule::do_something(Origin::signed(1), 42));
            // asserting that the stored value is equal to what we stored
            assert_eq!(KittyModule::something(), Some(42));
        });
    }
}
