use support::{decl_module, decl_storage, ensure, StorageValue, StorageMap, dispatch::Result, Parameter};
use sr_primitives::traits::{SimpleArithmetic, Bounded};
use codec::{Encode, Decode};
use runtime_io::blake2_128;
use system::ensure_signed;
use sr_primitives::traits::Hash;
use rstd::cmp;

pub trait Trait: system::Trait + balances::Trait {
	type KittyIndex: Parameter + SimpleArithmetic + Bounded + Default + Copy;
}


#[derive(Debug, Encode, Decode, Default, Clone, PartialEq)]
pub struct Kitty<Hash, Balance> {
    id : Hash,
	dna : [u8; 16],
	price : Balance,
	gen : u64,
}

decl_storage! {
	trait Store for Module<T: Trait> as Kitties {
		/// Stores all the kitties, key is the kitty id / index
		//pub Kitties get(kitty): map u32 => Kitty;
		/// Stores the total number of kitties. i.e. the next kitty index
		//pub KittiesCount get(kitties_count): u32;

		Kitties get(kitty): map T::Hash => Kitty<T::Hash, T::Balance>;
		KittyOwner get(owner_of): map T::Hash => Option<T::AccountId>;

		AllKittiesArray get(kitty_by_index): map u64 => T::Hash;
		AllKittiesCount get(all_kitties_count): u64;
		AllKittiesIndex: map T::Hash => u64;

		OwnedKittiesArray get(kitty_of_owner_by_index): map (T::AccountId, u64) => T::Hash;
		OwnedKittiesCount get(owned_kitty_count): map T::AccountId => u64;
		OwnedKittiesIndex: map T::Hash => u64;

		Nonce: u128;

		
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		fn create(origin) -> Result {
		    let sender = ensure_signed(origin)?;

		    // `nonce` and `random_hash` generation can stay here
		    let nonce = Nonce::get();
		    let random_hash = (<system::Module<T>>::random_seed(), &sender, nonce)
		        .using_encoded(<T as system::Trait>::Hashing::hash);	    
			let payload = (<system::Module<T>>::random_seed(), &sender, 
				<system::Module<T>>::extrinsic_index(), 
				<system::Module<T>>::block_number());
			let dna_hash = payload.using_encoded(blake2_128);


		    // ACTION: Move this collision check to the `mint()` function
		    ensure!(!<KittyOwner<T>>::exists(random_hash), "Kitty already exists");

		    // Creating the `Kitty` object can stay here
		    let new_kitty = Kitty {
		        id: random_hash,
		        dna: dna_hash,
		        price: 0.into(),
		        gen: 0,
		    };

			Self::insert_kitty(sender, random_hash, new_kitty)?;


		    // Nonce update can stay here
		    Nonce::mutate(|n| *n += 1);

		    // ACTION: Move this event to the `mint()` function
		    //Self::deposit_event(RawEvent::Created(sender, random_hash));

		    Ok(())
		}
		
		fn breed_kitty(origin, kitty_id_1: T::Hash, kitty_id_2: T::Hash) -> Result{
            let sender = ensure_signed(origin)?;

            ensure!(<Kitties<T>>::exists(kitty_id_1), "This cat 1 does not exist");
            ensure!(<Kitties<T>>::exists(kitty_id_2), "This cat 2 does not exist");

            let nonce = Nonce::get();
            let random_hash = (<system::Module<T>>::random_seed(), &sender, nonce)
                .using_encoded(<T as system::Trait>::Hashing::hash);

			let payload = (<system::Module<T>>::random_seed(), &sender, 
				<system::Module<T>>::extrinsic_index(), 
				<system::Module<T>>::block_number());
			let dna_hash = payload.using_encoded(blake2_128);

            let kitty_1 = Self::kitty(kitty_id_1);
            let kitty_2 = Self::kitty(kitty_id_2);

            let kitty1_dna = kitty_1.dna;
			let kitty2_dna = kitty_2.dna;

			// Generate a random 128bit value
			let selector = Self::random_value(&sender);
			let mut new_dna = [0u8; 16];

			// Combine parents and selector to create new kitty
			for i in 0..kitty1_dna.len() {
				new_dna[i] = combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
			}

            let new_kitty = Kitty {
                id: random_hash,
                dna: new_dna,
                price: 0.into(),
                gen: cmp::max(kitty_1.gen, kitty_2.gen) + 1,
            };

            Self::insert_kitty(sender, random_hash, new_kitty)?;
			// ACTION: Move all of the kitty related storage updates to the `mint()` function
		    

            // Nonce update can stay here
		    Nonce::mutate(|n| *n += 1);

            Ok(())
        }

	fn transfer(origin, to: T::AccountId, kitty_id: T::Hash) -> Result {
            let sender = ensure_signed(origin)?;

            let owner = Self::owner_of(kitty_id).ok_or("No owner for this kitty")?;
            ensure!(owner == sender, "You do not own this kitty");

            Self::transfer_from(sender, to, kitty_id)?;

            Ok(())
        }

	}
}

fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
	// 作业：实现combine_dna
	// 伪代码：
	// selector.map_bits(|bit, index| if (bit == 1) { dna1 & (1 << index) } else { dna2 & (1 << index) })
	// 注意 map_bits这个方法不存在。只要能达到同样效果，不局限算法
	// 测试数据：dna1 = 0b11110000, dna2 = 0b11001100, selector = 0b10101010, 返回值 0b11100100
	(dna1 & selector) | (dna2 & (!selector))	
}

impl<T: Trait> Module<T> {

	 fn insert_kitty(to: T::AccountId, kitty_id: T::Hash, new_kitty: Kitty<T::Hash, T::Balance>) -> Result {
	        
			// ACTION: Move this `owned_kitty_count` and `new_owned_kitty_count` logic into the `mint()` function
		    let owned_kitty_count = Self::owned_kitty_count(&to);

		    let new_owned_kitty_count = owned_kitty_count.checked_add(1)
		        .ok_or("Overflow adding a new kitty to account balance")?;

		    // ACTION: Move this `all_kitties_count` and `new_all_kitties_count` logic into the `mint()` function
		    let all_kitties_count = Self::all_kitties_count();

		    let new_all_kitties_count = all_kitties_count.checked_add(1)
		        .ok_or("Overflow adding a new kitty to total supply")?;

			
			<Kitties<T>>::insert(kitty_id, new_kitty);
		    <KittyOwner<T>>::insert(kitty_id, &to);

		    <AllKittiesArray<T>>::insert(all_kitties_count, kitty_id);
		    <AllKittiesCount>::put(new_all_kitties_count);
		    <AllKittiesIndex<T>>::insert(kitty_id, all_kitties_count);

		    <OwnedKittiesArray<T>>::insert((to.clone(), owned_kitty_count), kitty_id);
		    <OwnedKittiesCount<T>>::insert(&to, new_owned_kitty_count);
		    <OwnedKittiesIndex<T>>::insert(kitty_id, owned_kitty_count);

			Ok(())	 
	 }

    fn transfer_from(from: T::AccountId, to: T::AccountId, kitty_id: T::Hash) -> Result {
        let owner = Self::owner_of(kitty_id).ok_or("No owner for this kitty")?;

        ensure!(owner == from, "'from' account does not own this kitty");

        let owned_kitty_count_from = Self::owned_kitty_count(&from);
        let owned_kitty_count_to = Self::owned_kitty_count(&to);

        let new_owned_kitty_count_to = owned_kitty_count_to.checked_add(1)
            .ok_or("Transfer causes overflow of 'to' kitty balance")?;

        let new_owned_kitty_count_from = owned_kitty_count_from.checked_sub(1)
            .ok_or("Transfer causes underflow of 'from' kitty balance")?;

        let kitty_index = <OwnedKittiesIndex<T>>::get(kitty_id);
        if kitty_index != new_owned_kitty_count_from {
            let last_kitty_id = <OwnedKittiesArray<T>>::get((from.clone(), new_owned_kitty_count_from));
            <OwnedKittiesArray<T>>::insert((from.clone(), kitty_index), last_kitty_id);
            <OwnedKittiesIndex<T>>::insert(last_kitty_id, kitty_index);
        }

        <KittyOwner<T>>::insert(&kitty_id, &to);
        <OwnedKittiesIndex<T>>::insert(kitty_id, owned_kitty_count_to);

        <OwnedKittiesArray<T>>::remove((from.clone(), new_owned_kitty_count_from));
        <OwnedKittiesArray<T>>::insert((to.clone(), owned_kitty_count_to), kitty_id);

        <OwnedKittiesCount<T>>::insert(&from, new_owned_kitty_count_from);
        <OwnedKittiesCount<T>>::insert(&to, new_owned_kitty_count_to);

        //Self::deposit_event(RawEvent::Transferred(from, to, kitty_id));

        Ok(())
    }

	fn random_value(sender: &T::AccountId) -> [u8; 16] {
		let payload = (<system::Module<T>>::random_seed(), sender, <system::Module<T>>::extrinsic_index(), <system::Module<T>>::block_number());
		payload.using_encoded(blake2_128)
	}
}
