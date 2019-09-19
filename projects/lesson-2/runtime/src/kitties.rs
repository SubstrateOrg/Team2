/// A runtime module kitties with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references


/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs
/// 
use rstd::prelude::*;
use support::{decl_module, decl_storage, decl_event, StorageValue, StorageMap, dispatch::Result};
use codec::{Encode,Decode};
use system::ensure_signed;
use runtime_io::blake2_128;
//use primitives::traits::{As};

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Kitty<U> {
	dna: [u8; 16],
	owner: U,
	price: u64
}

/// The module's configuration trait.
pub trait Trait: system::Trait {
	// TODO: Add other types and constants required configure this module.

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}



// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as KttiesModule {

		//所有Kitty: kitty_id => kitty
		//可以通过递增kitty_id遍历所有kitties，通过kitties_count终止遍历
        pub Kitties get(kitties): map u32 => Kitty<T::AccountId>;
		//Kitty总数，模拟数组，用途：确定新增Kitty在map中的key，遍历所有Kitty
        pub KittiesCount get(kitties_count): u32;

		//用户的所有Kitty:（account,index） => kitty_id
		//可以通过account和递增的index遍历用户的kitties，通过account_kitties_count
		pub AccountKitties get(account_kitties): map (T::AccountId, u32) => u32;
		//这个用户的Kitty总数
		pub AccountKittiesCount get(account_kitties_count): map T::AccountId => u32;

	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event() = default;

		// 为自己创建一只kitty
		pub fn create(origin) -> Result {
			let sender = ensure_signed(origin)?;

			let kitty_id = Self::kitties_count();
			let account_kitty_id = Self::account_kitties_count(&sender);

			let dna = Self::dna(&sender, kitty_id);

			let kitty = Kitty {
				dna: dna,
				owner: sender.clone(),
				price: 0
			};
			<Kitties<T>>::insert(kitty_id, kitty);
			<AccountKitties<T>>::insert((sender.clone(),account_kitty_id), kitty_id);


			KittiesCount::put(kitty_id + 1);
			<AccountKittiesCount<T>>::insert(sender.clone(), account_kitty_id + 1);
			
			Self::deposit_event(RawEvent::KittyCreated(kitty_id, sender));
			Ok(())
		}

		//两只kitty繁殖
		pub fn breed(origin, first_kitty_id: u32, second_kitty_id: u32) -> Result {
			let sender = ensure_signed(origin)?;

			Ok(())
		}

		//转移kitty
		pub fn transfer(origin, kitty_id: u32, to: T::AccountId) -> Result {
			let sender = ensure_signed(origin)?;
			Ok(())
		}

		//设置kitty价格，价格不为零则为在售
		pub fn set_price(origin, kitty_id: u32, price: u64) -> Result {
			let sender = ensure_signed(origin)?;
			Ok(())
		}

		//购买在售的kitty
		pub fn buy(origin, kitty_id: u32) -> Result {
			let sender = ensure_signed(origin)?;
			Ok(())
		}

	}

}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		// Just a dummy event.
		// Event `Something` is declared with a parameter of the type `u32` and `AccountId`
		// To emit this event, we call the deposit funtion, from our runtime funtions
		KittyCreated(u32, AccountId),
	}
);


impl<T: Trait> Module<T> {
	fn dna(sender: &T::AccountId, kitty_id: u32) -> [u8; 16] {
		(<system::Module<T>>::random_seed(), sender, kitty_id)
		.using_encoded(blake2_128)
	}
}

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
	type KttiesModule = Module<Test>;

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
			assert_ok!(KttiesModule::do_something(Origin::signed(1), 42));
			// asserting that the stored value is equal to what we stored
			assert_eq!(KttiesModule::something(), Some(42));
		});
	}
}
