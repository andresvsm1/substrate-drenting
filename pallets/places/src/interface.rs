use crate::{Bytes, Config, PlaceType};
use frame_support::sp_std::prelude::*;

/// Interface for Places pallet
pub trait PlacesInterface<T: Config> {
	type Error;

	/// Store a new Place with its information.
	///
	/// This function creates a new Place with the specified details and stores it in the
	/// blockchain. The Place can represent different types of accommodations like hotels,
	/// apartments, etc. It returns a unique identifier (`Hash`) for the created Place.
	///
	/// # Arguments
	///
	/// * `place_type` - The type of the Place, e.g., Hotel, Apartment, etc.
	/// * `name` - The name of the Place.
	/// * `address` - The address of the Place.
	/// * `description` - A hash of the description of the Place (stored separately).
	/// * `price_per_night` - The price per night for booking the Place.
	/// * `checkin_hour` - The hour when guests can check-in (in 24-hour format).
	/// * `checkout_hour` - The hour when guests must check-out (in 24-hour format).
	/// * `images` - A list of hashes representing images of the Place (stored separately).
	/// * `number_of_floors` - An optional field indicating the number of floors in the Place.
	/// * `sender` - The account identifier of the sender creating the Place.
	///
	/// # Returns
	///
	/// Returns a `Result` indicating the success or failure of the Place creation operation. If the
	/// operation is successful, the `Result` contains the unique identifier (`Hash`) for the
	/// created Place. Otherwise, it contains an error indicating the reason for failure.
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
	) -> Result<T::Hash, Self::Error>;

	/// Update an existing Place's information.
	///
	/// This function updates the information of an existing Place with the specified `place_id`.
	/// Any of the provided optional fields (`place_type`, `name`, `address`, `description`,
	/// `price_per_night`, `checkin_hour`, `checkout_hour`, `images`, `number_of_floors`) can be set
	/// to `None` to indicate that the corresponding attribute should remain unchanged.
	/// The `sender` account identifier must have the necessary permissions to update the Place.
	/// The function returns a unique identifier (`Hash`) for the updated Place.
	///
	/// # Arguments
	///
	/// * `place_id` - The identifier of the Place to update.
	/// * `place_type` - An optional new type of the Place (if provided).
	/// * `name` - An optional new name of the Place (if provided).
	/// * `address` - An optional new address of the Place (if provided).
	/// * `description` - An optional new hash of the description of the Place (if provided).
	/// * `price_per_night` - An optional new price per night for booking the Place (if provided).
	/// * `checkin_hour` - An optional new hour when guests can check-in (in 24-hour format, if
	///   provided).
	/// * `checkout_hour` - An optional new hour when guests must check-out (in 24-hour format, if
	///   provided).
	/// * `images` - An optional new list of hashes representing images of the Place (if provided).
	/// * `number_of_floors` - An optional new field indicating the number of floors in the Place
	///   (if provided).
	/// * `sender` - The account identifier of the sender updating the Place.
	///
	/// # Returns
	///
	/// Returns a `Result` indicating the success or failure of the Place update operation. If the
	/// operation is successful, the `Result` contains the unique identifier (`Hash`) for the
	/// updated Place. Otherwise, it contains an error indicating the reason for failure.
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
	) -> Result<T::Hash, Self::Error>;

	/// Delete a Place.
	///
	/// This function removes the Place associated with the specified `place_id`.
	/// The `sender` account identifier must have the necessary permissions to delete the Place.
	/// The function returns a unique identifier (`Hash`) for the deleted Place.
	///
	/// # Arguments
	///
	/// * `place_id` - The identifier of the Place to delete.
	///
	/// # Returns
	///
	/// Returns a `Result` indicating the success or failure of the Place deletion operation. If the
	/// operation is successful, the `Result` contains the unique identifier (`Hash`) for the
	/// deleted Place. Otherwise, it contains an error indicating the reason for failure.
	fn _remove_place(place_id: &T::Hash) -> Result<T::Hash, Self::Error>;
}
