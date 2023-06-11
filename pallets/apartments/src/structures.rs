

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct DoctorInfo {
	pub name: Vec<u8>,
	pub email: Vec<u8>,
	pub country: CountryCode,
	pub region: RegionCode,
	pub city: CityCode,
	pub address: Vec<u8>,
	pub latitude: Option<Vec<u8>>,
	pub longitude: Option<Vec<u8>>,
	pub profile_image: Option<Vec<u8>>,
}