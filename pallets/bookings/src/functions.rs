#![cfg_attr(not(feature = "std"), no_std)]
use crate::{
	interface::BookingsInterface,
	structures::{BookingData, BookingHashingData},
	BalanceOf, BookingState, BookingsData, BookingsIds, Config, Error, Pallet,
	PendingBookingWithdraws, PlaceBookings,
};
use frame_support::{
	ensure,
	sp_runtime::{traits::Hash, DispatchError, SaturatedConversion},
	sp_std::{cmp::Ordering, vec::Vec},
	traits::{tokens::ExistenceRequirement, Currency, ReservableCurrency},
};
use pallet_places::Error as PlacesError;

impl<T: Config> BookingsInterface<T> for Pallet<T> {
	type Error = Error<T>;

	fn _create_booking(
		sender: T::AccountId,
		place_id: T::Hash,
		start_date: T::Moment,
		end_date: T::Moment,
		amount: BalanceOf<T>,
	) -> Result<T::Hash, DispatchError> {
		ensure!(end_date > start_date, Error::<T>::InvalidDates);
		if let Some(place) = pallet_places::Pallet::<T>::get_place_by_id(place_id) {
			ensure!(&place.owner != &sender, Error::<T>::CannotBookOwnedPlace);

			let formatted_start_date = Self::modify_timestamp(start_date, place.checkin_hour)?;
			let formatted_end_date = Self::modify_timestamp(end_date, place.checkout_hour)?;

			let current_moment = <pallet_places::pallet_timestamp::Pallet<T>>::now();
			ensure!(formatted_start_date > current_moment, Error::<T>::InvalidStartDate);

			if !Self::check_availability(place_id, formatted_start_date, formatted_end_date) {
				return Err(Error::<T>::BookingDatesNotAvailable.into())
			}

			let expected_amount = Self::calculate_total_amount(
				formatted_start_date,
				formatted_end_date,
				place.price_per_night,
			)?;

			match amount.saturated_into::<u64>().cmp(&expected_amount) {
				Ordering::Less =>
					return Err(DispatchError::Other("Amount provided is less that required")),
				Ordering::Greater =>
					return Err(DispatchError::Other("Amount provided exceeds the requested one")),
				Ordering::Equal => {},
			};
			ensure!(T::Currency::can_reserve(&sender, amount), Error::<T>::NotEnoughFreeBalance);

			let booking_data: BookingData<T> = BookingData::new(
				place_id,
				place.owner.clone(),
				sender.clone(),
				formatted_start_date,
				formatted_end_date,
				amount,
			);

			let hashing_data = BookingHashingData::from(booking_data.clone());
			let booking_id = T::Hashing::hash_of(&hashing_data);

			// Ensure id does not exists
			ensure!(!<BookingsData<T>>::contains_key(booking_id), Error::<T>::BookingAlreadyExists);

			// Make persistance
			<BookingsData<T>>::insert(booking_id, booking_data);
			<BookingsIds<T>>::append(booking_id);
			<PlaceBookings<T>>::mutate(place_id, |booking_list| booking_list.push(booking_id));
			// Lock users funds and store a reference
			T::Currency::reserve(&sender, amount)?;
			<PendingBookingWithdraws<T>>::mutate(&place.owner, |booking_withdraws| {
				booking_withdraws.push((booking_id, amount));
			});

			// Logging to the console on debug level
			log::debug!(target: "did", "A new Booking with ID ➡ {:?} has been placed.", booking_id);

			return Ok(booking_id)
		}
		Err(PlacesError::<T>::PlaceNotFound.into())
	}

	fn _update_booking(
		sender: <T>::AccountId,
		booking_id: &<T>::Hash,
		place_id: &T::Hash,
		start_date: <T>::Moment,
		end_date: <T>::Moment,
	) -> Result<<T>::Hash, DispatchError> {
		todo!()
	}

	fn _cancel_booking(
		sender: <T>::AccountId,
		booking_id: &<T>::Hash,
	) -> Result<<T>::Hash, DispatchError> {
		todo!()
	}

	fn _confirm_booking(
		sender: <T>::AccountId,
		booking_id: &<T>::Hash,
	) -> Result<<T>::Hash, DispatchError> {
		if let Some(mut booking) = Self::get_booking_by_id(booking_id) {
			ensure!(sender == booking.host, Error::<T>::NotPlaceOwner);
			ensure!(booking.state == BookingState::Created, Error::<T>::WrongState);
			let current_moment = <pallet_places::pallet_timestamp::Pallet<T>>::now();
			ensure!(current_moment < booking.start_date, Error::<T>::CannotConfirmOutdatedBooking);

			for booking_id_to_cancel in Self::get_overlapping_bookings(
				booking.place_id,
				*booking_id,
				booking.start_date,
				booking.end_date,
			) {
				Self::_do_cancel_booking(
					booking.place_id,
					booking_id_to_cancel,
					booking.host.clone(),
					booking.guest.clone(),
					booking.amount,
				)?;
			}

			// Make persistence
			booking.state = BookingState::Confirmed;
			<BookingsData<T>>::insert(booking_id, booking);
			return Ok(*booking_id)
		}
		Err(Error::<T>::BookingNotFound.into())
	}

	fn _reject_booking(
		sender: <T>::AccountId,
		booking_id: &<T>::Hash,
	) -> Result<<T>::Hash, DispatchError> {
		if let Some(booking) = Self::get_booking_by_id(booking_id) {
			ensure!(sender == booking.host, Error::<T>::NotPlaceOwner);
			ensure!(booking.state == BookingState::Created, Error::<T>::WrongState);
			Self::_do_cancel_booking(
				booking.place_id,
				*booking_id,
				booking.host.clone(),
				booking.guest.clone(),
				booking.amount,
			)?;
			return Ok(*booking_id)
		}
		Err(Error::<T>::BookingNotFound.into())
	}

	fn _checkin(
		sender: <T>::AccountId,
		booking_id: &<T>::Hash,
	) -> Result<<T>::Hash, DispatchError> {
		if let Some(mut booking) = Self::get_booking_by_id(booking_id) {
			ensure!(sender == booking.guest, Error::<T>::NotPlaceGuest);
			ensure!(booking.state == BookingState::Confirmed, Error::<T>::WrongState);
			// For production, check start time is correct
			let current_moment = <pallet_places::pallet_timestamp::Pallet<T>>::now();
			ensure!(current_moment >= booking.start_date, Error::<T>::CheckinNotAvailableYet);
			// Make persistence
			booking.state = BookingState::OwnerCanWithdraw;
			<BookingsData<T>>::insert(booking_id, booking);

			return Ok(*booking_id)
		}
		Err(Error::<T>::BookingNotFound.into())
	}

	fn _withdraw_booking(
		sender: <T>::AccountId,
		booking_id: &<T>::Hash,
	) -> Result<<T>::Hash, DispatchError> {
		if let Some(booking) = Self::get_booking_by_id(booking_id) {
			return match booking.state {
				BookingState::Rejected | BookingState::UserCanWithdraw =>
					Self::_guest_withdraw_booking(sender, booking_id), // Unreserve funds for guest
				BookingState::Withdrawable => todo!(), // Check Refund Policy
				BookingState::OwnerCanWithdraw => Self::_host_withdraw_booking(sender, booking_id), /* Transfer reserved funds from guest to host */
				_ => return Err(Error::<T>::WrongState.into()),
			}
		}
		Err(Error::<T>::BookingNotFound.into())
	}
}

/// Auxiliar functions implementation
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
	/// * `skip_booking_id` - If provided, it omits the booking id in the validation process.
	///
	/// # Returns
	///
	/// Returns `true` if the place is available for booking, meaning it does not overlap with any
	/// existing bookings. Returns `false` if the place is not available for booking, indicating an
	/// overlap with an existing booking.
	fn check_availability(place_id: T::Hash, start_date: T::Moment, end_date: T::Moment) -> bool {
		let place_bookings = Self::get_place_bookings(place_id);
		for booking_id in place_bookings {
			if let Some(booking) = Self::get_booking_by_id(booking_id) {
				if booking.state == BookingState::Created {
					continue
				}
				match (start_date.cmp(&booking.start_date), end_date.cmp(&booking.end_date)) {
					(Ordering::Less, Ordering::Greater) |
					(Ordering::Equal, Ordering::Equal) |
					(Ordering::Greater, Ordering::Less) => return false,
					_ => {},
				}
			}
		}

		true
	}

	/// Get overlapping bookings for a specified place and booking period.
	///
	/// This function retrieves a list of booking identifiers (`Hash`) that overlap with the
	/// provided booking period. It iterates through the existing bookings associated with the given
	/// `place_id` and compares their start and end dates with the provided `booking_start_date` and
	/// `booking_end_date`. If there is an overlap in the booking periods, the booking identifier is
	/// added to the list of `bookings_to_cancel`. The `booking_id_to_confirm` is excluded from the
	/// overlapping check, allowing the booking confirmation without conflict with itself.
	///
	/// # Arguments
	///
	/// * `place_id` - The unique identifier of the place for which overlapping bookings are to be
	///   found.
	/// * `booking_id_to_confirm` - The unique identifier of the booking to be confirmed, excluded
	///   from the overlapping check.
	/// * `booking_start_date` - The start date of the booking to be confirmed.
	/// * `booking_end_date` - The end date of the booking to be confirmed.
	///
	/// # Returns
	///
	/// Returns a vector (`Vec`) of booking identifiers (`Hash`) representing the list of
	/// overlapping bookings. If no overlapping bookings are found, the vector will be empty.
	fn get_overlapping_bookings(
		place_id: T::Hash,
		booking_id_to_confirm: T::Hash,
		booking_start_date: T::Moment,
		booking_end_date: T::Moment,
	) -> Vec<T::Hash> {
		let mut bookings_to_cancel: Vec<T::Hash> = Vec::new();

		let place_bookings = Self::get_place_bookings(place_id);
		for booking_id in place_bookings {
			if booking_id == booking_id_to_confirm {
				continue
			}
			if let Some(booking) = Self::get_booking_by_id(booking_id) {
				match (
					booking_start_date.cmp(&booking.start_date),
					booking_end_date.cmp(&booking.end_date),
				) {
					(Ordering::Less, Ordering::Greater) |
					(Ordering::Equal, Ordering::Equal) |
					(Ordering::Greater, Ordering::Less) => {
						bookings_to_cancel.push(booking_id);
					},
					_ => {},
				}
			}
		}

		bookings_to_cancel
	}

	/// Perform the cancellation of a booking.
	///
	/// This function cancels a specific booking identified by `booking_id`. It updates the
	/// booking's status and releases any held funds. The cancellation is initiated by the `host` or
	/// the `guest`, and the `amount` is returned accordingly.
	///
	/// # Arguments
	///
	/// * `place_id` - The unique identifier of the place associated with the booking to be
	///   canceled.
	/// * `booking_id` - The unique identifier of the booking to be canceled.
	/// * `host` - The account identifier of the host initiating the booking cancellation.
	/// * `guest` - The account identifier of the guest initiating the booking cancellation.
	/// * `amount` - The payment amount associated with the booking to be canceled.
	///
	/// # Returns
	///
	/// Returns a `Result` indicating the success or failure of the booking cancellation operation.
	/// If the operation is successful, the booking is canceled, and the `Result` contains no error.
	/// If the booking cancellation fails, the `Result` contains a `DispatchError` explaining the
	/// reason for failure.
	fn _do_cancel_booking(
		place_id: T::Hash,
		booking_id: T::Hash,
		host: T::AccountId,
		guest: T::AccountId,
		amount: BalanceOf<T>,
	) -> Result<(), DispatchError> {
		// Make persistance
		if let Some(mut booking_data) = Self::get_booking_by_id(booking_id) {
			booking_data.state = BookingState::Rejected;
			<BookingsData<T>>::insert(booking_id, booking_data);
		}

		<PlaceBookings<T>>::try_mutate(place_id, |booking_list| {
			if let Some(ind) = booking_list.iter().position(|&bid| bid == booking_id) {
				booking_list.swap_remove(ind);
				return Ok(())
			}
			Err(())
		})
		.map_err(|_| <Error<T>>::BookingNotFound)?;

		T::Currency::unreserve(&guest, amount);

		<PendingBookingWithdraws<T>>::mutate(&host, |booking_withdraws| {
			for (index, tuple) in booking_withdraws.iter().enumerate() {
				// Check if the first element of the tuple matches the target value
				if tuple.0 == booking_id {
					// Perform the swap_remove operation
					booking_withdraws.swap_remove(index);
					break
				}
			}
		});

		<PendingBookingWithdraws<T>>::mutate(&guest, |booking_withdraws| {
			booking_withdraws.push((booking_id, amount))
		});

		Ok(())
	}

	/// Perform the withdrawal as the OWNER of the place.
	///
	/// # Arguments
	///
	/// * `sender` - The caller of the function.
	/// * `booking_id` - The unique identifier of the booking to be withdrawed.
	///
	/// # Returns
	///
	/// Returns a `Result` indicating the success or failure of the booking withdrawal function. If
	/// the OWNER was able to claim all the funds from its booking, it simply returns the same
	/// booking_id, otherwise, it will throw an specific error.
	fn _host_withdraw_booking(
		sender: <T>::AccountId,
		booking_id: &T::Hash,
	) -> Result<T::Hash, DispatchError> {
		if let Some(mut booking_data) = Self::get_booking_by_id(booking_id) {
			ensure!(sender == booking_data.host, Error::<T>::NotPlaceOwner);

			// Try to withdraw first
			T::Currency::unreserve(&booking_data.guest, booking_data.amount);
			T::Currency::transfer(
				&booking_data.guest,
				&booking_data.host,
				booking_data.amount,
				ExistenceRequirement::KeepAlive,
			)?;

			// Now persist new state
			<PendingBookingWithdraws<T>>::mutate(&booking_data.host, |booking_withdraws| {
				for (index, tuple) in booking_withdraws.iter().enumerate() {
					// Check if the first element of the tuple matches the target value
					if tuple.0 == *booking_id {
						// Perform the swap_remove operation
						booking_withdraws.swap_remove(index);
						break
					}
				}
			});

			<PlaceBookings<T>>::try_mutate(&booking_data.place_id, |booking_list| {
				if let Some(ind) = booking_list.iter().position(|&bid| bid == *booking_id) {
					booking_list.swap_remove(ind);
					return Ok(())
				}
				Err(())
			})
			.map_err(|_| <Error<T>>::BookingNotFound)?;

			booking_data.state = BookingState::Completed;
			<BookingsData<T>>::insert(booking_id, booking_data);

			return Ok(*booking_id)
		}

		Err(Error::<T>::BookingNotFound.into())
	}

	/// Perform the withdrawal as the GUEST of the place.
	///
	/// # Arguments
	///
	/// * `sender` - The caller of the function.
	/// * `booking_id` - The unique identifier of the booking to be withdrawed.
	///
	/// # Returns
	///
	/// Returns a `Result` indicating the success or failure of the booking withdrawal function. If
	/// the GUEST was able to unserve its locked funds, it simply returns the same
	/// booking_id, otherwise, it will throw an specific error.
	fn _guest_withdraw_booking(
		sender: <T>::AccountId,
		booking_id: &T::Hash,
	) -> Result<T::Hash, DispatchError> {
		if let Some(mut booking_data) = Self::get_booking_by_id(booking_id) {
			ensure!(sender == booking_data.guest, Error::<T>::NotPlaceGuest);

			// Simply unreserve the funds
			T::Currency::unreserve(&booking_data.guest, booking_data.amount);

			// Now persist new state
			<PendingBookingWithdraws<T>>::mutate(&booking_data.guest, |booking_withdraws| {
				for (index, tuple) in booking_withdraws.iter().enumerate() {
					// Check if the first element of the tuple matches the target value
					if tuple.0 == *booking_id {
						// Perform the swap_remove operation
						booking_withdraws.swap_remove(index);
						break
					}
				}
			});

			booking_data.state = BookingState::Completed;
			<BookingsData<T>>::insert(booking_id, booking_data);

			return Ok(*booking_id)
		}

		Err(Error::<T>::BookingNotFound.into())
	}
}
