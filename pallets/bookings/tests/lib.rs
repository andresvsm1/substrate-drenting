#[cfg(test)]
pub mod mock;
pub mod utils;
use frame_support::{assert_noop, assert_ok, traits::Currency};
use pallet_bookings::{BookingData, BookingState, BookingsData, Error};
use pallet_places::{Error as PlaceError, PlaceType};
use sp_core::H256;

use crate::{mock::*, utils::*};

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
	// To emit events, we need to be past block 0
	setup_blocks(1);
}

fn create_default_booking() {
	let year = 2025;
	let month = 4;
	let start_day = 10;
	let end_day = 13;
	let start_date: u64 = generate_timestamp(year, month, start_day, 17, 33, 44);
	let end_date: u64 = generate_timestamp(year, month, end_day, 17, 33, 44);

	let place_id: H256 = Places::get_all_places()[0];
	let place_data = Places::get_place_by_id(place_id).unwrap();

	let n_days = end_day - start_day;
	let amount = (n_days as u64) * place_data.price_per_night;

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

		let n_days = end_day - start_day;
		let amount = (n_days as u64) * place_data.price_per_night;

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

		// Check GUEST funds have been locked
		let reserved_balances = Balances::reserved_balance(&GUEST_A);
		assert_eq!(reserved_balances, amount);

		// Check emitted events
		System::assert_last_event(
			pallet_bookings::Event::BookingPlaced { id: booking_id, sender: GUEST_A }.into(),
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
		let place_data = Places::get_place_by_id(place_id).unwrap();

		let year = 2025;
		let month = 4;
		let start_day = 10;
		let end_day = 13;

		let start_date: u64 = generate_timestamp_millis(year, month, start_day, 17, 33, 44);
		let end_date: u64 = generate_timestamp_millis(year, month, end_day, 17, 33, 44);

		let n_days = end_day - start_day;
		let amount = (n_days as u64) * place_data.price_per_night;

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
		let place_data = Places::get_place_by_id(place_id).unwrap();

		let year = 2025;
		let month = 4;
		let start_day = 10;
		let end_day = 13;

		let start_date: u64 = generate_timestamp_millis(year, month, start_day, 17, 33, 44);
		let end_date: u64 = generate_timestamp_millis(year, month, end_day, 17, 33, 44);

		let n_days = end_day - start_day;
		let amount = (n_days as u64) * place_data.price_per_night;

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
		let place_data = Places::get_place_by_id(place_id).unwrap();

		let year = 2025;
		let month = 4;
		let start_day = 10;
		let end_day = 13;

		let start_date: u64 = generate_timestamp_millis(year, month, start_day, 17, 33, 44);
		let end_date: u64 = generate_timestamp_millis(year, month, end_day, 17, 33, 44);

		let n_days = end_day - start_day;
		let amount = (n_days as u64) * place_data.price_per_night;

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

		let n_days = end_day - start_day;
		let amount = (n_days as u64) * place_data.price_per_night;

		let current_time = Bookings::modify_timestamp(start_date, place_data.checkin_hour).unwrap();

		// Set current chain time to the expected start_date + 1
		<pallet_places::pallet_timestamp::Pallet<Test>>::set_timestamp(current_time + 1);
		// Advance one block to update chain now() time
		setup_blocks(2);

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

		// Check emitted events
		System::assert_last_event(
			pallet_bookings::Event::BookingUpdated {
				id: booking_id,
				sender: OWNER,
				state: BookingState::Confirmed,
			}
			.into(),
		);
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
		setup_blocks(2);

		assert_noop!(
			Bookings::confirm_booking(RuntimeOrigin::signed(OWNER), booking_id),
			Error::<Test>::CannotConfirmOutdatedBooking
		);
	})
}

// ========================================================
// Reject Bookings Unit Tests
// ========================================================
#[test]
fn test_reject_booking_should_work() {
	build_with_defult_place_and_booking().execute_with(|| {
		let place_id = Places::get_all_places()[0];
		let booking_id: H256 = Bookings::get_all_bookings()[0];
		let bookings_data = Bookings::get_booking_by_id(booking_id).unwrap();

		assert_ok!(Bookings::reject_booking(RuntimeOrigin::signed(OWNER), booking_id));

		let booking_data = Bookings::get_booking_by_id(booking_id).unwrap();
		assert_eq!(booking_data.state, BookingState::Rejected);

		// Ensure structures have been updated correctly
		assert_eq!(Bookings::get_place_bookings(place_id), vec![]);
		assert_eq!(Bookings::get_pending_booking_withdraws_by_account(OWNER), vec![]);
		assert_eq!(
			Bookings::get_pending_booking_withdraws_by_account(GUEST_A),
			vec![(booking_id, bookings_data.amount)]
		);

		// Check emitted events
		System::assert_last_event(
			pallet_bookings::Event::BookingUpdated {
				id: booking_id,
				sender: OWNER,
				state: BookingState::Rejected,
			}
			.into(),
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

// ========================================================
// Withdrawals Unit Tests
// ========================================================
#[test]
fn test_withdraw_checked_in_booking_should_work() {
	// Caller: OWNER
	build_with_default_confirmed_booking().execute_with(|| {
		let place_id = Places::get_all_places()[0];
		let booking_id: H256 = Bookings::get_all_bookings()[0];
		let booking_data: BookingData<Test> = Bookings::get_booking_by_id(booking_id).unwrap();

		// Set current chain time to the expected start_date + 1 ot enable the checkin
		<pallet_places::pallet_timestamp::Pallet<Test>>::set_timestamp(booking_data.start_date + 1);
		assert_ok!(Bookings::checkin(RuntimeOrigin::signed(GUEST_A), booking_id));

		// As the owner, withdraw the booking
		assert_ok!(Bookings::withdraw_booking(RuntimeOrigin::signed(OWNER), booking_id));

		// Retrieve latest state
		let booking_data: BookingData<Test> = Bookings::get_booking_by_id(booking_id).unwrap();
		assert_eq!(booking_data.state, BookingState::Completed);

		// Check OWNER has successfully withdrawed the funds
		assert_eq!(Balances::total_balance(&OWNER), BASE_TOKEN_AMOUNT + booking_data.amount);
		// Check GUEST_A has less funds
		assert_eq!(Balances::total_balance(&GUEST_A), BASE_TOKEN_AMOUNT - booking_data.amount);

		// Check structs have been updated correctly
		assert_eq!(Bookings::get_place_bookings(booking_id), vec![]);
		assert_eq!(Bookings::get_pending_booking_withdraws_by_account(OWNER), vec![]);
		assert_eq!(Bookings::get_place_bookings(place_id), vec![]);

		// Check emitted events
		System::assert_last_event(
			pallet_bookings::Event::BookingUpdated {
				id: booking_id,
				sender: OWNER,
				state: BookingState::Completed,
			}
			.into(),
		);
	})
}

#[test]
fn test_withdraw_rejected_booking_should_work() {
	// Caller: GUEST_A
	build_with_defult_place_and_booking().execute_with(|| {
		let place_id = Places::get_all_places()[0];
		let booking_id: H256 = Bookings::get_all_bookings()[0];

		// Reject booking
		assert_ok!(Bookings::reject_booking(RuntimeOrigin::signed(OWNER), booking_id));

		// As the GUEST_A, withdraw the rejected booking
		assert_ok!(Bookings::withdraw_booking(RuntimeOrigin::signed(GUEST_A), booking_id));

		// Retrieve latest state
		let booking_data: BookingData<Test> = Bookings::get_booking_by_id(booking_id).unwrap();
		assert_eq!(booking_data.state, BookingState::Completed);

		// Check GUEST_A has successfully unreserved his funds
		assert_eq!(Balances::reserved_balance(&GUEST_A), 0);

		// Check structs have been updated correctly
		assert_eq!(Bookings::get_place_bookings(booking_id), vec![]);
		assert_eq!(Bookings::get_pending_booking_withdraws_by_account(OWNER), vec![]);
		assert_eq!(Bookings::get_pending_booking_withdraws_by_account(GUEST_A), vec![]);
		assert_eq!(Bookings::get_place_bookings(place_id), vec![]);

		// Check emitted events
		System::assert_last_event(
			pallet_bookings::Event::BookingUpdated {
				id: booking_id,
				sender: GUEST_A,
				state: BookingState::Completed,
			}
			.into(),
		);
	})
}
