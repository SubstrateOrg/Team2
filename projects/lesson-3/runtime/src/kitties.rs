use support::{decl_module, decl_storage, StorageValue, StorageMap, ensure, dispatch::Result};
use codec::{Encode, Decode};
use sr_primitives::traits::{CheckedAdd};
use runtime_io::blake2_128;
use system::ensure_signed;

pub trait Trait: system::Trait {
}

#[derive(Encode, Decode, Default)]
pub struct Kitty(pub [u8; 16]);

decl_storage! {
	trait Store for Module<T: Trait> as Kitties {
		/// Stores all the kitties, key is the kitty id / index
		pub Kitties get(kitty): map u32 => Kitty;
		/// Stores the total number of kitties. i.e. the next kitty index
		pub KittiesCount get(kitties_count): u32;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		/// Create a new kitty
		pub fn create(origin) {
			let sender = ensure_signed(origin)?;
			let count = Self::kitties_count();
			// if count == u32::max_value() {
			// 	return Err("Kitties count overflow");
			// }
			//检查count+1后是否溢出就报错
			let updated_count = count.checked_add(1).ok_or("overflow when adding KittiesCount.")?;

			let payload = (<system::Module<T>>::random_seed(), sender, <system::Module<T>>::extrinsic_index(), <system::Module<T>>::block_number());
			let dna = payload.using_encoded(blake2_128);
			let kitty = Kitty(dna);
			Kitties::insert(count, kitty);
			KittiesCount::put(updated_count);
		}

		/// 两只kitty繁殖
		pub fn breed(origin, first_kitty_id: u32, second_kitty_id: u32) -> Result {
			let sender = ensure_signed(origin)?;

			ensure!(Kitties::exists(first_kitty_id), "First kitty does not exists");
			ensure!(Kitties::exists(second_kitty_id), "Second kitty does not exists");

			let count = Self::kitties_count();
			let updated_count = count.checked_add(1).ok_or("overflow when adding KittiesCount.")?;

			let first_kitty = Self::kitty(first_kitty_id);
			let second_kitty = Self::kitty(second_kitty_id);

			//dna继承算法
			//在生成dna增加父亲母亲的dna
			let payload = (<system::Module<T>>::random_seed(), sender, <system::Module<T>>::extrinsic_index(), <system::Module<T>>::block_number(), first_kitty.0, second_kitty.0);
			let dna = payload.using_encoded(blake2_128);

			let kitty = Kitty(dna);
			Kitties::insert(count, kitty);
			KittiesCount::put(updated_count);

			Ok(())
		}

		/// 转移kitty
		pub fn transfer(origin, kitty_id: u32, to: T::AccountId) -> Result {
			let sender = ensure_signed(origin)?;
			Ok(())
		}

		/// 设置kitty价格，价格不为零则为在售
		pub fn set_price(origin, kitty_id: u32, price: u64) -> Result {
			let sender = ensure_signed(origin)?;
			Ok(())
		}

		/// 购买在售的kitty
		pub fn buy(origin, kitty_id: u32) -> Result {
			let sender = ensure_signed(origin)?;
			Ok(())
		}
	}
}
