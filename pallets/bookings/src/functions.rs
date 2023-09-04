#![cfg_attr(not(feature = "std"), no_std)]
use crate::{
	interface::BookingsInterface,
	structures::{BookingData, BookingHashingData},
	BalanceOf, BookingState, BookingsData, BookingsIds, Config, Error, Pallet,
	PendingBookingWithdraws, PlaceBookings,
};
use chrono::{DateTime, Duration, NaiveDateTime, Timelike, Utc};
use frame_support::{
	ensure,
	sp_runtime::{traits::Hash, ArithmeticError, DispatchError, SaturatedConversion},
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

			let expected_amount = Self::_calculate_total_amount(
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
				BookingState::Created,
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

			return Ok(booking_id);
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
			return Ok(*booking_id);
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
			return Ok(*booking_id);
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

			return Ok(*booking_id);
		}
		Err(Error::<T>::BookingNotFound.into())
	}

	fn _withdraw_booking(
		sender: <T>::AccountId,
		booking_id: &<T>::Hash,
	) -> Result<<T>::Hash, DispatchError> {
		if let Some(booking) = Self::get_booking_by_id(booking_id) {
			return match booking.state {
				BookingState::Confirmed => todo!(), // Check dates to see if its unused
				BookingState::Rejected => todo!(),  // Only guest should be able to claim the funds
				BookingState::Withdrawable => todo!(), // Check Refund Policy
				BookingState::UserCanWithdraw => todo!(), // Unreserve funds for guest
				BookingState::OwnerCanWithdraw => Self::host_withdraw_booking(sender, booking_id), // Transfer reserved funds from guest to host
				_ => return Err(Error::<T>::WrongState.into()),
			};
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
	/// Returns `true` if the place is available for booking, meaning it does not overlap with any existing bookings.
	/// Returns `false` if the place is not available for booking, indicating an overlap with an existing booking.
	///
	fn check_availability(place_id: T::Hash, start_date: T::Moment, end_date: T::Moment) -> bool {
		let place_bookings = Self::get_place_bookings(place_id);
		for booking_id in place_bookings {
			if let Some(booking) = Self::get_booking_by_id(booking_id) {
				if booking.state == BookingState::Created {
					continue;
				}
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

	/// Get overlapping bookings for a specified place and booking period.
	///
	/// This function retrieves a list of booking identifiers (`Hash`) that overlap with the provided booking period.
	/// It iterates through the existing bookings associated with the given `place_id` and compares their start and end dates
	/// with the provided `booking_start_date` and `booking_end_date`. If there is an overlap in the booking periods, the booking
	/// identifier is added to the list of `bookings_to_cancel`. The `booking_id_to_confirm` is excluded from the overlapping check,
	/// allowing the booking confirmation without conflict with itself.
	///
	/// # Arguments
	///
	/// * `place_id` - The unique identifier of the place for which overlapping bookings are to be found.
	/// * `booking_id_to_confirm` - The unique identifier of the booking to be confirmed, excluded from the overlapping check.
	/// * `booking_start_date` - The start date of the booking to be confirmed.
	/// * `booking_end_date` - The end date of the booking to be confirmed.
	///
	/// # Returns
	///
	/// Returns a vector (`Vec`) of booking identifiers (`Hash`) representing the list of overlapping bookings.
	/// If no overlapping bookings are found, the vector will be empty.
	///
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
				continue;
			}
			if let Some(booking) = Self::get_booking_by_id(booking_id) {
				match (
					booking_start_date.cmp(&booking.start_date),
					booking_end_date.cmp(&booking.end_date),
				) {
					(Ordering::Less, Ordering::Greater)
					| (Ordering::Equal, Ordering::Equal)
					| (Ordering::Greater, Ordering::Less) => {
						bookings_to_cancel.push(booking_id);
					},
					_ => {},
				}
			}
		}

		bookings_to_cancel
	}

	fn _calculate_total_amount(
		start_date: T::Moment,
		end_date: T::Moment,
		price_per_night: u64,
	) -> Result<u64, DispatchError> {
		let days_in_between = end_date - start_date;
		let timestamp_u64 = Self::convert_moment_to_u64_in_milliseconds(days_in_between)?;
		let timestamp_i64: i64 = i64::try_from(timestamp_u64).unwrap();
		let days = u64::try_from(Duration::milliseconds(timestamp_i64).num_days()).unwrap();
		// Add one extra night as it is rounded down
		return Ok((days + 1) * price_per_night)
	}

	/// Perform the cancellation of a booking.
	///
	/// This function cancels a specific booking identified by `booking_id`. It updates the booking's status and releases any held funds.
	/// The cancellation is initiated by the `host` or the `guest`, and the `amount` is returned accordingly.
	///
	/// # Arguments
	///
	/// * `place_id` - The unique identifier of the place associated with the booking to be canceled.
	/// * `booking_id` - The unique identifier of the booking to be canceled.
	/// * `host` - The account identifier of the host initiating the booking cancellation.
	/// * `guest` - The account identifier of the guest initiating the booking cancellation.
	/// * `amount` - The payment amount associated with the booking to be canceled.
	///
	/// # Returns
	///
	/// Returns a `Result` indicating the success or failure of the booking cancellation operation.
	/// If the operation is successful, the booking is canceled, and the `Result` contains no error.
	/// If the booking cancellation fails, the `Result` contains a `DispatchError` explaining the reason for failure.
	///
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
				return Ok(());
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
					break;
				}
			}
		});

		<PendingBookingWithdraws<T>>::mutate(&guest, |booking_withdraws| {
			booking_withdraws.push((booking_id, amount))
		});

		Ok(())
	}

	/// This function takes a u64 value and calculates its size
	///
	/// # Arguments
	///
	/// * `n` - The u64 number to process
	///
	/// # Returns
	///
	/// Returns a `usize` with the length of the number.
	///
	fn u64_len(mut n: u64) -> usize {
		if n == 0 {
			return 1; // A single-digit number has length 1.
		}

		let mut len = 0;
		while n > 0 {
			len += 1;
			n /= 10;
		}

		len
	}

	/// Convert a Moment to a 64-bit unsigned integer representing milliseconds.
	///
	/// This function takes a Moment 64-bit unsigned integer representing milliseconds since the Unix epoch
	/// If the conversion is successful, the function returns the resulting integer. Otherwise, it returns
	/// an error indicating the failure to convert the Moment.
	///
	/// # Argument
	///
	/// * `date` - The Moment to convert to a 64-bit unsigned integer.
	///
	/// # Returns
	///
	/// Returns a `Result` containing the converted 64-bit unsigned integer on success.
	/// If the conversion fails, the `Result` contains a `DispatchError` explaining the reason for the failure.
	///
	fn convert_moment_to_u64_in_milliseconds(date: T::Moment) -> Result<u64, DispatchError> {
		let date_as_u64_millis;
		if let Some(_date_as_u64) = TryInto::<u64>::try_into(date).ok() {
			date_as_u64_millis = _date_as_u64;
		} else {
			return Err(DispatchError::Other("Unable to convert Moment to u64 for date"));
		}
		return Ok(date_as_u64_millis);
	}

	/// Convert a 64-bit unsigned integer timestamp to a Moment.
	///
	/// This function takes a 64-bit unsigned integer `timestamp` representing milliseconds since the Unix epoch
	/// and converts it into a `Moment` type, which represents a point in time in the Substrate runtime.
	/// The function uses the `TryInto` trait to attempt the conversion, returning the resulting `Moment` on success.
	/// If the conversion fails, the function returns a `DispatchError` explaining the reason for the failure.
	///
	/// # Arguments
	///
	/// * `timestamp` - The 64-bit unsigned integer timestamp to convert to a Moment.
	///
	/// # Returns
	///
	/// Returns a `Result` containing the converted `Moment` on success.
	/// If the conversion fails, the `Result` contains a `DispatchError` explaining the reason for the failure.
	///
	fn convert_u64_to_moment(timestamp: u64) -> Result<T::Moment, DispatchError> {
		if let Some(moment) = TryInto::<T::Moment>::try_into(timestamp).ok() {
			return Ok(moment);
		} else {
			return Err(DispatchError::Other("Unable to convert u64 to Moment"));
		}
	}

	/// Convert a Moment to a UTC DateTime.
	///
	/// This function takes a `Moment` and converts it into a UTC `DateTime` with 13-digit precision.
	/// If the provided `Moment` is not represented with 13-digit precision (i.e., milliseconds),
	/// the function returns a `DispatchError` indicating the failure.
	///
	/// # Arguments
	///
	/// * `timestamp` - The `Moment` to convert to a UTC `DateTime` with 13-digit precision.
	///
	/// # Returns
	///
	/// Returns a `Result` containing the converted UTC `DateTime` on success.
	/// If the provided `Moment` does not have 13-digit precision or any other error occurs during the conversion,
	/// the `Result` contains a `DispatchError` explaining the reason for the failure.
	///
	fn convert_moment_to_datetime(timestamp: T::Moment) -> Result<DateTime<Utc>, DispatchError> {
		let mut timestamp_parsed: u64 = Self::convert_moment_to_u64_in_milliseconds(timestamp)?;
		match Self::u64_len(timestamp_parsed) {
			10 => {
				// The timestamp is in seconds. Multiply it by 1000 to convert it to milliseconds.
				timestamp_parsed = timestamp_parsed
					.checked_mul(1000)
					.ok_or(DispatchError::Arithmetic(ArithmeticError::Overflow))?;
			},
			13 => {}, // The timestamp is in milliseconds. No need to do anything.
			_ => return Err(DispatchError::Other("Timestamp precision is not 10 nor 13")),
		}
		let timestamp_i64: i64 = i64::try_from(timestamp_parsed).unwrap();
		let datetime = DateTime::<Utc>::from_utc(
			NaiveDateTime::from_timestamp_millis(timestamp_i64).unwrap(),
			Utc,
		);
		if datetime.timestamp_millis() == timestamp_i64 {
			// This does not fix anything it seems, we might need to check if the year is valid or else
			return Ok(datetime);
		}
		Err(DispatchError::Other("Provided timestamp is not a 13-digit precision timestamp"))
	}

	/// Modify the timestamp by setting the desired time of day.
	///
	/// This function takes a Moment (`timestamp`) and a desired hour of the day (`desired_time` in 24-hour format).
	/// It modifies the timestamp to have the desired time, setting minutes, seconds, and nanoseconds to zero.
	/// The function returns a Moment with the new time set.
	///
	/// # Arguments
	///
	/// * `timestamp` - The original Moment representing the starting timestamp.
	/// * `desired_time` - The desired hour of the day (in 24-hour format) to set in the modified timestamp.
	///
	/// # Returns
	///
	/// Returns a `Result` containing the modified Moment on success.
	/// If the `desired_time` is out of range or any other error occurs during the conversion, the `Result`
	/// contains a `DispatchError` explaining the reason for the failure.
	///
	pub fn modify_timestamp(
		timestamp: T::Moment,
		desired_time: u32,
	) -> Result<T::Moment, DispatchError> {
		match (desired_time.cmp(&0), desired_time.cmp(&23)) {
			(Ordering::Greater, Ordering::Less)
			| (Ordering::Equal, Ordering::Less)
			| (Ordering::Greater, Ordering::Equal) => {},
			_ => return Err(DispatchError::Other("desired time out of range")),
		}
		let datetime = Self::convert_moment_to_datetime(timestamp)?;
		let formatted_timestamp = datetime
			.with_hour(desired_time)
			.and_then(|dt| dt.with_minute(0))
			.and_then(|dt| dt.with_second(0))
			.and_then(|dt| dt.with_nanosecond(0))
			.map(|dt| dt.timestamp_millis())
			.unwrap();

		Ok(Self::convert_u64_to_moment(formatted_timestamp.try_into().unwrap())?)
	}

	fn host_withdraw_booking(
		sender: <T>::AccountId,
		booking_id: &T::Hash,
	) -> Result<T::Hash, DispatchError> {
		if let Some(mut booking_data) = Self::get_booking_by_id(booking_id) {
			ensure!(sender == booking_data.host, Error::<T>::NotPlaceOwner);

			// Try to withdraw first
			T::Currency::unreserve(&booking_data.guest, booking_data.amount);
			T::Currency::transfer(
				&booking_data.host,
				&booking_data.guest,
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
						break;
					}
				}
			});

			<PlaceBookings<T>>::try_mutate(&booking_data.place_id, |booking_list| {
				if let Some(ind) = booking_list.iter().position(|&bid| bid == *booking_id) {
					booking_list.swap_remove(ind);
					return Ok(());
				}
				Err(())
			})
			.map_err(|_| <Error<T>>::BookingNotFound)?;

			booking_data.state = BookingState::Completed;
			<BookingsData<T>>::insert(booking_id, booking_data);

			return Ok((*booking_id));
		}

		Err(Error::<T>::BookingNotFound.into())
	}
}
