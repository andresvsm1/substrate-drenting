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
	/// After creation, the booking status is set to "Pending", and requires host's supervision.
	///
	/// # Arguments
	///
	/// * `sender` - The account identifier of the booking sender.
	/// * `place_id` - The identifier of the place to book.
	/// * `start_date` - The start date of the booking.
	/// * `end_date` - The end date of the booking.
	///
	/// # Returns
	///
	/// Returns a `Result` indicating the success or failure of the booking creation operation. If the
	/// operation is successful, the `Result` contains the unique identifier (`Hash`) for the created booking.
	/// Otherwise, it contains an error indicating the reason for failure.
	///
	fn _create_booking(
		sender: T::AccountId,
		place_id: T::Hash,
		start_date: T::Moment,
		end_date: T::Moment,
		amount: BalanceOf<T>,
	) -> Result<T::Hash, DispatchError>;

	/// Update a booking's information.
	///
	/// This function updates an existing booking identified by `booking_id` with the provided `start_date`,
	/// `end_date`, and `sender` account identifier. It modifies the booking information and returns the
	/// updated booking identifier.
	/// After an update, the booking status changes to "Pending", as it is considered
	/// a new booking, which has to be analyzed by the host.
	///
	/// # Arguments
	///
	/// * `sender` - The account identifier of the booking sender.
	/// * `booking_id` - The identifier of the booking to update.
	/// * `place_id` - The identifier of the booked place.
	/// * `start_date` - The updated start date of the booking.
	/// * `end_date` - The updated end date of the booking.
	///
	/// # Returns
	///
	/// Returns a `Result` indicating the success or failure of the booking update operation. If the operation
	/// is successful, the `Result` contains the updated booking identifier. Otherwise, it contains an error
	/// indicating the reason for failure.
	///
	fn _update_booking(
		sender: T::AccountId,
		booking_id: &T::Hash,
		place_id: &T::Hash,
		start_date: T::Moment,
		end_date: T::Moment,
	) -> Result<T::Hash, DispatchError>;

	/// Cancel a booking.
	///
	/// This function cancels an existing booking identified by `booking_id`. It removes the booking from
	/// the system and returns the canceled booking identifier.
	/// After a cancelation, the booking status changes to "Canceled," and the place becomes available for other potential guests.
	///
	/// # Arguments
	///
	/// * `sender` - The account identifier of the sender requesting the cancellation.
	/// * `booking_id` - The identifier of the booking to cancel.
	///
	/// # Returns
	///
	/// Returns a `Result` indicating the success or failure of the booking cancellation operation. If the
	/// operation is successful, the `Result` contains the canceled booking identifier. Otherwise, it contains
	/// an error indicating the reason for failure.
	///
	fn _cancel_booking(
		sender: T::AccountId,
		booking_id: &T::Hash,
	) -> Result<T::Hash, DispatchError>;

	/// Confirm a Booking Request.
	///
	/// This function allows a host to confirm a booking request for a specific place.
	/// The `booking_id` parameter identifies the booking request to be confirmed.
	/// Upon successful confirmation, the booking status changes to "Confirmed," and the booked dates are reserved for the guest.
	///
	/// # Arguments
	///
	/// * `sender` - The account identifier of the sender requesting the cancellation.
	/// * `booking_id` - The identifier of the booking request to be confirmed.
	///
	/// # Returns
	///
	/// Returns a `Result` indicating the success or failure of the booking confirmation operation.
	/// If the operation is successful, the `Result` contains the unique identifier (`Hash`) for the confirmed booking.
	/// Otherwise, it contains a `DispatchError` explaining the reason for failure.
	///
	fn _confirm_booking(
		sender: T::AccountId,
		booking_id: &T::Hash,
	) -> Result<T::Hash, DispatchError>;

	/// Reject a Booking Request.
	///
	/// This function allows a host to reject a booking request for a specific place.
	/// The `booking_id` parameter identifies the booking request to be rejected.
	/// After rejection, the booking status changes to "Rejected," and the place becomes available for other potential guests.
	///
	/// # Arguments
	///
	/// * `sender` - The account identifier of the sender requesting the cancellation.
	/// * `booking_id` - The identifier of the booking request to be rejected.
	///
	/// # Returns
	///
	/// Returns a `Result` indicating the success or failure of the booking rejection operation.
	/// If the operation is successful, the `Result` contains the unique identifier (`Hash`) for the rejected booking.
	/// Otherwise, it contains a `DispatchError` explaining the reason for failure.
	///
	fn _reject_booking(
		sender: T::AccountId,
		booking_id: &T::Hash,
	) -> Result<T::Hash, DispatchError>;

	/// Perform Check-In for a Booking.
	///
	/// This function allows a guest to perform the check-in process for a confirmed booking.
	/// The `booking_id` parameter identifies the booking for which the check-in is being performed.
	/// After successful check-in, the guest gains access to the place for the specified booking period and the booking
	/// state is set to `HostCanWithdraw`.
	///
	/// # Arguments
	///
	/// * `sender` - The account identifier of the sender requesting the cancellation.
	/// * `booking_id` - The identifier of the booking for which the check-in is being performed.
	///
	/// # Returns
	///
	/// Returns a `Result` indicating the success or failure of the check-in operation.
	/// If the operation is successful, the `Result` contains the unique identifier (`Hash`) for the checked-in booking.
	/// Otherwise, it contains a `DispatchError` explaining the reason for failure.
	///
	fn _checkin(sender: T::AccountId, booking_id: &T::Hash) -> Result<T::Hash, DispatchError>;

	/// Withdraw a Booking.
	///
	/// This function allows a guest or host to withdraw a booking (canceled or completed), transferring/releasing the funds.
	/// The `booking_id` parameter identifies the booking to be withdrawn.
	/// Based on a RefundPolicy, the booking can be fully refunded, partially or not refunded at all.
	/// After executiong this function, the state of the booking can be `UserCanWithdraw`, `HostCanWithdraw` or `Finalized`
	///
	/// # Arguments
	///
	/// * `sender` - The account identifier of the sender requesting the cancellation.
	/// * `booking_id` - The identifier of the booking to be withdrawn.
	///
	/// # Returns
	///
	/// Returns a `Result` indicating the success or failure of the booking withdrawal operation.
	/// If the operation is successful, the `Result` contains the unique identifier (`Hash`) for the withdrawn booking.
	/// Otherwise, it contains a `DispatchError` explaining the reason for failure.
	///
	fn _withdraw_booking(
		sender: T::AccountId,
		booking_id: &T::Hash,
	) -> Result<T::Hash, DispatchError>;
}
