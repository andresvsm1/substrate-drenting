use crate::{Bytes, PlaceType};
use frame_support::sp_std::prelude::*;

/// Interface for Places pallet
pub trait PlacesInterface<T: frame_system::Config> {
	type Error;

	/// Store a Place with its information
	fn _create_place(
		place_type: PlaceType,
		name: Bytes,
		address: Bytes,
		description: T::Hash,
		price_per_night: u64,
		images: Vec<T::Hash>,
		number_of_floors: Option<u8>,
		sender: &T::AccountId,
	) -> Result<T::Hash, Self::Error>;

	/// Update a Place information
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
	) -> Result<T::Hash, Self::Error>;

	/// Delete Place
	fn _remove_place(place_id: &T::Hash) -> Result<T::Hash, Self::Error>;
}
