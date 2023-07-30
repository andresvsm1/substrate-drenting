#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod functions;
pub mod interface;
pub mod structures;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::*;
	use crate::interface::BookingsInterface;
	use frame_support::{
		pallet_prelude::{ValueQuery, *},
		sp_std::prelude::*,
		traits::{Currency, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	pub type Bytes = Vec<u8>;

	pub use structures::*;

	#[pallet::pallet]
	#[pallet::without_storage_info] // This allows us to use unsafe storages, at some point we might need bounded storages
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_places::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The Currency handler for the Bookings pallet.
		type Currency: ReservableCurrency<Self::AccountId>;
	}

	/// Stores all the bookings in the system
	#[pallet::storage]
	#[pallet::getter(fn get_all_bookings)]
	pub type BookingsIds<T: Config> = StorageValue<_, Vec<T::Hash>, ValueQuery>;

	/// Stores a mapping between a booking id and the actual booking
	#[pallet::storage]
	#[pallet::getter(fn get_booking_by_id)]
	pub type BookingsData<T: Config> = StorageMap<_, Twox64Concat, T::Hash, BookingData<T>>;

	/// Stores a mapping between a place id and all the bookings associated to it.
	/// It only tracks active bookings
	#[pallet::storage]
	#[pallet::getter(fn get_place_bookings)]
	pub type PlaceBookings<T: Config> =
		StorageMap<_, Twox64Concat, T::Hash, Vec<T::Hash>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_pending_booking_withdraws_by_account)]
	pub type PendingBookingWithdraws<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Vec<(T::Hash, BalanceOf<T>)>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new Booking has been created
		BookingPlaced { id: T::Hash, sender: T::AccountId },
		/// A Booking has been updated
		BookingUpdated { id: T::Hash, sender: T::AccountId, state: BookingState },
		/// A Booking has been canceled
		BookingCanceled { id: T::Hash, sender: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Generic Error
		UnhandledException,
		/// Booking id not found
		BookingNotFound,
		/// Booking id already exists
		BookingAlreadyExists,
		/// Booking dates are not available
		BookingDatesNotAvailable,
		/// Owner cannot book its own place
		CannotBookOwnedPlace,
		/// Account does not have enough free balance to book
		NotEnoughFreeBalance,
		/// end_date cannot be less than the start_date
		InvalidDates,
		/// start_date cannot be less or equal to current chain moment
		InvalidStartDate,
		/// Not Place Owner
		NotPlaceOwner,
		/// State is not correct
		WrongState,
		/// Cannot confirm booking. Booking is outdated
		CannotConfirmOutdatedBooking,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new booking for a specified place with the provided booking details.
		///
		/// This extrinsic allows any signed account (`origin`) to create a booking for a specific `place_id` with the given `start_date`,
		/// `end_date`, and `amount`. The booking request is processed, and if successful, a unique identifier (`Hash`) for the created booking
		/// is returned. The `amount` parameter represents the payment to be made for the booking.
		///
		/// # Arguments
		///
		/// * `origin` - The account identifier of the sender initiating the booking creation.
		/// * `place_id` - The unique identifier of the place to book.
		/// * `start_date` - The start date of the booking.
		/// * `end_date` - The end date of the booking.
		/// * `amount` - The payment amount for the booking.
		///
		/// # Returns
		///
		/// Returns a `DispatchResult` indicating the success or failure of the booking creation operation.
		/// If the operation is successful, a new booking is created, and the `DispatchResult` contains no error.
		/// If the booking creation fails, the `DispatchResult` contains an error describing the reason for failure.
		///
		#[pallet::call_index(1)]
		pub fn create_booking(
			origin: OriginFor<T>,
			place_id: T::Hash,
			start_date: T::Moment,
			end_date: T::Moment,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			// Check sender
			let sender = ensure_signed(origin)?;

			let booking_id =
				Self::_create_booking(sender.clone(), place_id, start_date, end_date, amount)?;

			// Deposit our "Placed" event.
			Self::deposit_event(Event::BookingPlaced { id: booking_id, sender });
			Ok(())
		}

		/// Update an existing booking with new booking details.
		///
		/// This extrinsic allows any signed account (`origin`) to update the booking details for a specific `place_id` with the provided `start_date`,
		/// `end_date`, and `amount`. The update is processed, and if successful, the booking details are modified accordingly.
		///
		/// # Arguments
		///
		/// * `origin` - The account identifier of the sender initiating the booking update.
		/// * `booking_id` - The identifier of the booking to update.
		/// * `place_id` - The unique identifier of the place associated with the booking to be updated.
		/// * `start_date` - The updated start date of the booking.
		/// * `end_date` - The updated end date of the booking.
		/// * `amount` - The updated payment amount for the booking.
		///
		/// # Returns
		///
		/// Returns a `DispatchResult` indicating the success or failure of the booking update operation.
		/// If the operation is successful, the booking details are updated, and the `DispatchResult` contains no error.
		/// If the booking update fails, the `DispatchResult` contains an error describing the reason for failure.
		///
		#[pallet::call_index(2)]
		pub fn update_booking(
			origin: OriginFor<T>,
			booking_id: T::Hash,
			place_id: T::Hash,
			start_date: T::Moment,
			end_date: T::Moment,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			// Check sender
			todo!();
		}

		/// Cancel an existing booking for a specified place.
		///
		/// This extrinsic allows any signed account (`origin`) to cancel a booking for a specific `booking_id`.
		/// The cancelation process at the time is pretty simple, there are no penalties, but this can evolve to a more complex solution
		/// where we implement a refund policy.
		///
		/// # Arguments
		///
		/// * `origin` - The account identifier of the sender initiating the booking cancellation.
		/// * `booking_id` - The identifier of the booking to cancel.
		///
		/// # Returns
		///
		/// Returns a `DispatchResult` indicating the success or failure of the booking cancellation operation.
		/// If the operation is successful, the booking is canceled, and the `DispatchResult` contains no error.
		/// If the booking cancellation fails, the `DispatchResult` contains an error describing the reason for failure.
		///
		#[pallet::call_index(3)]
		pub fn cancel_booking(origin: OriginFor<T>, booking_id: T::Hash) -> DispatchResult {
			// Check sender
			todo!();
		}

		/// Confirm a pending booking request for a specified place.
		///
		/// This extrinsic allows any signed account (`origin`) to confirm a booking request for a specific `booking_id`.
		/// Upon successful confirmation, the booking status changes to "Confirmed," and the booked dates are reserved for the guest.
		///
		/// # Arguments
		///
		/// * `origin` - The account identifier of the sender initiating the booking confirmation.
		/// * `booking_id` - The identifier of the booking to accept.
		///
		/// # Returns
		///
		/// Returns a `DispatchResult` indicating the success or failure of the booking confirmation operation.
		/// If the operation is successful, the booking is confirmed, and the `DispatchResult` contains no error.
		/// If the booking confirmation fails, the `DispatchResult` contains an error describing the reason for failure.
		///
		#[pallet::call_index(4)]
		pub fn confirm_booking(origin: OriginFor<T>, booking_id: T::Hash) -> DispatchResult {
			// Check sender
			let sender = ensure_signed(origin)?;

			Self::_confirm_booking(sender.clone(), &booking_id)?;

			// Deposit our "Confirmed" event.
			Self::deposit_event(Event::BookingUpdated {
				id: booking_id,
				sender,
				state: BookingState::Confirmed,
			});
			Ok(())
		}

		/// Reject a pending booking request for a specified place.
		///
		/// This extrinsic allows any signed account (`origin`) to reject a booking request for a specific `booking_id`.
		/// After rejection, the booking status changes to "Rejected," and the place becomes available for other potential guests.
		///
		/// # Arguments
		///
		/// * `origin` - The account identifier of the sender initiating the booking rejection.
		/// * `booking_id` - The identifier of the booking to update.
		///
		/// # Returns
		///
		/// Returns a `DispatchResult` indicating the success or failure of the booking rejection operation.
		/// If the operation is successful, the booking is rejected, and the `DispatchResult` contains no error.
		/// If the booking rejection fails, the `DispatchResult` contains an error describing the reason for failure.
		///
		#[pallet::call_index(5)]
		pub fn reject_booking(origin: OriginFor<T>, booking_id: T::Hash) -> DispatchResult {
			// Check sender
			let sender = ensure_signed(origin)?;

			Self::_reject_booking(sender.clone(), &booking_id)?;

			// Deposit our "Rejected" event.
			Self::deposit_event(Event::BookingUpdated {
				id: booking_id,
				sender,
				state: BookingState::Rejected,
			});
			Ok(())
		}

		/// Perform Check-In for a Confirmed Booking.
		///
		/// This extrinsic allows any signed account (`origin`) to perform the check-in process for a confirmed booking.
		/// The check-in process ensures that the guest gains access to the place for the specified booking period.
		///
		/// # Arguments
		///
		/// * `origin` - The account identifier of the sender initiating the check-in process.
		/// * `booking_id` - The identifier of the booking to update.
		///
		/// # Returns
		///
		/// Returns a `DispatchResult` indicating the success or failure of the check-in process.
		/// If the check-in is successful, the guest gains access to the place, and the `DispatchResult` contains no error.
		/// If the check-in process fails, the `DispatchResult` contains an error describing the reason for failure.
		///
		#[pallet::call_index(6)]
		pub fn checkin(origin: OriginFor<T>, booking_id: T::Hash) -> DispatchResult {
			// Check sender
			todo!();
		}

		/// Withdraw a Booking, Canceling the Reservation and Releasing the Funds.
		///
		/// This extrinsic allows any signed account (`origin`) to withdraw a booking for a specific `booking_id`.
		/// At the time, this extrinsic might only be used by the host of the place or the guest in case of a cancelation
		/// but when a refund policy is implemented, it might also be executed by both users controlling the amount
		/// each part receives from the booking.
		///
		/// # Arguments
		///
		/// * `origin` - The account identifier of the sender initiating the booking withdrawal.
		/// * `booking_id` - The identifier of the booking to update.
		///
		/// # Returns
		///
		/// Returns a `DispatchResult` indicating the success or failure of the booking withdrawal operation.
		/// If the operation is successful, the booking is withdrawn, and the `DispatchResult` contains no error.
		/// If the booking withdrawal fails, the `DispatchResult` contains an error describing the reason for failure.
		///
		#[pallet::call_index(7)]
		pub fn withdraw_booking(origin: OriginFor<T>, booking_id: T::Hash) -> DispatchResult {
			// Check sender
			todo!();
		}
	}
}
