use codec::{Decode, Encode};
use runtime_io::blake2_128;
use support::{decl_module, decl_storage, dispatch::Result, ensure, StorageMap, StorageValue};
use system::ensure_signed;

pub trait Trait: system::Trait {}

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
            let new_count = count.checked_add(1)
                .ok_or("Overflow adding a new kitty to total supply")?;

            let payload = (<system::Module<T>>::random_seed(), sender, <system::Module<T>>::extrinsic_index(), <system::Module<T>>::block_number());
            let dna = payload.using_encoded(blake2_128);
            let kitty = Kitty(dna);
            Kitties::insert(count, kitty);
            KittiesCount::put(new_count);
        }

        pub fn breed(origin, father: u32, mother: u32) -> Result {
            let sender = ensure_signed(origin)?;

            ensure!(Kitties::exists(father), "First kitty does not exists");
            ensure!(Kitties::exists(mother), "Second kitty does not exists");

            let count = Self::kitties_count();
            let new_count = count.checked_add(1)
                .ok_or("Overflow adding a new kitty to total supply")?;

            let father = Self::kitty(father);
            let mother = Self::kitty(mother);

            let random = (<system::Module<T>>::random_seed(), sender, <system::Module<T>>::extrinsic_index(), <system::Module<T>>::block_number(), father.0, mother.0)
                .using_encoded(blake2_128);
            let mut dna = [0_u8; 16];
            for (((dna_father, dna_mother), dna_child), r) in father.0.iter().zip(&mother.0).zip(&mut dna).zip(&random) {
                if r % 2 == 0 {
                    *dna_child = *dna_father;
                } else {
                    *dna_child = *dna_mother;
                }
            }

            let child = Kitty(dna);
            Kitties::insert(count, child);
            KittiesCount::put(new_count);

            Ok(())
        }
    }
}
