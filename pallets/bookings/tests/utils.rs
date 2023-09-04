use chrono::NaiveDate;
use sp_core::H256;

pub fn create_hash(data: &str) -> H256 {
	let bytes = data.as_bytes();
	let mut array = [0; 32];
	array[..bytes.len()].copy_from_slice(bytes);
	H256::from_slice(&array)
}

pub fn generate_timestamp(
	year: i32,
	month: u32,
	day: u32,
	hour: u32,
	minute: u32,
	second: u32,
) -> u64 {
	NaiveDate::from_ymd_opt(year, month, day)
		.unwrap()
		.and_hms_opt(hour, minute, second)
		.unwrap()
		.timestamp()
		.try_into()
		.unwrap()
}

pub fn generate_timestamp_millis(
	year: i32,
	month: u32,
	day: u32,
	hour: u32,
	minute: u32,
	second: u32,
) -> u64 {
	NaiveDate::from_ymd_opt(year, month, day)
		.unwrap()
		.and_hms_opt(hour, minute, second)
		.unwrap()
		.timestamp_millis()
		.try_into()
		.unwrap()
}
