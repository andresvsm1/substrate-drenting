#![cfg_attr(not(feature = "std"), no_std)]
use crate::{Config, Pallet};
use chrono::{DateTime, Duration, NaiveDateTime, Timelike, Utc};
use frame_support::{
	sp_runtime::{traits::CheckedSub, ArithmeticError, DispatchError},
	sp_std::cmp::Ordering,
};

/// Utils functions implementation
impl<T: Config> Pallet<T> {
	/// This function takes a u64 value and calculates its size
	///
	/// # Arguments
	///
	/// * `n` - The u64 number to process
	///
	/// # Returns
	///
	/// Returns a `usize` with the length of the number.
	fn u64_len(mut n: u64) -> usize {
		if n == 0 {
			return 1 // A single-digit number has length 1.
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
	/// This function takes a Moment 64-bit unsigned integer representing milliseconds since the
	/// Unix epoch If the conversion is successful, the function returns the resulting integer.
	/// Otherwise, it returns an error indicating the failure to convert the Moment.
	///
	/// # Argument
	///
	/// * `date` - The Moment to convert to a 64-bit unsigned integer.
	///
	/// # Returns
	///
	/// Returns a `Result` containing the converted 64-bit unsigned integer on success.
	/// If the conversion fails, the `Result` contains a `DispatchError` explaining the reason for
	/// the failure.
	pub fn convert_moment_to_u64_in_milliseconds(date: T::Moment) -> Result<u64, DispatchError> {
		let date_as_u64_millis;
		if let Some(_date_as_u64) = TryInto::<u64>::try_into(date).ok() {
			date_as_u64_millis = _date_as_u64;
		} else {
			return Err(DispatchError::Other("Unable to convert Moment to u64 for date"))
		}
		return Ok(date_as_u64_millis)
	}

	/// Convert a 64-bit unsigned integer timestamp to a Moment.
	///
	/// This function takes a 64-bit unsigned integer `timestamp` representing milliseconds since
	/// the Unix epoch and converts it into a `Moment` type, which represents a point in time in the
	/// Substrate runtime. The function uses the `TryInto` trait to attempt the conversion,
	/// returning the resulting `Moment` on success. If the conversion fails, the function returns a
	/// `DispatchError` explaining the reason for the failure.
	///
	/// # Arguments
	///
	/// * `timestamp` - The 64-bit unsigned integer timestamp to convert to a Moment.
	///
	/// # Returns
	///
	/// Returns a `Result` containing the converted `Moment` on success.
	/// If the conversion fails, the `Result` contains a `DispatchError` explaining the reason for
	/// the failure.
	fn convert_u64_to_moment(timestamp: u64) -> Result<T::Moment, DispatchError> {
		if let Some(moment) = TryInto::<T::Moment>::try_into(timestamp).ok() {
			return Ok(moment)
		} else {
			return Err(DispatchError::Other("Unable to convert u64 to Moment"))
		}
	}

	/// Convert a Moment to a UTC DateTime.
	///
	/// This function takes a `Moment` and converts it into a UTC `DateTime` with 13-digit
	/// precision. If the provided `Moment` is not represented with 13-digit precision (i.e.,
	/// milliseconds), the function returns a `DispatchError` indicating the failure.
	///
	/// # Arguments
	///
	/// * `timestamp` - The `Moment` to convert to a UTC `DateTime` with 13-digit precision.
	///
	/// # Returns
	///
	/// Returns a `Result` containing the converted UTC `DateTime` on success.
	/// If the provided `Moment` does not have 13-digit precision or any other error occurs during
	/// the conversion, the `Result` contains a `DispatchError` explaining the reason for the
	/// failure.
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
			// This does not fix anything it seems, we might need to check if the year is valid or
			// else
			return Ok(datetime)
		}
		Err(DispatchError::Other("Provided timestamp is not a 13-digit precision timestamp"))
	}

	/// Modify the timestamp by setting the desired time of day.
	///
	/// This function takes a Moment (`timestamp`) and a desired hour of the day (`desired_time` in
	/// 24-hour format). It modifies the timestamp to have the desired time, setting minutes,
	/// seconds, and nanoseconds to zero. The function returns a Moment with the new time set.
	///
	/// # Arguments
	///
	/// * `timestamp` - The original Moment representing the starting timestamp.
	/// * `desired_time` - The desired hour of the day (in 24-hour format) to set in the modified
	///   timestamp.
	///
	/// # Returns
	///
	/// Returns a `Result` containing the modified Moment on success.
	/// If the `desired_time` is out of range or any other error occurs during the conversion, the
	/// `Result` contains a `DispatchError` explaining the reason for the failure.
	pub fn modify_timestamp(
		timestamp: T::Moment,
		desired_time: u32,
	) -> Result<T::Moment, DispatchError> {
		match (desired_time.cmp(&0), desired_time.cmp(&23)) {
			(Ordering::Greater, Ordering::Less) |
			(Ordering::Equal, Ordering::Less) |
			(Ordering::Greater, Ordering::Equal) => {},
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

	/// Calculate the total amount of a booking
	///
	/// This function takes two dates (T::Moment) and the price per night of a certain place.
	/// Then, it calculates the days between both provided moments and multiplies that by the price
	/// per night. All the arithmetic operations are performed with overflow control.
	///
	/// # Arguments
	///
	/// * `start_date` - The start date of the booking.
	/// * `end_date` - The end date of the booking.
	/// * `price_per_night` - The price per night of the place.
	///
	/// # Returns
	///
	/// Returns a `Result` containing the total price of the booking if success. If some operation
	/// fails, it returns the specific error.
	pub fn calculate_total_amount(
		start_date: T::Moment,
		end_date: T::Moment,
		price_per_night: u64,
	) -> Result<u64, DispatchError> {
		let days_in_between = end_date
			.checked_sub(&start_date)
			.ok_or(DispatchError::Arithmetic(ArithmeticError::Underflow))?;

		let timestamp_u64 = Self::convert_moment_to_u64_in_milliseconds(days_in_between)?;
		let timestamp_i64: i64 = i64::try_from(timestamp_u64).unwrap();
		let days = u64::try_from(Duration::milliseconds(timestamp_i64).num_days()).unwrap();
		// Add one extra night as it is rounded down
		let total_days = days
			.checked_add(1)
			.ok_or(DispatchError::Arithmetic(ArithmeticError::Overflow))?;

		return total_days
			.checked_mul(price_per_night)
			.ok_or(DispatchError::Arithmetic(ArithmeticError::Overflow))
	}
}
