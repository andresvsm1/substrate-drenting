#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::{ensure, sp_runtime::traits::Hash, sp_std::prelude::*};

use crate::{structures::*, Bytes, Config, Error, Pallet, PlacesData, PlacesIds};
impl<T: Config> Pallet<T> {
	pub fn _create_place(
		place_type: PlaceType,
		name: Bytes,
		address: Bytes,
		description: T::Hash,
		price_per_night: u64,
		images: Vec<T::Hash>,
		number_of_floors: Option<u8>,
		sender: &T::AccountId,
	) -> Result<T::Hash, Error<T>> {
		// Create a new master ddo
		let place_data: PlaceData<T> = PlaceData::new(
			place_type,
			name,
			address,
			description,
			price_per_night,
			images,
			number_of_floors,
			sender.clone(),
		);

		let hashing_data = PlaceHashingData::from(place_data.clone());
		let place_id = T::Hashing::hash_of(&hashing_data);

		// Ensure id does not exists
		ensure!(!<PlacesData<T>>::contains_key(place_id), Error::<T>::PlaceAlreadyExists);

		// Make persistance
		<PlacesData<T>>::insert(place_id, place_data);
		<PlacesIds<T>>::append(place_id);

		// Logging to the console on debug level
		log::debug!(target: "did", "A new Place with ID ➡ {:?} has been created.", place_id);

		Ok(place_id)
	}
}
