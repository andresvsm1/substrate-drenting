#![cfg_attr(not(feature = "std"), no_std)]
use crate::{
	interface::PlacesInterface, structures::*, Bytes, Config, Error, Pallet, PlacesData, PlacesIds,
};
use frame_support::{
	ensure,
	sp_runtime::traits::Hash,
	sp_std::{collections::btree_set::BTreeSet, prelude::*},
};

impl<T: Config> PlacesInterface<T> for Pallet<T> {
	type Error = Error<T>;

	fn _create_place(
		place_type: PlaceType,
		name: Bytes,
		address: Bytes,
		description: T::Hash,
		price_per_night: u64,
		checkin_hour: u32,
		checkout_hour: u32,
		images: Vec<T::Hash>,
		number_of_floors: Option<u8>,
		sender: &T::AccountId,
	) -> Result<T::Hash, Error<T>> {
		Self::ensure_checkin_checkout_hours_are_correct(checkin_hour, checkout_hour)?;

		// Create a new place
		let place_data: PlaceData<T> = PlaceData::new(
			place_type,
			name,
			address,
			description,
			price_per_night,
			checkin_hour,
			checkout_hour,
			images.into_iter().collect(),
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
		checkin_hour: Option<u32>,
		checkout_hour: Option<u32>,
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

			if let Some(cih) = checkin_hour {
				place_data.checkin_hour = cih;
			}

			if let Some(coh) = checkout_hour {
				place_data.checkout_hour = coh;
			}

			Self::ensure_checkin_checkout_hours_are_correct(
				place_data.checkin_hour,
				place_data.checkout_hour,
			)?;

			if let Some(new_images) = images {
				let new_images_set: BTreeSet<T::Hash> = new_images.into_iter().collect();
				let images_union = new_images_set.union(&place_data.images).cloned().collect();
				place_data.images = images_union;
			}

			if let Some(new_nof) = number_of_floors {
				place_data.number_of_floors = new_nof;
			}

			place_data.on_chain_update = Some(AuditTrail::new(sender.clone()));

			// Make persistance
			<PlacesData<T>>::insert(place_id, place_data);

			// Logging to the console on debug level
			log::debug!(target: "did", "A Place with ID ➡ {:?} has been updated.", place_id);

			return Ok(*place_id)
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

			return Ok(*place_id)
		}
		Err(Error::<T>::PlaceNotFound)
	}
}

/// Auxiliar functions implementation
impl<T: Config> Pallet<T> {
	fn ensure_checkin_checkout_hours_are_correct(
		checkin_hour: u32,
		checkout_hour: u32,
	) -> Result<(), Error<T>> {
		ensure!(
			checkin_hour > checkout_hour,
			Error::<T>::CheckoutHourCannotBeGreaterThanCheckinHour
		);
		ensure!(checkin_hour > 0 && checkin_hour <= 23, Error::<T>::BadHoursProvided);
		ensure!(checkout_hour > 0 && checkout_hour <= 23, Error::<T>::BadHoursProvided);

		Ok(())
	}
}
