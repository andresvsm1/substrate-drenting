#![cfg_attr(not(feature = "std"), no_std)]
use crate::{
	interface::BookingsInterface,
	structures::{BookingData, BookingHashingData},
	BalanceOf, BookingsData, BookingsIds, Config, Error, Pallet, PlaceBookings,
};
use frame_support::{
	ensure,
	sp_runtime::{traits::Hash, DispatchError},
	sp_std::cmp::Ordering,
};
pub use pallet_places::Error as PlacesError;

impl<T: Config> BookingsInterface<T> for Pallet<T> {
	type Error = Error<T>;

	fn _create_booking(
		place_id: T::Hash,
		start_date: T::Moment,
		end_date: T::Moment,
		sender: T::AccountId,
		amount: BalanceOf<T>,
	) -> Result<T::Hash, DispatchError> {
		if let Some(place) = pallet_places::Pallet::<T>::get_place_by_id(place_id) {
			ensure!(&place.owner != &sender, Error::<T>::CannotBookOwnedPlace);
			if !Self::check_availability(place_id, start_date, end_date) {
				return Err((Error::<T>::BookingDatesNotAvailable).into());
			}

			let booking_data: BookingData<T> =
				BookingData::new(place.owner, sender, start_date, end_date, amount);

			let hashing_data = BookingHashingData::from(booking_data.clone());
			let booking_id = T::Hashing::hash_of(&hashing_data);

			// Ensure id does not exists
			ensure!(!<BookingsData<T>>::contains_key(booking_id), Error::<T>::BookingAlreadyExists);

			// Make persistance
			<BookingsData<T>>::insert(booking_id, booking_data);
			<BookingsIds<T>>::append(booking_id);
			<PlaceBookings<T>>::mutate(place_id, |booking_list| booking_list.push(booking_id));

			// Logging to the console on debug level
			log::debug!(target: "did", "A new Booking with ID ➡ {:?} has been placed.", booking_id);

			return Ok(booking_id);
		}
		Err((PlacesError::<T>::PlaceNotFound).into())
	}

	fn _update_booking(
		booking_id: &<T>::Hash,
		start_date: <T>::Moment,
		end_date: <T>::Moment,
		sender: <T>::AccountId,
	) -> Result<<T>::Hash, DispatchError> {
		todo!()
	}

	fn _cancel_booking(
		booking_id: &<T>::Hash,
		sender: <T>::AccountId,
	) -> Result<<T>::Hash, DispatchError> {
		todo!()
	}
}

impl<T: Config> Pallet<T> {
	/// Checks the availability of a place for booking.
	///
	/// This function determines whether a place is available for booking
	/// within the specified `start_date` and `end_date` range.
	///
	/// # Arguments
	///
	/// * `place_id` - The identifier of the place to check availability for.
	/// * `start_date` - The start date of the booking range.
	/// * `end_date` - The end date of the booking range.
	///
	/// # Returns
	///
	/// Returns `true` if the place is available for booking, meaning it does not overlap with any existing bookings.
	/// Returns `false` if the place is not available for booking, indicating an overlap with an existing booking.
	///
	pub fn check_availability(
		place_id: T::Hash,
		start_date: T::Moment,
		end_date: T::Moment,
	) -> bool {
		let place_bookings = Self::get_place_bookings(place_id);
		for booking_id in place_bookings {
			if let Some(booking) = Self::get_booking_by_id(booking_id) {
				match (start_date.cmp(&booking.start_date), end_date.cmp(&booking.end_date)) {
					(Ordering::Less, Ordering::Greater)
					| (Ordering::Equal, Ordering::Equal)
					| (Ordering::Greater, Ordering::Less) => {
						return false;
					},
					_ => {},
				}
			}
		}

		true
	}
}
