use support::{decl_module, decl_storage, decl_event, StorageValue, StorageMap, dispatch::Result};
use codec::{Encode, Decode};
use runtime_io::blake2_128;
use system::ensure_signed;

/// The module's configuration trait.
pub trait Trait: system::Trait {}

#[derive(Encode, Decode, Clone, PartialEq)]
pub struct Kitty<A> {
    dna: [u8; 16],
    owner: A,
    price: u64,
}

decl_storage! {
	trait Store for Module<T: Trait> as KittyModule {
	    pub Kitties get(kitties):map u32=>Kitty<T::AccountId>;
        pub KittiesCount get(kitties_count): u32;
		pub AccountKitties get(account_kitties): map (T::AccountId, u32) => u32;
        pub AccountKittiesCount get(account_kitties_count): map T::AccountId => u32;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		/// Create a new kitty
		pub fn create(origin) -> Result {
			let sender = ensure_signed(origin)?;
            let kitty_id = Self::kitties_count();
            if kitty_id == u32::max_value() {
				return Err("Kitties count overflow");
			}
		    let account_kitty_id = Self::account_kitties_count(&sender);


      		let payload = (<system::Module<T>>::random_seed(), sender, <system::Module<T>>::extrinsic_index(), <system::Module<T>>::block_number());
			let dna = payload.using_encoded(blake2_128);
			let new_kitty = Kitty {
				dna: dna,
				owner: sender.clone(),
				price:0,
			};
			<Kitties<T>>::insert(kitty_id,new_kitty);
			<AccountKitties<T>>::insert(sender.clone(),kitty_id);

			KittiesCount::put(kitty_id+1);
			<AccountKitties<T>>::insert(sender.clone(),account_kitty_id+1);
			Ok(())
		}
		pub fn breed(origin, first_kitty_id: u32, second_kitty_id: u32) -> Result {
			let sender = ensure_signed(origin)?;

			// 确保两只猫都存在
			ensure!(Kitties::exists(first_kitty_id), "第一只猫不存在");
			ensure!(Kitties::exists(second_kitty_id), "第二只猫不存在");

			let count = Self::kitties_count();
			let updated_count = count.checked_add(1).ok_or("生育数量过多，失败")?;
		    let account_kitty_id = Self::account_kitties_count(&sender);

			let first_kitty = Self::kitty(first_kitty_id);
			let second_kitty = Self::kitty(second_kitty_id);

			let payload = (<system::Module<T>>::random_seed(), sender, <system::Module<T>>::extrinsic_index(), <system::Module<T>>::block_number(), first_kitty.0, second_kitty.0);
			let dna = payload.using_encoded(blake2_128);

            let new_kitty = Kitty {
                dna: dna,
                owner: sender.clone(),
                price:0,
            };
			Kitties::insert(count, new_kitty);
			<AccountKitties<T>>::insert(sender.clone(),updated_count);

			KittiesCount::put(updated_count);
			<AccountKitties<T>>::insert(sender.clone(),account_kitty_id+1);

			Ok(())
		}

		/// transfer kitty
		pub fn transfer(origin, kitty_id: u32, to: T::AccountId) -> Result {
			let sender = ensure_signed(origin)?;
			Ok(())
		}

        // set price
        pub fn set_price(origin, kitty_id: u32, price: u64) -> Result {
            let sender = ensure_signed(origin)?;
            Ok(())
        }

        // buy kitty
        pub fn buy(origin, kitty_id: u32) -> Result {
            let sender = ensure_signed(origin)?;
            Ok(())
        }
	}
}
