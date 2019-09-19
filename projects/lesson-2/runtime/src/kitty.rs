use codec::{Decode, Encode};
use support::{decl_event, decl_module, decl_storage};
use byteorder::{ByteOrder, LittleEndian};
use runtime_io::blake2_128;

#[derive(Debug, Default, Clone, PartialEq, Encode, Decode)]
pub struct Kitty<Hash, Balance> {
    id: Hash,
    dna: u128,
    price: Balance,
}

impl<Hash, Balance> Kitty<Hash, Balance> {
    pub fn new(id: Hash, dna: u128, price: Balance) -> Self {
        Self { id, dna, price }
    }
}

pub trait Trait: balances::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Hash = <T as system::Trait>::Hash,
        Balance = <T as balances::Trait>::Balance,
    {
        Created(AccountId, Hash, Balance),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as KittyStorage {
        AllKitties get(kitty): map u64 => Kitty<T::Hash, T::Balance>;
        AllKittiesLen get(all_kitties_len): u64;
        UserKitties get(kitty_of): map (T::AccountId, u64) => u64;
        UserKittiesLen get(kitty_len_of): map T::AccountId => u64;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;
    }
}

impl<T: Trait> Module<T> {
    fn dna() -> u128 {
        let dna_hash = (<system::Module<T>>::random_seed())
            .using_encoded(blake2_128);
        LittleEndian::read_u128(&dna_hash)
    }
}
