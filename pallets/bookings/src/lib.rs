#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod functions;
pub mod interface;
pub mod structures;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use crate::interface::BookingsInterface;

	use super::*;
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

	#[pallet::storage]
	#[pallet::getter(fn get_pending_deposit_withdraws_by_account)]
	pub type PendingDepositWithdraws<T: Config> =
		StorageMap<_, Twox64Concat, T::Hash, Vec<(T::Hash, BalanceOf<T>)>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new Booking has been created
		BookingPlaced { id: T::Hash, sender: T::AccountId },
		/// A Booking has been updated
		BookingUpdated { id: T::Hash, sender: T::AccountId },
		/// A Booking has been canceled
		BookingCanceled { id: T::Hash, sender: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Generic Error
		UnhandledException,
		/// Booking id already exists
		BookingAlreadyExists,
		/// Booking dates are not available
		BookingDatesNotAvailable,
		/// Owner cannot book its own place
		CannotBookOwnedPlace,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Extrinsic to create a new booking
		///
		/// * `place_id` - The ID of the place to book
		/// * `start_date` - The start date of the booking
		/// * `end_date` - The end date of the booking
		///
		/// # Errors
		///
		/// Fails if the caller is the owner of the place, or if the booking overlaps with an existing booking.
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
				Self::_create_booking(place_id, start_date, end_date, sender.clone(), amount)?;

			// Deposit our "Placed" event.
			Self::deposit_event(Event::BookingPlaced { id: booking_id, sender });
			Ok(())
		}
	}
}
