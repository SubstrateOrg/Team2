use support::{decl_module, decl_storage, StorageValue, StorageMap};
use codec::{Encode, Decode};
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
			if count == u32::max_value() {
				return Err("Kitties count overflow");
			}
			let payload = (<system::Module<T>>::random_seed(), sender, <system::Module<T>>::extrinsic_index(), <system::Module<T>>::block_number());
			let dna = payload.using_encoded(blake2_128);
			let kitty = Kitty(dna);
			Kitties::insert(count, kitty);
			KittiesCount::put(count + 1);
		}

		/// breed a child kitty
		pub fn breed(origin, index1: u32, index2: u32) {
			let sender = ensure_signed(origin)?;
			
			if index1 == index2 || index1 == u32::max_value() || index2 == u32::max_value() {
				return Err("Invalid parent kitties ID");
			}
			let kitty1 = Self::kitty(index1);
			let kitty2 = Self::kitty(index2);
			
			let count = Self::kitties_count();
			if count == u32::max_value() {
				return Err("Kitties count overflow");
			}

			//calc child kitty dna
			let kitty1_dna = kitty1.0;
			let kitty2_dna = kitty2.0;
			let payload = (<system::Module<T>>::random_seed(), sender, <system::Module<T>>::extrinsic_index(), <system::Module<T>>::block_number());
			let dna = payload.using_encoded(blake2_128);
			let mut child_dna = [0u8; 16];
			for i in 0..kitty1_dna.len() {
				child_dna[i] = (!dna[i] & kitty1_dna[i]) | (dna[i] & kitty2_dna[i]);
			}

			let child_kitty = Kitty(child_dna);
			Kitties::insert(count, child_kitty);
			KittiesCount::put(count + 1);
		}
	}

}
