#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::Randomness,
    RuntimeDebug, StorageDoubleMap, StorageValue,
};
use frame_system::ensure_signed;
use sp_io::hashing::blake2_128;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
pub struct Kitty(pub [u8; 16]);

#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq)]
pub enum KittyGender {
    Male,
    Female,
}

impl Kitty {
    pub fn gender(&self) -> KittyGender {
        if self.0[0] % 2 == 0 {
            KittyGender::Male
        } else {
            KittyGender::Female
        }
    }
}

pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    //type RandomnessSource: Randomness<H256>;
}

decl_storage! {
    trait Store for Module<T: Config> as Kitties {
        /// Stores all the kitties, key is the kitty id
        pub Kitties get(fn kitties): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) u32 => Option<Kitty>;
        /// Stores the next kitty ID
        pub NextKittyId get(fn next_kitty_id): u32;
    }
}

decl_event! {
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
    {
        /// A kitty is created. \[owner, kitty_id, kitty\]
        KittyCreated(AccountId, u32, Kitty),
        // / A new kitten is bred. \[owner, kitty_id, kitty\]
        KittyBred(AccountId, u32, Kitty),
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        KittiesIdOverflow,
        InvalidKittyId,
        SameGender,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Create a new kitty
        #[weight = 1000]
        pub fn create(origin) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let next_id = Self::get_next_kitty_id()?;
            let dna = Self::random_value(&sender);
            let kitty = Kitty(dna);
            Kitties::<T>::insert(&sender, next_id, kitty.clone());
            Self::deposit_event(RawEvent::KittyCreated(sender, next_id, kitty));
             Ok(())
        }

        /// Breed kitties
        #[weight = 1000]
        pub fn breed(origin, kitty_id_1: u32, kitty_id_2: u32) {
            //Checks First
            //ensure sender is signed
            let sender = ensure_signed(origin)?;
            //ensure both kitty ids supplied are valid
            let kitty1 = Self::kitties(&sender, kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
            let kitty2 = Self::kitties(&sender, kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

            //ensure sender is not receiver
            ensure!(kitty1.gender() != kitty2.gender(), Error::<T>::SameGender);

            let kitty_id = Self::get_next_kitty_id()?;

            let (kitty1_dna, kitty2_dna) = (kitty1.0, kitty2.0);

            let selector = Self::random_value(&sender);
            //create new kitty dna
            let mut new_dna = [0u8; 16];

            // Combine parents and selector to create new kitty
            for i in 0..kitty1_dna.len() {
                new_dna[i] = combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
            }
            //new kitty from dna
            let new_kitty = Kitty(new_dna);
            //Now insert into map
            Kitties::<T>::insert(&sender, kitty_id, &new_kitty);
            //tell the listening world
            Self::deposit_event(RawEvent::KittyBred(sender, kitty_id, new_kitty));
        }
    }
}

pub fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
    // TODO: finish this implementation
    // selector[bit_index] == 0 -> use dna1[bit_index]
    // selector[bit_index] == 1 -> use dna2[bit_index]
    // e.g.
    // selector = 0b00000001
    // dna1		= 0b10101010
    // dna2		= 0b00001111
    // result	= 0b10101011

    // not(selector) and dna1 or selector and dna2
    //some cool bit arithmetic here
    //(!selector.bitand(dna1)).bitor(selector.bitand(dna2))

    (!selector & dna1) | (selector & dna2)
}

impl<T: Config> Module<T> {
    fn get_next_kitty_id() -> sp_std::result::Result<u32, DispatchError> {
        NextKittyId::try_mutate(|next_id| -> sp_std::result::Result<u32, DispatchError> {
            let current_id = *next_id;
            *next_id = next_id
                .checked_add(1)
                .ok_or(Error::<T>::KittiesIdOverflow)?;
            Ok(current_id)
        })
    }

    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        // TODO: finish this implementation
        // we'll use the senders id as the subject that prevents this method from returning same value always
        // let random_value = <pallet_randomness_collective_flip::Module<T>>::random(sender.to_string().as_bytes());
        // random_value.using_encoded(blake2_128(data))
        let payload = (
            <pallet_randomness_collective_flip::Module<T> as Randomness<T::Hash>>::random_seed(),
            sender,
            <frame_system::Module<T>>::extrinsic_index(),
        );
        payload.using_encoded(blake2_128)
    }
}
