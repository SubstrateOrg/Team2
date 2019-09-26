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
		/// 改位Option，不采用默认值，删除可以设为None，区分默认值0
		pub OwnedKitties get(owned_kitties): map (T::AccountId, T::KittyIndex) => Option<T::KittyIndex>;
		/// Get number of kitties by account ID
		pub OwnedKittiesCount get(owned_kitties_count): map T::AccountId => T::KittyIndex;

		/// Get kitty index of account by kitty ID
		/// KittyIndex => AccountKittyIndex
		pub OwnedKittiesIndex get(owned_kitties_index): map T::KittyIndex => T::KittyIndex;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		/// Create a new kitty
		pub fn create(origin) {
			let sender = ensure_signed(origin)?;

			let kitty_id = Self::next_kitty_id()?;

			let dna = Self::random_value(&sender);

			let kitty = Kitty(dna);
			Self::insert_kitty(sender,kitty_id,kitty);

		}

		/// Breed kitties
		pub fn breed(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) {
			let sender = ensure_signed(origin)?;

			Self::do_breed(sender, kitty_id_1, kitty_id_2)?;
		}

		/// Transfer a kitty
		pub fn transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex) {
			let sender = ensure_signed(origin)?;

			Self::do_transfer(sender, to, kitty_id)?;
		}
	}
}

fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
	// 作业：实现combine_dna
	// 伪代码：
	// selector.map_bits(|bit, index| if (bit == 1) { dna1 & (1 << index) } else { dna2 & (1 << index) })
	// 注意 map_bits这个方法不存在。只要能达到同样效果，不局限算法
	// 测试数据：dna1 = 0b11110000, dna2 = 0b11001100, selector = 0b10101010, 返回值 0b11100100

	((selector & dna1) | (!selector & dna2))

	// 注：参考陈老师github已有kitties项目，做了 手工验证 和 单元测试 验证
	// selector & dna1 = 0b10100000
	// !selector = 0b01010101
	// !selector & dna2 = 0b01000100
	// 结果 0b11100100
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

		<OwnedKittiesIndex<T>>::insert(kitty_id, user_kitties_id);
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

	fn do_transfer(from: T::AccountId, to: T::AccountId, kitty_id: T::KittyIndex) -> Result{
		let kitty = Self::kitty(kitty_id);
		ensure!(kitty.is_some(), "Invalid kitty_id");
		ensure!(from != to, "To account can not be yourself");

		let from_kitty_id = Self::owned_kitties_index(kitty_id);

		//let from_kitty_count = Self::owned_kitties_count(from);
		let to_kitty_count = Self::owned_kitties_count(&to);

		//修改from: 
		//from:OwnedKitties，删除，变为None
		<OwnedKitties<T>>::remove((from, from_kitty_id));
		//from:OwnedKittiesCount，不修改

		//修改to：
		//to:OwnedKitties，插入
		<OwnedKitties<T>>::insert((to.clone(), to_kitty_count), kitty_id);
		// //to:OwnedKittiesCount，修改
		<OwnedKittiesCount<T>>::insert(to, to_kitty_count + 1.into());

		// //OwnedKittiesIndex，修改为to的index
		<OwnedKittiesIndex<T>>::insert(kitty_id, to_kitty_count);
		
		Ok(())
	}
}


#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn it_works_for_combine_dna() {
		let dna1 = 0b11110000;
		let dna2 = 0b11001100;
		let selector = 0b10101010;
		let dna = combine_dna(dna1, dna2, selector);
		assert_eq!(dna, 0b11100100);
	}
}
