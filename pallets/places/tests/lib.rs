#[cfg(test)]
pub mod mock;
use frame_support::{assert_noop, assert_ok};

use crate::mock::*;
use pallet_places::*;
use sp_core::H256;
use structures::PlaceData;

fn create_hash(data: &str) -> H256 {
	let bytes = data.as_bytes();
	let mut array = [0; 32];
	array[..bytes.len()].copy_from_slice(bytes);
	H256::from_slice(&array)
}

fn create_default_place() {
	let _ = Places::create_place(
		RuntimeOrigin::signed(1),
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

fn build_with_demo_place() -> sp_io::TestExternalities {
	let mut ext = build_with_default_config();
	ext.execute_with(create_default_place);
	ext
}

#[test]
fn test_create_place_should_work() {
	build_with_default_config().execute_with(|| {
		assert_ok!(Places::create_place(
			RuntimeOrigin::signed(1),
			PlaceType::Apartment,
			b"Demo Place".to_vec(),
			b"Demo Address".to_vec(),
			create_hash("Demo Description"),
			10,
			17,
			12,
			vec![create_hash("image_1"), create_hash("image_2")],
			None,
		));

		let place_id = Places::get_all_places()[0];
		let place_data = Places::get_place_by_id(place_id);

		// Check the place has been created correctly
		assert_eq!(
			place_data,
			Some(PlaceData {
				place_type: PlaceType::Apartment,
				name: b"Demo Place".to_vec(),
				address: b"Demo Address".to_vec(),
				description: create_hash("Demo Description"),
				price_per_night: 10,
				checkin_hour: 17,
				checkout_hour: 12,
				active: true,
				images: vec![create_hash("image_1"), create_hash("image_2")].into_iter().collect(),
				number_of_floors: 1,
				owner: 1,
				on_chain_creation: AuditTrail { account: 1, block: 0, time: 0 },
				on_chain_update: None
			})
		);
	})
}

#[test]
fn test_update_place_should_work() {
	build_with_demo_place().execute_with(|| {
		let place_id = Places::get_all_places()[0];

		let new_place_type = Some(PlaceType::House);
		let new_name = Some(b"Demo Place 2".to_vec());
		let new_address = Some(b"Demo Address 2".to_vec());
		let new_description = Some(create_hash("Demo Description 2"));
		let new_price_per_night = Some(20);
		let new_checkin_hour = Some(18);
		let new_checkout_hour = Some(11);
		let new_images = Some(vec![create_hash("image_3"), create_hash("image_2")]);
		let new_number_of_floors = Some(2);

		assert_ok!(Places::update_place(
			RuntimeOrigin::signed(1),
			place_id,
			new_place_type,
			new_name,
			new_address,
			new_description,
			new_price_per_night,
			new_checkin_hour,
			new_checkout_hour,
			new_images,
			new_number_of_floors,
		));

		let place_data = Places::get_place_by_id(place_id);

		// Check the place has been created correctly
		assert_eq!(
			place_data,
			Some(PlaceData {
				place_type: PlaceType::House,
				name: b"Demo Place 2".to_vec(),
				address: b"Demo Address 2".to_vec(),
				description: create_hash("Demo Description 2"),
				price_per_night: 20,
				checkin_hour: 18,
				checkout_hour: 11,
				active: true,
				images: vec![
					create_hash("image_1"),
					create_hash("image_2"),
					create_hash("image_3")
				]
				.into_iter()
				.collect(),
				number_of_floors: 2,
				owner: 1,
				on_chain_creation: AuditTrail { account: 1, block: 0, time: 0 },
				on_chain_update: Some(AuditTrail { account: 1, block: 0, time: 0 }),
			})
		);
	})
}

#[test]
fn test_remove_place_should_work() {
	build_with_demo_place().execute_with(|| {
		let place_id = Places::get_all_places()[0];

		assert_ok!(Places::remove_place(RuntimeOrigin::signed(1), place_id));

		let places = PlacesData::<Test>::iter().next();
		assert_eq!(Places::get_all_places().len(), 0);
		assert_eq!(places, None);
	})
}
