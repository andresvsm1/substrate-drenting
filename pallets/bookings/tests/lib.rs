#[cfg(test)]
pub mod mock;
use frame_support::{assert_noop, assert_ok};
use pallet_bookings::{BookingData, BookingState};
use pallet_places::PlaceType;

use crate::mock::*;
use sp_core::H256;

fn create_hash(data: &str) -> H256 {
	let bytes = data.as_bytes();
	let mut array = [0; 32];
	array[..bytes.len()].copy_from_slice(bytes);
	H256::from_slice(&array)
}

fn create_demo_place() {
	let _ = Places::create_place(
		RuntimeOrigin::signed(0),
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

fn build_with_funded_accounts() -> sp_io::TestExternalities {
	build_with_default_config(vec![(1, *b"123456789012345a")])
}

fn build_with_defult_place() -> sp_io::TestExternalities {
	let mut ext = build_with_funded_accounts();
	ext.execute_with(create_demo_place);
	ext
}

// ========================================================
// Functions Unit Tests
// ========================================================
#[test]
fn test_modify_timestamp_function() {
	// Should work with seconds
	build_with_funded_accounts().execute_with(|| {
		let start_date = 1696922449;
		let checkin_hour = 17;
		let end_date = 1697354449;
		let checkout_hour = 12;

		// Expected times
		let expected_start_date = 1696957200000;
		let expected_end_date = 1697371200000;

		assert_eq!(
			Bookings::modify_timestamp(start_date, checkin_hour).unwrap(),
			expected_start_date
		);
		assert_eq!(Bookings::modify_timestamp(end_date, checkout_hour).unwrap(), expected_end_date);
	});

	// Should work with milliseconds
	build_with_funded_accounts().execute_with(|| {
		let start_date = 1696922449000;
		let checkin_hour = 17;
		let end_date = 1697354449000;
		let checkout_hour = 12;

		// Expected times
		let expected_start_date = 1696957200000;
		let expected_end_date = 1697371200000;

		assert_eq!(
			Bookings::modify_timestamp(start_date, checkin_hour).unwrap(),
			expected_start_date
		);
		assert_eq!(Bookings::modify_timestamp(end_date, checkout_hour).unwrap(), expected_end_date);
	});

	build_with_funded_accounts().execute_with(|| {
		let start_date = 1696922449;
		let checkin_hour = 25; // Should fail

		assert_noop!(
			Bookings::modify_timestamp(start_date, checkin_hour),
			sp_runtime::DispatchError::Other("desired time out of range")
		);
	});
}

// ========================================================
// Bookings Unit Tests
// ========================================================
#[test]
fn test_create_booking_should_work() {
	build_with_defult_place().execute_with(|| {
		let place_id = Places::get_all_places()[0];
		let place_data = Places::get_place_by_id(place_id).unwrap();

		let start_date = 1696922449;
		let end_date = 1697354449;
		let amount = 10;

		// let place_data = Places::get_place_by_id(place_id);
		// println!("{:?}", place_data);
		assert_ok!(Bookings::create_booking(
			RuntimeOrigin::signed(1),
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
