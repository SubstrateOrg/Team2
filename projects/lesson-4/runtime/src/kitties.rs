use support::{decl_module, decl_storage, ensure, StorageValue, StorageMap, dispatch::Result, Parameter};
use sr_primitives::traits::{SimpleArithmetic, Bounded};
use codec::{Encode, Decode};
use runtime_io::blake2_128;
use system::ensure_signed;
use rstd::result;
use core::convert::TryInto;

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

			// 作业：重构create方法，避免重复代码

			let kitty_id = Self::kitties_count();
			if kitty_id == T::KittyIndex::max_value() {
				return Err("Kitties count overflow");
			}

			// Generate a random 128bit value
			// let payload = (<system::Module<T>>::random_seed(), &sender, <system::Module<T>>::extrinsic_index(), <system::Module<T>>::block_number());
			// let dna = payload.using_encoded(blake2_128);
			let dna = Self::random_value(&sender);

			// Create and store kitty
			let kitty = Kitty(dna);
			Self::insert_kitty(sender, kitty_id, kitty);
			
			// <Kitties<T>>::insert(kitty_id, kitty);
			// <KittiesCount<T>>::put(kitty_id + 1.into());

			// // Store the ownership information
			// let user_kitties_id = Self::owned_kitties_count(&sender);
			// <OwnedKitties<T>>::insert((sender.clone(), user_kitties_id), kitty_id);
			// <OwnedKittiesCount<T>>::insert(sender, user_kitties_id + 1.into());
		}

		/// Breed kitties
		pub fn breed(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) {
			let sender = ensure_signed(origin)?;

			Self::do_breed(sender, kitty_id_1, kitty_id_2)?;
		}

		/// transfer kitties
		pub fn transfer(origin, from: T::AccountId, to: T::AccountId, kitty_id: T::KittyIndex) {
            let sender = ensure_signed(origin)?;

			ensure!(from != to, "Can't transfer to oneself");

            let kitty = Self::kitty(kitty_id);  
			ensure!(kitty.is_some(), "kitty does not exist");

			// from remove: kitty, from_kitty_count-1
			let from_kitties_id = Self::owned_kitties_count(&from);
			<OwnedKitties<T>>::remove((from.clone(), from_kitties_id));
			<OwnedKittiesCount<T>>::insert(from, from_kitties_id);

			// to add kitty
			let to_kitties_id = Self::owned_kitties_count(&to);
			<OwnedKitties<T>>::insert((to.clone(), to_kitties_id), kitty_id);
			<OwnedKittiesCount<T>>::insert(to, to_kitties_id + 1.into());
		}
	}
}

fn to_binary_arr(number: u8) -> [u8; 8] {
    let mut arr:[u8; 8] = [0; 8];
    for i in 0..8 {
        if number & (1 << i) > 0 {
			arr[i] = 1;
        }
    }
    return arr;
}

fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
	// 作业：实现combine_dna
	// 伪代码：
	// selector.map_bits(|bit, index| if (bit == 1) { dna1 & (1 << index) } else { dna2 & (1 << index) })
	// 注意 map_bits这个方法不存在。只要能达到同样效果，不局限算法
	// 测试数据：dna1 = 0b11110000, dna2 = 0b11001100, selector = 0b10101010, 返回值 0b11100100
	//u8 --> binary array
    let dna1_arr = to_binary_arr(dna1);
    let dna2_arr = to_binary_arr(dna2);
    let selector_arr = to_binary_arr(selector);

    let mut dna = 0u8;
    for i in 0..8 {
        if selector_arr[i] == 1 {
			let value:u8 = (i << dna1_arr[i]).try_into().unwrap();
			dna = dna + value;
        } else {
			let value:u8 = (i << dna2_arr[i]).try_into().unwrap();
			dna = dna + value;
        }
    }
	return dna;
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
}
