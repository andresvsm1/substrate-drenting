#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod functions;
pub mod interface;
pub mod structures;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use crate::interface::PlacesInterface;
	use crate::structures::PlaceData;

	use super::*;
	use frame_support::{pallet_prelude::*, sp_std::prelude::*};
	use frame_system::pallet_prelude::*;
	pub type Bytes = Vec<u8>;

	pub use structures::*;

	#[pallet::pallet]
	#[pallet::without_storage_info] // This allows us to use unsafe storages, at some point we might need bounded storages
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_timestamp::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn get_all_places)]
	pub type PlacesIds<T: Config> = StorageValue<_, Vec<T::Hash>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_place_by_id)]
	pub type PlacesData<T: Config> = StorageMap<_, Twox64Concat, T::Hash, PlaceData<T>>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new Place has been created
		PlaceCreated {
			id: T::Hash,
			sender: T::AccountId,
		},
		PlaceUpdated {
			id: T::Hash,
			sender: T::AccountId,
		},
		PlaceRemoved {
			id: T::Hash,
			sender: T::AccountId,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Generic Error
		UnhandledException,
		/// Place id already exists
		PlaceAlreadyExists,
		/// Place does not exists
		PlaceNotFound,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Extrinsic to create a new Place
		///
		/// * `place_type` - The PlaceType
		/// * `name` - Name for the Place, initially `Bytes`
		/// * `address` - Location of the Place
		/// * `description` - Hash reference of the Place description
		/// * `price_per_night` - Price per night of the Place
		/// * `images` - List of images from the place, hash references
		/// * `number_of_floors` - Number of floors, in case the Place has more than one
		#[pallet::call_index(1)]
		pub fn create_place(
			origin: OriginFor<T>,
			place_type: PlaceType,
			name: Bytes,
			address: Bytes,
			description: T::Hash,
			price_per_night: u64,
			checkin_hour: u32,
			checkout_hour: u32,
			images: Vec<T::Hash>,
			number_of_floors: Option<u8>,
		) -> DispatchResult {
			// Check sender
			let sender = ensure_signed(origin)?;

			let place_id = Self::_create_place(
				place_type,
				name,
				address,
				description,
				price_per_night,
				checkin_hour,
				checkout_hour,
				images,
				number_of_floors,
				&sender,
			)?;

			// Deposit our "Created" event.
			Self::deposit_event(Event::PlaceCreated { id: place_id, sender });
			Ok(())
		}
		/// Extrinsic to update a Place. At the time it allows most of the data to be modified,
		/// but at some point it'll be restricted. DEV
		///
		/// * `place_id` - The Place identifier
		/// * `place_type` - The PlaceType
		/// * `name` - Name for the Place, initially `Bytes`
		/// * `address` - Location of the Place
		/// * `description` - Hash reference of the Place description
		/// * `price_per_night` - Price per night of the Place
		/// * `images` - List of images from the place, hash references
		/// * `number_of_floors` - Number of floors, in case the Place has more than one
		#[pallet::call_index(2)]
		pub fn update_place(
			origin: OriginFor<T>,
			place_id: T::Hash,
			place_type: Option<PlaceType>,
			name: Option<Bytes>,
			address: Option<Bytes>,
			description: Option<T::Hash>,
			price_per_night: Option<u64>,
			checkin_hour: Option<u32>,
			checkout_hour: Option<u32>,
			images: Option<Vec<T::Hash>>,
			number_of_floors: Option<u8>,
		) -> DispatchResult {
			// Check sender
			let sender = ensure_signed(origin)?;

			let place_id = Self::_update_place(
				&place_id,
				place_type,
				name,
				address,
				description,
				price_per_night,
				checkin_hour,
				checkout_hour,
				images,
				number_of_floors,
				&sender,
			)?;

			// Deposit our "Updated" event.
			Self::deposit_event(Event::PlaceUpdated { id: place_id, sender });
			Ok(())
		}

		/// Extrinsic to remove a Place.
		///
		/// * `place_id` - The Place identifier
		#[pallet::call_index(3)]
		pub fn remove_place(origin: OriginFor<T>, place_id: T::Hash) -> DispatchResult {
			// Check sender
			let sender = ensure_signed(origin)?;

			let place_id = Self::_remove_place(&place_id)?;

			// Deposit our "Removed" event.
			Self::deposit_event(Event::PlaceRemoved { id: place_id, sender });
			Ok(())
		}
	}
}
