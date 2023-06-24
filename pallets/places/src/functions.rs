#![cfg_attr(not(feature = "std"), no_std)]
use crate::{
	interface::PlacesInterface, structures::*, Bytes, Config, Error, Pallet, PlacesData, PlacesIds,
};
use frame_support::{ensure, sp_runtime::traits::Hash, sp_std::prelude::*};

impl<T: Config> PlacesInterface<T> for Pallet<T> {
	type Error = Error<T>;

	fn _create_place(
		place_type: PlaceType,
		name: Bytes,
		address: Bytes,
		description: T::Hash,
		price_per_night: u64,
		images: Vec<T::Hash>,
		number_of_floors: Option<u8>,
		sender: &T::AccountId,
	) -> Result<T::Hash, Error<T>> {
		// Create a new place
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

	fn _update_place(
		place_id: &T::Hash,
		place_type: Option<PlaceType>,
		name: Option<Bytes>,
		address: Option<Bytes>,
		description: Option<T::Hash>,
		price_per_night: Option<u64>,
		images: Option<Vec<T::Hash>>,
		number_of_floors: Option<u8>,
		sender: &T::AccountId,
	) -> Result<T::Hash, Self::Error> {
		// Retrieve place
		if let Some(mut place_data) = Self::get_place_by_id(place_id) {
			if let Some(new_pt) = place_type {
				place_data.place_type = new_pt;
			}
			if let Some(new_name) = name {
				place_data.name = new_name;
			}
			if let Some(new_address) = address {
				place_data.address = new_address;
			}
			if let Some(new_description) = description {
				place_data.description = new_description;
			}
			if let Some(new_ppn) = price_per_night {
				place_data.price_per_night = new_ppn;
			}
			if let Some(new_images) = images {
				let mut images = place_data.images.clone();

				for image in new_images {
					if !images.contains(&image) {
						images.push(image);
					}
				}

				place_data.images = images;
			}
			if let Some(new_nof) = number_of_floors {
				place_data.number_of_floors = new_nof;
			}

			place_data.on_chain_update = Some(AuditTrail::new(sender.clone()));

			// Make persistance
			<PlacesData<T>>::insert(place_id, place_data);

			// Logging to the console on debug level
			log::debug!(target: "did", "A Place with ID ➡ {:?} has been updated.", place_id);

			return Ok(*place_id);
		}
		Err(Error::<T>::UnhandledException)
	}

	fn _remove_place(place_id: &<T as frame_system::Config>::Hash) -> Result<T::Hash, Self::Error> {
		// Retrieve place
		if let Some(_) = Self::get_place_by_id(place_id) {
			// Make persistance
			<PlacesIds<T>>::mutate(|pids| {
				if let Some(idx) = pids.iter().position(|x| x == place_id) {
					pids.swap_remove(idx);
				}
			});
			<PlacesData<T>>::remove(place_id);

			return Ok(*place_id);
		}
		Err(Error::<T>::PlaceNotFound)
	}
}
