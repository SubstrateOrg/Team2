use support::{decl_module, decl_storage, ensure, StorageValue, StorageMap, dispatch::Result, Parameter};
use sr_primitives::traits::{SimpleArithmetic, Bounded, Member, Zero};
use codec::{Encode, Decode};
use runtime_io::blake2_128;
use system::ensure_signed;
use rstd::result;

pub trait Trait: balances::Trait {
	type KittyIndex: Parameter + Member + SimpleArithmetic + Bounded + Default + Copy;
}

#[derive(Encode, Decode)]
pub struct Kitty<T: Trait> {
  pub dna: [u8; 16],
  pub price: T::Balance, //添加 balance
}

// pub struct Kitty(pub [u8; 16]);
// pub struct Kitty<Balance>(pub [u8; 16], Balance);

#[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode)]
pub struct KittyLinkedItem<T: Trait> {
	pub prev: Option<T::KittyIndex>,
	pub next: Option<T::KittyIndex>,
}

decl_storage! {
	trait Store for Module<T: Trait> as Kitties {
		/// Stores all the kitties, key is the kitty id / index
		pub Kitties get(kitty): map T::KittyIndex => Option<Kitty<T> >;
		/// Stores the total number of kitties. i.e. the next kitty index
		pub KittiesCount get(kitties_count): T::KittyIndex;

		pub OwnedKitties get(owned_kitties): map (T::AccountId, Option<T::KittyIndex>) => Option<KittyLinkedItem<T>>;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		/// Create a new kitty
		pub fn create(origin) {
			let sender = ensure_signed(origin)?;
			let kitty_id = Self::next_kitty_id()?;

			// Generate a random 128bit value
			let dna = Self::random_value(&sender);

			// Create and store kitty
			let kitty = Kitty {
				dna: dna, price: Zero::zero() };

			Self::insert_kitty(&sender, kitty_id, kitty);
		}

		/// Breed kitties
		pub fn breed(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) {
			let sender = ensure_signed(origin)?;

			Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;
		}

		// 作业：实现 transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex)
		// 使用 ensure! 来保证只有主人才有权限调用 transfer
		// 使用 OwnedKitties::append 和 OwnedKitties::remove 来修改小猫的主人
		pub fn transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex) {
			let sender = ensure_signed(origin)?;

			let kitty = <OwnedKitties<T>>::get(&(sender.clone(), Some(kitty_id)));
			ensure!(kitty.is_some(), "the kitty is not belong to you!");

			<OwnedKitties<T>>::append(&to, kitty_id);
			<OwnedKitties<T>>::remove(&sender, kitty_id);
		}

		// 设定价格
		pub fn set_price(origin, to: T::AccountId, kitty_id: T::KittyIndex, price: T::Balance) {
			let sender = ensure_signed(origin)?;
			let kitty = <OwnedKitties<T>>::get(&(sender.clone(), Some(kitty_id)));
			ensure!(kitty.is_some(), "the kitty is not belong to you!");

			// ensure!(price>0, "the kitty price must > 0!");

			let mut _kitty = <Kitties<T>>::get(&kitty_id);
			_kitty.unwrap().price = price;
		}

		//  买猫
		pub fn buy_kitty(origin, to: T::AccountId, kitty_id: T::KittyIndex) {
			// TODO
			// 价格0 为非卖品
			// check balance
			// transfer
			// balance - price ， balance + price
		}

		// 卖猫
		pub fn sell_kitty(origin, to: T::AccountId, kitty_id: T::KittyIndex) {
			// TODO
			// TODO
			// TODO
		}

	}
}

impl<T: Trait> OwnedKitties<T> {
	fn read_head(account: &T::AccountId) -> KittyLinkedItem<T> {
 		Self::read(account, None)
 	}

	fn write_head(account: &T::AccountId, item: KittyLinkedItem<T>) {
 		Self::write(account, None, item);
 	}

	fn read(account: &T::AccountId, key: Option<T::KittyIndex>) -> KittyLinkedItem<T> {
 		<OwnedKitties<T>>::get(&(account.clone(), key)).unwrap_or_else(|| KittyLinkedItem {
 			prev: None,
 			next: None,
 		})
 	}

	fn write(account: &T::AccountId, key: Option<T::KittyIndex>, item: KittyLinkedItem<T>) {
 		<OwnedKitties<T>>::insert(&(account.clone(), key), item);
 	}

	pub fn append(account: &T::AccountId, kitty_id: T::KittyIndex) {
		let head = Self::read_head(account);
		let new_head = KittyLinkedItem {
 			prev: Some(kitty_id),
 			next: head.next,
 		};

		Self::write_head(account, new_head);

		let prev = Self::read(account, head.prev);
		let new_prev = KittyLinkedItem {
 			prev: prev.prev,
 			next: Some(kitty_id),
 		};
		Self::write(account, head.prev, new_prev);

		let item = KittyLinkedItem {
 			prev: head.prev,
 			next: None,
 		};
 		Self::write(account, Some(kitty_id), item);
	}

	pub fn remove(account: &T::AccountId, kitty_id: T::KittyIndex) {
		if let Some(item) = <OwnedKitties<T>>::take(&(account.clone(), Some(kitty_id))) {
			let prev = Self::read(account, item.prev);
			let new_prev = KittyLinkedItem {
 				prev: prev.prev,
 				next: item.next,
 			};

			Self::write(account, item.prev, new_prev);

			let next = Self::read(account, item.next);
 			let new_next = KittyLinkedItem {
 				prev: item.prev,
 				next: next.next,
 			};

  			Self::write(account, item.next, new_next);
		}
	}
}

fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
	((selector & dna1) | (!selector & dna2))
}

impl<T: Trait> Module<T> {
	fn random_value(sender: &T::AccountId) -> [u8; 16] {
		let payload = (<system::Module<T>>::random_seed(), sender, <system::Module<T>>::extrinsic_index(), <system::Module<T>>::block_number());
		payload.using_encoded(blake2_128)
	}

	fn next_kitty_id() -> result::Result<T::KittyIndex, &'static str> {
		let kitty_id = Self::kitties_count();
		if kitty_id == T::KittyIndex::max_value() {
			return Err("Kitties count overflow");
		}

		Ok(kitty_id)
	}

	fn insert_owned_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex) {
		// 作业：调用 OwnedKitties::append 完成实现
		<OwnedKitties<T>>::append(owner, kitty_id);
  }

	fn insert_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty<T>) {
		// Create and store kitty
		<Kitties<T>>::insert(kitty_id, kitty);
		<KittiesCount<T>>::put(kitty_id + 1.into());

		Self::insert_owned_kitty(owner, kitty_id);
	}

	fn do_breed(sender: &T::AccountId, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> Result {
		let kitty1 = Self::kitty(kitty_id_1);
		let kitty2 = Self::kitty(kitty_id_2);

		ensure!(kitty1.is_some(), "Invalid kitty_id_1");
		ensure!(kitty2.is_some(), "Invalid kitty_id_2");
		ensure!(kitty_id_1 != kitty_id_2, "Needs different parent");

		let kitty_id = Self::next_kitty_id()?;

		let kitty1_dna = kitty1.unwrap().dna;
		let kitty2_dna = kitty2.unwrap().dna;

		// Generate a random 128bit value
		let selector = Self::random_value(&sender);
		let mut new_dna = [0u8; 16];

		// Combine parents and selector to create new kitty
		for i in 0..kitty1_dna.len() {
			new_dna[i] = combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
		}

		let kitty = Kitty {
			dna: new_dna, price: Zero::zero()	};

		Self::insert_kitty(sender, kitty_id, kitty);

		Ok(())
	}
}

/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use runtime_io::with_externalities;
	use primitives::{H256, Blake2Hasher};
	use support::{impl_outer_origin, parameter_types};
	use sr_primitives::{traits::{BlakeTwo256, IdentityLookup}, testing::Header};
	use sr_primitives::weights::Weight;
	use sr_primitives::Perbill;

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq, Debug)]
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
		type KittyIndex = u32;
	}
	type OwnedKittiesTest = OwnedKitties<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	}

	#[test]
	fn owned_kitties_can_append_values() {
		with_externalities(&mut new_test_ext(), || {
			OwnedKittiesTest::append(&0, 1);

			assert_eq!(OwnedKittiesTest::get(&(0, None)), Some(KittyLinkedItem {
 				prev: Some(1),
 				next: Some(1),
 			}));

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(1))), Some(KittyLinkedItem {
 				prev: None,
 				next: None,
 			}));

			OwnedKittiesTest::append(&0, 2);

			assert_eq!(OwnedKittiesTest::get(&(0, None)), Some(KittyLinkedItem {
 				prev: Some(2),
 				next: Some(1),
 			}));

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(1))), Some(KittyLinkedItem {
 				prev: None,
 				next: Some(2),
 			}));

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(2))), Some(KittyLinkedItem {
 				prev: Some(1),
 				next: None,
 			}));

			OwnedKittiesTest::append(&0, 3);

  			assert_eq!(OwnedKittiesTest::get(&(0, None)), Some(KittyLinkedItem {
 				prev: Some(3),
 				next: Some(1),
 			}));

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(1))), Some(KittyLinkedItem {
 				prev: None,
 				next: Some(2),
 			}));

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(2))), Some(KittyLinkedItem {
 				prev: Some(1),
 				next: Some(3),
 			}));

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(3))), Some(KittyLinkedItem {
 				prev: Some(2),
 				next: None,
 			}));
		});
	}

	#[test]
 	fn owned_kitties_can_remove_values() {
 		with_externalities(&mut new_test_ext(), || {
			OwnedKittiesTest::append(&0, 1);
 			OwnedKittiesTest::append(&0, 2);
 			OwnedKittiesTest::append(&0, 3);

			OwnedKittiesTest::remove(&0, 2);

			assert_eq!(OwnedKittiesTest::get(&(0, None)), Some(KittyLinkedItem {
 				prev: Some(3),
 				next: Some(1),
 			}));

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(1))), Some(KittyLinkedItem {
 				prev: None,
 				next: Some(3),
 			}));

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(2))), None);

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(3))), Some(KittyLinkedItem {
 				prev: Some(1),
 				next: None,
 			}));

			OwnedKittiesTest::remove(&0, 1);

  			assert_eq!(OwnedKittiesTest::get(&(0, None)), Some(KittyLinkedItem {
 				prev: Some(3),
 				next: Some(3),
 			}));

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(1))), None);

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(2))), None);

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(3))), Some(KittyLinkedItem {
 				prev: None,
 				next: None,
 			}));

			OwnedKittiesTest::remove(&0, 3);

  			assert_eq!(OwnedKittiesTest::get(&(0, None)), Some(KittyLinkedItem {
 				prev: None,
 				next: None,
 			}));

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(1))), None);

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(2))), None);

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(2))), None);
		});
	}
}
