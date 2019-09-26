use support::{decl_module, decl_storage, ensure, StorageValue, StorageMap, dispatch::Result, Parameter};
use sr_primitives::traits::{SimpleArithmetic, Bounded};
use codec::{Encode, Decode};
use runtime_io::blake2_128;
use system::ensure_signed;
use rstd::result;

pub trait Trait: system::Trait {
	type KittyIndex: Parameter + SimpleArithmetic + Bounded + Default + Copy;
}

#[derive(Encode, Decode)]
pub struct Kitty(pub [u8; 16]);

decl_storage! {
	trait Store for Module<T: Trait> as Kitties {
		/// Stores all the kitties, key is the kitty id / index
		pub Kitties get(kitty): map T::KittyIndex => Option<Kitty>;
		/// Stores the total number of kitties. i.e. the next kitty index
		pub KittiesCount get(kitties_count): T::KittyIndex;

		/// Get kitty ID by account ID and user kitty index
		pub OwnedKitties get(owned_kitties): map (T::AccountId, T::KittyIndex) => T::KittyIndex;
		/// Get number of kitties by account ID
		pub OwnedKittiesCount get(owned_kitties_count): map T::AccountId => T::KittyIndex;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		/// Create a new kitty
		pub fn create(origin) {
			let sender = ensure_signed(origin)?;

			Self::do_create(sender)?;
		}

		/// Breed kitties
		pub fn breed(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) {
			let sender = ensure_signed(origin)?;

			Self::do_breed(sender, kitty_id_1, kitty_id_2)?;
		}

		pub fn transfer(origin, kitty_id: T::KittyIndex, to: T::AccountId) {
			let sender = ensure_signed(origin)?;

			Self::do_transfer(sender, kitty_id, to)?;
		}
	}
}

fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
	let dna1 = selector & dna1;
	let dna2 = !selector & dna2;

	dna1 | dna2
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

	fn insert_kitty(owner: T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty) {
		// Create and store kitty
		<Kitties<T>>::insert(kitty_id, kitty);
		<KittiesCount<T>>::put(kitty_id + 1.into());

		// Store the ownership information
		let user_kitties_id = Self::owned_kitties_count(owner.clone());
		<OwnedKitties<T>>::insert((owner.clone(), user_kitties_id), kitty_id);
		<OwnedKittiesCount<T>>::insert(owner, user_kitties_id + 1.into());
	}

	fn do_create(sender: T::AccountId) -> Result {
		let kitty_id = Self::next_kitty_id()?;

		// Generate a random 128bit value
		let dna = Self::random_value(&sender);

		// Create and store kitty
		Self::insert_kitty(sender, kitty_id, Kitty(dna));

		Ok(())
	}

	fn do_breed(sender: T::AccountId, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> Result {
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

	fn do_transfer(sender: T::AccountId, kitty_id: T::KittyIndex, to: T::AccountId) -> Result {
		let kitty_counts = Self::owned_kitties_count(&sender);
		ensure!(kitty_id < kitty_counts, "Invalid kitty_id");
		let global_kitty_id = Self::owned_kitties((sender.clone(), kitty_id));

		let last_kitty_id = kitty_counts - 1.into();
		if last_kitty_id != kitty_id {
			let last_global_kitty_id =  Self::owned_kitties((sender.clone(), last_kitty_id));
			<OwnedKitties<T>>::remove((sender.clone(), last_kitty_id));
			<OwnedKitties<T>>::insert((sender.clone(), kitty_id), last_global_kitty_id);
		}

		<OwnedKittiesCount<T>>::insert(sender, last_kitty_id);

		let last_kitty_id = Self::owned_kitties_count(&to);
		<OwnedKitties<T>>::insert((to.clone(), last_kitty_id), global_kitty_id);
		<OwnedKittiesCount<T>>::insert(to, last_kitty_id + 1.into());

		Ok(())
	}
}

#[test]
fn test_combine_dna() {
	let dna1 = 0b11110000;
	let dna2 = 0b11001100;
	let selector = 0b10101010;

	assert_eq!(combine_dna(dna1, dna2, selector), 0b11100100);
}
