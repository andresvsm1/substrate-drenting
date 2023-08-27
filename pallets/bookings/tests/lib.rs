#[cfg(test)]
pub mod mock;
pub mod utils;
use frame_support::{assert_noop, assert_ok};
use pallet_bookings::BookingsData;
use pallet_bookings::{BookingData, BookingState, Error};
use pallet_places::{Error as PlaceError, PlaceType};
use sp_core::H256;

use crate::mock::*;
use crate::utils::*;

// Default owner
const OWNER: u64 = 0;
const GUEST_A: u64 = 1;
const GUEST_B: u64 = 2;

fn create_default_place() {
	let _ = Places::create_place(
		RuntimeOrigin::signed(OWNER),
		PlaceType::Apartment,
		b"Demo Place".to_vec(),
		b"Demo Address".to_vec(),
		create_hash("Demo Description"),
		10,
		17,
		12,
		vec![create_hash("image_1"), create_hash("image_2")],
		None,
	);
}

fn create_default_booking() {
	let year = 2025;
	let month = 4;
	let start_day = 10;
	let end_day = 13;
	let start_date: u64 = generate_timestamp(year, month, start_day, 17, 33, 44);
	let end_date: u64 = generate_timestamp(year, month, end_day, 17, 33, 44);

	let amount: u64 = 10;

	let place_id: H256 = Places::get_all_places()[0];

	let _ = Bookings::create_booking(
		RuntimeOrigin::signed(GUEST_A),
		place_id,
		start_date,
		end_date,
		amount,
	);
}

fn confirm_default_booking() {
	let booking_id: H256 = Bookings::get_all_bookings()[0];
	let _ = Bookings::confirm_booking(RuntimeOrigin::signed(OWNER), booking_id);
}

fn build_with_defult_place() -> sp_io::TestExternalities {
	let mut ext = build_with_funded_accounts();
	ext.execute_with(create_default_place);
	ext
}

fn build_with_defult_place_and_booking() -> sp_io::TestExternalities {
	let mut ext = build_with_defult_place();
	ext.execute_with(create_default_booking);
	ext
}

fn build_with_default_confirmed_booking() -> sp_io::TestExternalities {
	let mut ext = build_with_defult_place_and_booking();
	ext.execute_with(confirm_default_booking);
	ext
}

// ========================================================
// Functions Unit Tests
// ========================================================
#[test]
fn test_modify_timestamp_function() {
	// Should work with seconds
	build_with_funded_accounts().execute_with(|| {
		let year = 2025;
		let month = 4;
		let start_day = 10;
		let end_day = 13;

		let checkin_hour = 17;
		let checkout_hour = 12;

		let start_date: u64 = generate_timestamp(year, month, start_day, 17, 33, 44);
		let end_date: u64 = generate_timestamp(year, month, end_day, 17, 33, 44);

		// Expected times
		let expected_start_date: u64 =
			generate_timestamp_millis(year, month, start_day, checkin_hour, 0, 0);
		let expected_end_date: u64 =
			generate_timestamp_millis(year, month, end_day, checkout_hour, 0, 0);

		assert_eq!(
			Bookings::modify_timestamp(start_date, checkin_hour).unwrap(),
			expected_start_date
		);
		assert_eq!(Bookings::modify_timestamp(end_date, checkout_hour).unwrap(), expected_end_date);
	});

	// Should work with milliseconds
	build_with_funded_accounts().execute_with(|| {
		let year = 2025;
		let month = 4;
		let start_day = 10;
		let end_day = 13;

		let checkin_hour = 17;
		let checkout_hour = 12;

		let start_date: u64 = generate_timestamp_millis(year, month, start_day, 17, 33, 44);
		let end_date: u64 = generate_timestamp_millis(year, month, end_day, 17, 33, 44);

		// Expected times
		let expected_start_date: u64 =
			generate_timestamp_millis(year, month, start_day, checkin_hour, 0, 0);
		let expected_end_date: u64 =
			generate_timestamp_millis(year, month, end_day, checkout_hour, 0, 0);

		assert_eq!(
			Bookings::modify_timestamp(start_date, checkin_hour).unwrap(),
			expected_start_date
		);
		assert_eq!(Bookings::modify_timestamp(end_date, checkout_hour).unwrap(), expected_end_date);
	});

	build_with_funded_accounts().execute_with(|| {
		let year = 2025;
		let month = 4;
		let start_day = 10;

		let checkin_hour = 25;

		let start_date: u64 = generate_timestamp_millis(year, month, start_day, 17, 33, 44);

		assert_noop!(
			Bookings::modify_timestamp(start_date, checkin_hour),
			sp_runtime::DispatchError::Other("desired time out of range")
		);
	});
}

// ========================================================
// Create Bookings Unit Tests
// ========================================================
#[test]
fn test_create_booking_should_work() {
	build_with_defult_place().execute_with(|| {
		let place_id: H256 = Places::get_all_places()[0];
		let place_data = Places::get_place_by_id(place_id).unwrap();

		let year = 2025;
		let month = 4;
		let start_day = 10;
		let end_day = 13;

		let start_date: u64 = generate_timestamp_millis(year, month, start_day, 17, 33, 44);
		let end_date: u64 = generate_timestamp_millis(year, month, end_day, 17, 33, 44);

		let amount = 10;

		assert_ok!(Bookings::create_booking(
			RuntimeOrigin::signed(GUEST_A),
			place_id,
			start_date,
			end_date,
			amount
		));

		let booking_id = Bookings::get_all_bookings()[0];
		let booking_data = Bookings::get_booking_by_id(booking_id);

		let formatted_start_date =
			Bookings::modify_timestamp(start_date, place_data.checkin_hour).unwrap();
		let formatted_end_date =
			Bookings::modify_timestamp(end_date, place_data.checkout_hour).unwrap();

		// Check the place has been created correctly
		assert_eq!(
			booking_data,
			Some(BookingData {
				place_id,
				host: 0,
				guest: 1,
				start_date: formatted_start_date,
				end_date: formatted_end_date,
				amount,
				state: BookingState::Created
			})
		);
	})
}

#[test]
fn test_create_booking_with_invalid_place_should_fail() {
	build_with_defult_place().execute_with(|| {
		let year = 2025;
		let month = 4;
		let start_day = 10;
		let end_day = 13;

		let start_date: u64 = generate_timestamp_millis(year, month, start_day, 17, 33, 44);
		let end_date: u64 = generate_timestamp_millis(year, month, end_day, 17, 33, 44);

		let amount: u64 = 10;

		let place_id = create_hash("dummy");

		assert_noop!(
			Bookings::create_booking(
				RuntimeOrigin::signed(GUEST_A),
				place_id,
				start_date,
				end_date,
				amount
			),
			PlaceError::<Test>::PlaceNotFound
		);
	})
}

#[test]
fn test_create_booking_with_invalid_dates_should_fail() {
	build_with_defult_place().execute_with(|| {
		let place_id = Places::get_all_places()[0];

		let year = 2025;
		let month = 4;
		let start_day = 10;
		let end_day = 13;

		let start_date: u64 = generate_timestamp_millis(year, month, start_day, 17, 33, 44);
		let end_date: u64 = generate_timestamp_millis(year, month, end_day, 17, 33, 44);
		let amount = 10;

		assert_noop!(
			Bookings::create_booking(
				RuntimeOrigin::signed(GUEST_A),
				place_id,
				end_date, // switched dates
				start_date,
				amount
			),
			Error::<Test>::InvalidDates
		);
	})
}

#[test]
fn test_create_booking_without_funds_should_fail() {
	build_with_defult_place().execute_with(|| {
		let place_id = Places::get_all_places()[0];

		let year = 2025;
		let month = 4;
		let start_day = 10;
		let end_day = 13;

		let start_date: u64 = generate_timestamp_millis(year, month, start_day, 17, 33, 44);
		let end_date: u64 = generate_timestamp_millis(year, month, end_day, 17, 33, 44);
		let amount = 10;

		assert_noop!(
			Bookings::create_booking(
				RuntimeOrigin::signed(3), // This account has no balance
				place_id,
				start_date,
				end_date,
				amount
			),
			Error::<Test>::NotEnoughFreeBalance
		);
	})
}

#[test]
fn test_create_booking_in_owned_place_should_fail() {
	build_with_defult_place().execute_with(|| {
		let place_id = Places::get_all_places()[0];

		let year = 2025;
		let month = 4;
		let start_day = 10;
		let end_day = 13;

		let start_date: u64 = generate_timestamp_millis(year, month, start_day, 17, 33, 44);
		let end_date: u64 = generate_timestamp_millis(year, month, end_day, 17, 33, 44);
		let amount = 10;

		assert_noop!(
			Bookings::create_booking(
				RuntimeOrigin::signed(OWNER),
				place_id,
				start_date,
				end_date,
				amount
			),
			Error::<Test>::CannotBookOwnedPlace
		);
	})
}

#[test]
fn test_create_booking_with_outdated_start_day_should_fail() {
	build_with_defult_place().execute_with(|| {
		let place_id = Places::get_all_places()[0];
		let place_data = Places::get_place_by_id(place_id).unwrap();

		let year = 2025;
		let month = 4;
		let start_day = 10;
		let end_day = 13;

		let start_date: u64 = generate_timestamp_millis(year, month, start_day, 17, 33, 44);
		let end_date: u64 = generate_timestamp_millis(year, month, end_day, 17, 33, 44);
		let amount = 10;

		let current_time = Bookings::modify_timestamp(start_date, place_data.checkin_hour).unwrap();

		// Set current chain time to the expected start_date + 1
		<pallet_places::pallet_timestamp::Pallet<Test>>::set_timestamp(current_time + 1);
		// Advance one block to update chain now() time
		setup_blocks(1);

		assert_noop!(
			Bookings::create_booking(
				RuntimeOrigin::signed(GUEST_A),
				place_id,
				start_date,
				end_date,
				amount
			),
			Error::<Test>::InvalidStartDate
		);
	})
}

// ========================================================
// Confirm Bookings Unit Tests
// ========================================================
#[test]
fn test_confirm_booking_should_work() {
	build_with_defult_place_and_booking().execute_with(|| {
		let booking_id: H256 = Bookings::get_all_bookings()[0];

		assert_ok!(Bookings::confirm_booking(RuntimeOrigin::signed(OWNER), booking_id));

		let booking_data = Bookings::get_booking_by_id(booking_id).unwrap();
		assert_eq!(booking_data.state, BookingState::Confirmed);
	})
}

#[test]
fn test_confirm_missing_booking_should_fail() {
	build_with_funded_accounts().execute_with(|| {
		let booking_id: H256 = create_hash("dummy");

		assert_noop!(
			Bookings::confirm_booking(RuntimeOrigin::signed(GUEST_A), booking_id),
			Error::<Test>::BookingNotFound
		);
	})
}

#[test]
fn test_confirm_booking_not_owner_should_fail() {
	build_with_defult_place_and_booking().execute_with(|| {
		let booking_id: H256 = Bookings::get_all_bookings()[0];

		assert_noop!(
			Bookings::confirm_booking(RuntimeOrigin::signed(GUEST_A), booking_id),
			Error::<Test>::NotPlaceOwner
		);
	})
}

#[test]
fn test_confirm_booking_with_wrong_state_should_fail() {
	build_with_defult_place_and_booking().execute_with(|| {
		let booking_id: H256 = Bookings::get_all_bookings()[0];
		let mut booking_data: BookingData<Test> = Bookings::get_booking_by_id(booking_id).unwrap();
		booking_data.state = BookingState::Confirmed; // state != Created
		BookingsData::insert(booking_id, booking_data);

		assert_noop!(
			Bookings::confirm_booking(RuntimeOrigin::signed(OWNER), booking_id),
			Error::<Test>::WrongState
		);
	})
}

#[test]
fn test_confirm_outdated_booking_should_fail() {
	build_with_defult_place_and_booking().execute_with(|| {
		let booking_id: H256 = Bookings::get_all_bookings()[0];
		let booking_data: BookingData<Test> = Bookings::get_booking_by_id(booking_id).unwrap();

		// Set current chain time to the expected start_date + 1
		<pallet_places::pallet_timestamp::Pallet<Test>>::set_timestamp(booking_data.start_date + 1);
		// Advance one block to update chain now() time
		setup_blocks(1);

		assert_noop!(
			Bookings::confirm_booking(RuntimeOrigin::signed(OWNER), booking_id),
			Error::<Test>::CannotConfirmOutdatedBooking
		);
	})
}

// ========================================================
// Checkin Bookings Unit Tests
// ========================================================
#[test]
fn test_checkin_confirmed_booking_should_work() {
	build_with_default_confirmed_booking().execute_with(|| {
		let booking_id: H256 = Bookings::get_all_bookings()[0];
		let booking_data: BookingData<Test> = Bookings::get_booking_by_id(booking_id).unwrap();

		// Set current chain time to the expected start_date + 1 ot enable the checkin
		<pallet_places::pallet_timestamp::Pallet<Test>>::set_timestamp(booking_data.start_date + 1);
		assert_ok!(Bookings::checkin(RuntimeOrigin::signed(GUEST_A), booking_id));

		// Retrieve latest state
		let booking_data: BookingData<Test> = Bookings::get_booking_by_id(booking_id).unwrap();
		assert_eq!(booking_data.state, BookingState::OwnerCanWithdraw);
	})
}

#[test]
fn test_checkin_booking_with_wrong_guest_should_fail() {
	build_with_default_confirmed_booking().execute_with(|| {
		let booking_id: H256 = Bookings::get_all_bookings()[0];
		let booking_data: BookingData<Test> = Bookings::get_booking_by_id(booking_id).unwrap();

		// Set current chain time to the expected start_date + 1 ot enable the checkin
		<pallet_places::pallet_timestamp::Pallet<Test>>::set_timestamp(booking_data.start_date + 1);
		assert_noop!(
			Bookings::checkin(RuntimeOrigin::signed(GUEST_B), booking_id),
			Error::<Test>::NotPlaceGuest
		);
	})
}

#[test]
fn test_checkin_booking_earlier_should_fail() {
	build_with_default_confirmed_booking().execute_with(|| {
		let booking_id: H256 = Bookings::get_all_bookings()[0];

		assert_noop!(
			Bookings::checkin(RuntimeOrigin::signed(GUEST_A), booking_id),
			Error::<Test>::CheckinNotAvailableYet
		);
	})
}
