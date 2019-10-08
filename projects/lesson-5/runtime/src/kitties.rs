use support::{decl_module, decl_storage, ensure, StorageValue, StorageMap, dispatch::Result, Parameter};
use sr_primitives::traits::{SimpleArithmetic, Bounded, Member};
use codec::{Encode, Decode};
use runtime_io::blake2_128;
use system::ensure_signed;
use rstd::result;

pub trait Trait: system::Trait {
	type KittyIndex: Parameter + Member + SimpleArithmetic + Bounded + Default + Copy;
}

#[derive(Encode, Decode)]
pub struct Kitty(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode)]
pub struct KittyLinkedItem<T: Trait> {
	pub prev: Option<T::KittyIndex>,
	pub next: Option<T::KittyIndex>,
}
#[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode)]
pub struct KittyInSale<T: Trait> {
	pub owned: Option<T::AccountId>,
	pub price: u64,
}

decl_storage! {
	trait Store for Module<T: Trait> as Kitties {
		/// Stores all the kitties, key is the kitty id / index
		pub Kitties get(kitty): map T::KittyIndex => Option<Kitty>;
		/// Stores the total number of kitties. i.e. the next kitty index
		pub KittiesCount get(kitties_count): T::KittyIndex;

		pub OwnedKitties get(owned_kitties): map (T::AccountId, Option<T::KittyIndex>) => Option<KittyLinkedItem<T>>;

		pub KittiesInSale get(kitty_price): map T::KittyIndex => Option<KittyInSale<T>>;
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
			let kitty = Kitty(dna);
			Self::insert_kitty(&sender, kitty_id, kitty);
		}

		/// Breed kitties
		pub fn breed(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) {
			let sender = ensure_signed(origin)?;

			Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;
		}

		/// Transfer kitties
		pub fn transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex) {
			let sender = ensure_signed(origin)?;

			Self::do_transfer(&sender, &to, kitty_id)?;
		}

		/// Set kitties price, for sale
		pub fn set_price(origin, kitty_id: T::KittyIndex, price: u64) {
			let sender = ensure_signed(origin)?;

			Self::do_set_price(sender, kitty_id, price)?;
		}

		/// Remove kitties price, not for sale
		pub fn remove_price(origin, kitty_id: T::KittyIndex) {
			let sender = ensure_signed(origin)?;

			Self::do_remove_price(sender, kitty_id)?;
		}

		/// Buy kitties on sale
		pub fn buy(origin, kitty_id: T::KittyIndex) {
			let sender = ensure_signed(origin)?;

			Self::do_buy(&sender, kitty_id)?;
		}

		// 作业：实现 transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex)
		// 使用 ensure! 来保证只有主人才有权限调用 transfer
		// 使用 OwnedKitties::append 和 OwnedKitties::remove 来修改小猫的主人
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
		//首次添加：得到None，None
		//第二次添加：得到上次的kitty_id，上次的kitty_id（头kitty_id）
		
		let head = Self::read_head(account);
		let new_head = KittyLinkedItem {
 			prev: Some(kitty_id),
 			next: head.next,//加入之前的链表头，首次添加：None
 		};

		//新加入的kitty成为链表头
		//链表头，最后一个，第一个
		Self::write_head(account, new_head);
		//首次添加：（Alice，None）-》{kitty_id, None}
		//第二次添加：（Alice，None）-》{本次kitty_id，上次kitty_id}

		//首次添加：得到new_head，{kitty_id, None}
		//第二次添加：得到(None,None),(上个，None)
		let prev = Self::read(account, head.prev);
		let new_prev = KittyLinkedItem {
 			prev: prev.prev,//首次添加，为kitty_id。第二次添加，上一个
 			next: Some(kitty_id),
 		};
		//首次添加：把头替换成（Alice，None）-》{kitty_id, kitty_id}
		//第二次添加：(Alice,上次kitty_id) -》{None,本次kitty_id}
		//链表中：上一个，下一个
		Self::write(account, head.prev, new_prev);

		//首次添加，None，None
		//第二次添加，上次的id，None
		let item = KittyLinkedItem {
 			prev: head.prev,
 			next: None,
 		};

		//首次添加：把头替换成（Alice，kitty_id）-》{None, None}
		//第二次添加：（Alice，kitty_id）-》{上次kitty_id, None}
		//链表尾：上一个，None
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

	fn insert_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty) {
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

		let kitty1_dna = kitty1.unwrap().0;
		let kitty2_dna = kitty2.unwrap().0;

		// Generate a random 128bit value
		let selector = Self::random_value(&sender);
		let mut new_dna = [0u8; 16];

		// Combine parents and selector to create new kitty
		for i in 0..kitty1_dna.len() {
			new_dna[i] = combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
		}

		Self::insert_kitty(sender, kitty_id, Kitty(new_dna));

		Ok(())
	}

	fn do_transfer(sender: &T::AccountId, to: &T::AccountId, kitty_id: T::KittyIndex ) -> Result {
		let kitty = Self::kitty(kitty_id);
		ensure!(kitty.is_some(), "Invalid kitty_id");

		let owned = <OwnedKitties<T>>::read(sender, Some(kitty_id));

		ensure!(owned.prev.is_some() || owned.next.is_some(), "You didn't owned this kitty");

		// 使用 OwnedKitties::append 和 OwnedKitties::remove 来修改小猫的主人
		<OwnedKitties<T>>::append(&to, kitty_id);

		<OwnedKitties<T>>::remove(sender, kitty_id);

		//转移后删除价格，不在售
		<KittiesInSale<T>>::remove(kitty_id);

		Ok(())
	}

	fn do_set_price(sender: T::AccountId, kitty_id: T::KittyIndex, price: u64) -> Result {
		let kitty = Self::kitty(kitty_id);
		ensure!(kitty.is_some(), "Invalid kitty_id");

		let owned = <OwnedKitties<T>>::read(&sender, Some(kitty_id));

		ensure!(owned.prev.is_some() || owned.next.is_some(), "You didn't owned this kitty");

		let in_sale = KittyInSale {
			owned: Some(sender),
			price: price
		};
		<KittiesInSale<T>>::insert(kitty_id, in_sale);

		Ok(())
	}

	fn do_remove_price(sender: T::AccountId, kitty_id: T::KittyIndex) -> Result{
		let kitty = Self::kitty(kitty_id);
		ensure!(kitty.is_some(), "Invalid kitty_id");

		let owned = <OwnedKitties<T>>::read(&sender, Some(kitty_id));

		ensure!(owned.prev.is_some() || owned.next.is_some(), "You didn't owned this kitty");

		<KittiesInSale<T>>::remove(kitty_id);

		Ok(())
	}

	fn do_buy(sender: &T::AccountId, kitty_id: T::KittyIndex) -> Result {
		let kitty = Self::kitty(kitty_id);
		ensure!(kitty.is_some(), "Invalid kitty_id");

		let owned = <OwnedKitties<T>>::read(&sender, Some(kitty_id));

		ensure!(owned.prev.is_none() || owned.next.is_none(), "You already owned this kitty");

		let in_sale = Self::kitty_price(kitty_id);
		ensure!(in_sale.is_some(), "Kitty not in sale");

		Self::do_transfer(&in_sale.unwrap().owned.unwrap(), &sender, kitty_id)?;

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
