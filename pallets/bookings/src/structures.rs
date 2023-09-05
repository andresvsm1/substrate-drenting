#![cfg_attr(not(feature = "std"), no_std)]

use super::{BalanceOf, Config};
use codec::{Decode, Encode};
use frame_support::sp_std::prelude::*;
use scale_info::TypeInfo;

#[derive(Eq, PartialEq, Encode, Decode, TypeInfo, Clone, Debug)]
pub enum BookingState {
	Created,
	Confirmed,
	Rejected,
	Withdrawable,
	UserCanWithdraw,
	OwnerCanWithdraw,
	Completed,
}

#[derive(Eq, PartialEq, Encode, Decode, TypeInfo, Clone, Debug)]
#[scale_info(skip_type_params(T))]
pub struct BookingData<T: Config> {
	pub place_id: T::Hash,
	pub host: T::AccountId,
	pub guest: T::AccountId,
	pub start_date: T::Moment,
	pub end_date: T::Moment,
	pub amount: BalanceOf<T>,
	pub state: BookingState,
}

impl<T: Config> BookingData<T> {
	pub fn new(
		place_id: T::Hash,
		host: T::AccountId,
		guest: T::AccountId,
		start_date: T::Moment,
		end_date: T::Moment,
		amount: BalanceOf<T>,
	) -> Self {
		BookingData {
			place_id,
			host,
			guest,
			start_date,
			end_date,
			amount,
			state: BookingState::Created,
		}
	}
}

#[derive(Encode, Decode, Clone, PartialEq, TypeInfo, Debug)]
#[scale_info(skip_type_params(T))]
pub struct BookingHashingData<T: Config> {
	pub place_id: T::Hash,
	pub host: T::AccountId,
	pub guest: T::AccountId,
	pub start_date: T::Moment,
	pub end_date: T::Moment,
	pub amount: BalanceOf<T>,
}

impl<T: Config> From<BookingData<T>> for BookingHashingData<T> {
	fn from(from: BookingData<T>) -> Self {
		let BookingData { place_id, host, guest, start_date, end_date, amount, .. } = from;

		Self { place_id, host, guest, start_date, end_date, amount }
	}
}
