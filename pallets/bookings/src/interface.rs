use crate::{BalanceOf, Config};
use frame_support::sp_runtime::DispatchError;
/// Interface for Bookings pallet
pub trait BookingsInterface<T: Config> {
	type Error;

	/// Store a booking with its information.
	///
	/// This function creates a new booking for the specified `place_id` with the provided `start_date`,
	/// `end_date`, and `sender` account identifier. It stores the booking information and returns a
	/// unique identifier (`Hash`) for the created booking.
	///
	/// # Arguments
	///
	/// * `place_id` - The identifier of the place to book.
	/// * `start_date` - The start date of the booking.
	/// * `end_date` - The end date of the booking.
	/// * `sender` - The account identifier of the booking sender.
	///
	/// # Returns
	///
	/// Returns a `Result` indicating the success or failure of the booking creation operation. If the
	/// operation is successful, the `Result` contains the unique identifier (`Hash`) for the created booking.
	/// Otherwise, it contains an error indicating the reason for failure.
	///
	fn _create_booking(
		place_id: T::Hash,
		start_date: T::Moment,
		end_date: T::Moment,
		sender: T::AccountId,
		amount: BalanceOf<T>,
	) -> Result<T::Hash, DispatchError>;

	/// Update a booking's information.
	///
	/// This function updates an existing booking identified by `booking_id` with the provided `start_date`,
	/// `end_date`, and `sender` account identifier. It modifies the booking information and returns the
	/// updated booking identifier.
	///
	/// # Arguments
	///
	/// * `booking_id` - The identifier of the booking to update.
	/// * `start_date` - The updated start date of the booking.
	/// * `end_date` - The updated end date of the booking.
	/// * `sender` - The account identifier of the sender requesting the update.
	///
	/// # Returns
	///
	/// Returns a `Result` indicating the success or failure of the booking update operation. If the operation
	/// is successful, the `Result` contains the updated booking identifier. Otherwise, it contains an error
	/// indicating the reason for failure.
	///
	fn _update_booking(
		booking_id: &T::Hash,
		start_date: T::Moment,
		end_date: T::Moment,
		sender: T::AccountId,
	) -> Result<T::Hash, DispatchError>;

	/// Cancel a booking.
	///
	/// This function cancels an existing booking identified by `booking_id`. It removes the booking from
	/// the system and returns the canceled booking identifier.
	///
	/// # Arguments
	///
	/// * `booking_id` - The identifier of the booking to cancel.
	/// * `sender` - The account identifier of the sender requesting the cancellation.
	///
	/// # Returns
	///
	/// Returns a `Result` indicating the success or failure of the booking cancellation operation. If the
	/// operation is successful, the `Result` contains the canceled booking identifier. Otherwise, it contains
	/// an error indicating the reason for failure.
	///
	fn _cancel_booking(booking_id: &T::Hash, sender: T::AccountId) -> Result<T::Hash, DispatchError>;
}
