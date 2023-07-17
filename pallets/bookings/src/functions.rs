#![cfg_attr(not(feature = "std"), no_std)]
use crate::{
	interface::BookingsInterface,
	structures::{BookingData, BookingHashingData},
	BalanceOf, BookingsData, BookingsIds, Config, Error, Pallet, PendingBookingWithdraws,
	PlaceBookings,
};
use chrono::{DateTime, NaiveDateTime, Timelike, Utc};
use frame_support::{
	ensure,
	sp_runtime::{traits::Hash, ArithmeticError, DispatchError},
	sp_std::cmp::Ordering,
	traits::ReservableCurrency,
};
use pallet_places::Error as PlacesError;

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
			ensure!(T::Currency::can_reserve(&sender, amount), Error::<T>::NotEnoughFreeBalance);
			ensure!(&place.owner != &sender, Error::<T>::CannotBookOwnedPlace);

			let formatted_start_date = Self::modify_timestamp(start_date, place.checkin_hour)?;
			let formatted_end_date = Self::modify_timestamp(end_date, place.checkout_hour)?;

			if !Self::check_availability(place_id, formatted_start_date, formatted_end_date) {
				return Err(Error::<T>::BookingDatesNotAvailable.into());
			}

			let booking_data: BookingData<T> = BookingData::new(
				place.owner,
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
			<PendingBookingWithdraws<T>>::mutate(&sender, |booking_withdraws| {
				booking_withdraws.push((booking_id, amount));
			});

			// Logging to the console on debug level
			log::debug!(target: "did", "A new Booking with ID ➡ {:?} has been placed.", booking_id);

			return Ok(booking_id);
		}
		Err(PlacesError::<T>::PlaceNotFound.into())
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
	fn modify_timestamp(
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
}
