#![cfg_attr(not(feature = "std"), no_std)]

use super::{Bytes, Config};
use codec::{Decode, Encode};
use frame_support::sp_std::{prelude::*, collections::btree_set::BTreeSet};
use scale_info::TypeInfo;

// Struct to keep track of chain interactions
#[derive(Eq, PartialEq, Encode, Decode, TypeInfo, Clone, Debug)]
#[scale_info(skip_type_params(T))]
pub struct AuditTrail<T: Config> {
	pub account: T::AccountId,
	pub block: T::BlockNumber,
	pub time: T::Moment,
}

impl<T: Config> AuditTrail<T> {
	pub fn new(account: T::AccountId) -> Self {
		AuditTrail {
			account,
			block: <frame_system::Pallet<T>>::block_number(),
			time: <pallet_timestamp::Pallet<T>>::now(),
		}
	}
}

#[derive(Eq, PartialEq, Encode, Decode, TypeInfo, Clone, Debug)]
pub enum PlaceType {
	Apartment,
	House,
	Van,
	Boat,
}

#[derive(Eq, PartialEq, Encode, Decode, TypeInfo, Clone, Debug)]
#[scale_info(skip_type_params(T))]

pub struct PlaceData<T: Config> {
	/// The palce type.
	pub place_type: PlaceType,
	/// The name of the place.
	pub name: Bytes,
	/// The address of the place.
	pub address: Bytes,
	/// The description of the place. Just a reference to an external description, as this might be too big for the chain.
	pub description: T::Hash,
	/// The price of the place.
	pub price_per_night: u64,
	/// The hour picked by the owner for the checkin 0-23
	pub checkin_hour: u32,
	/// The hour picked by the owner for the checkout 0-23
	pub checkout_hour: u32,
	/// Whether the place is available for rent. Controls if the place can receive bookings or not.
	pub active: bool,
	/// The images of the place. References to external images.
	pub images: BTreeSet<T::Hash>,
	/// The number of floors of the house, in case it has more than
	pub number_of_floors: u8,
	/// The owner of the place
	pub owner: T::AccountId,
	/// Audit Trailing
	pub on_chain_creation: AuditTrail<T>,
	pub on_chain_update: Option<AuditTrail<T>>,
}

impl<T: Config> PlaceData<T> {
	pub fn new(
		place_type: PlaceType,
		name: Bytes,
		address: Bytes,
		description: T::Hash,
		price_per_night: u64,
		checkin_hour: u32,
		checkout_hour: u32,
		images: BTreeSet<T::Hash>,
		number_of_floors: Option<u8>,
		created_by: T::AccountId,
	) -> Self {
		PlaceData {
			place_type,
			name,
			address,
			description,
			price_per_night,
			checkin_hour,
			checkout_hour,
			active: true,
			images,
			number_of_floors: number_of_floors.unwrap_or(1),
			owner: created_by.clone(),
			on_chain_creation: AuditTrail::<T>::new(created_by),
			on_chain_update: None,
		}
	}
}

#[derive(Encode, Decode, Clone, PartialEq, TypeInfo, Debug)]
#[scale_info(skip_type_params(T))]
pub struct PlaceHashingData<T: Config> {
	pub place_type: PlaceType,
	pub name: Bytes,
	pub address: Bytes,
	pub description: T::Hash,
	pub images: BTreeSet<T::Hash>,
	pub number_of_floors: u8,
}

impl<T: Config> From<PlaceData<T>> for PlaceHashingData<T> {
	fn from(from: PlaceData<T>) -> Self {
		let PlaceData { place_type, name, address, description, images, number_of_floors, .. } =
			from;

		Self { place_type, name, address, description, images, number_of_floors }
	}
}
