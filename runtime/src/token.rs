use support::{
	decl_module, decl_storage, decl_event, ensure,
	StorageValue, StorageMap, dispatch::Result
};
use codec::{Encode, Decode};
use system::ensure_signed;

/// The module's configuration trait.
pub trait Trait: balances::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
pub struct UserAssets {
	pub staking_amount: u128,
	pub locked_amount: u128,
	pub lending_amount: u128,
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as TokenModule {
		// circulation
		Circulation get(circulation): u128 = 0;
		// Admin account
		Admin get(admin): T::AccountId;
		// BDT balance of user
		BalanceOf get(balance_of): map T::AccountId => u128;
		// Assets of current pooling
		PoolAssets get(pool_assets): u128 = 0;
		// Assets info of current user
		UserAssetsInfo get(user_assets_info): map T::AccountId => UserAssets;
		// set rate fee by Oracle service
		RateFee1k get(rate_fee1k): u8 = 1;
	}
}

// The module's dispatchable functions.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		fn init(origin) -> Result{
			let sender = ensure_signed(origin)?;
			<Admin<T>>::put(&sender);

			Ok(())
		}

		fn mint(origin, to: T::AccountId, value: u128) -> Result {
			let sender = ensure_signed(origin)?;
			ensure!(sender == Self::admin(), "only owner can use!");

			let receiver_balance = Self::balance_of(to.clone());
			let updated_to_balance = receiver_balance.checked_add(value).ok_or("overflow in balance")?;
			<BalanceOf<T>>::insert(to.clone(), updated_to_balance);

			let base_circulation = Self::circulation();
			let updated_circulation = base_circulation.checked_add(value).ok_or("overflow in circulation")?;
			Circulation::put(updated_circulation);

			Self::deposit_event(RawEvent::Mint(to, value));

			Ok(())
		}

		fn burn(origin, to: T::AccountId, value:u128) -> Result {
			let sender = ensure_signed(origin)?;
			ensure!(sender == Self::admin(), "only owner can use!");

			let sender_balance = Self::balance_of(to.clone());
			ensure!(sender_balance >= value, "Not enough balance.");
			let updated_from_balance = sender_balance.checked_sub(value).ok_or("overflow in balance")?;
			<BalanceOf<T>>::insert(to.clone(), updated_from_balance);

			let base_circulation = Self::circulation();
			let updated_circulation = base_circulation.checked_sub(value).ok_or("overflow in circulation")?;
			Circulation::put(updated_circulation);

			Self::deposit_event(RawEvent::Burn(to, value));

			Ok(())
		}

		pub fn transfer(origin, to: T::AccountId, value: u128) -> Result {
			let sender = ensure_signed(origin)?;
			let sender_balance = Self::balance_of(sender.clone());
			ensure!(sender_balance >= value, "Not enough balance.");

			let updated_from_balance = sender_balance.checked_sub(value).ok_or("overflow in calculating balance")?;
			let receiver_balance = Self::balance_of(to.clone());
			let updated_to_balance = receiver_balance.checked_add(value).ok_or("overflow in calculating balance")?;

			// reduce sender balance
			<BalanceOf<T>>::insert(sender.clone(), updated_from_balance);
			// add receiver balance
			<BalanceOf<T>>::insert(to.clone(), updated_to_balance);

			Self::deposit_event(RawEvent::Transfer(sender, to, value));
			Ok(())
		}

		pub fn deposit(origin, amount: u128) -> Result {
			let owner = ensure_signed(origin)?;
			let mut user_asserts = Self::user_assets_info(owner.clone());

			let pre_staking_amount = user_asserts.staking_amount;

			user_asserts.staking_amount = pre_staking_amount.checked_add(amount).ok_or("overflow in staking")?;
			<UserAssetsInfo<T>>::insert(owner.clone(), user_asserts);

			let pre_pool_assets = Self::pool_assets();
			let updated_pool_assets = pre_pool_assets.checked_add(amount).ok_or("overflow in pool")?;
			PoolAssets::put(updated_pool_assets);

			Self::deposit_event(RawEvent::Deposit(owner, amount, updated_pool_assets));
			Ok(())
		}

		pub fn exchange(origin, amount: u128) -> Result {
			let owner = ensure_signed(origin)?;
			let mut user_asserts = Self::user_assets_info(owner.clone());

			// pre-exchange
			let pre_locked_amount = user_asserts.locked_amount;
			let pre_lending_amount = user_asserts.lending_amount;

			ensure!(pre_lending_amount >= pre_lending_amount.checked_add(amount).ok_or("overflow in lending")?, "Not enough stake.");
			user_asserts.lending_amount = pre_lending_amount.checked_add(amount).ok_or("overflow in lending")?;
			user_asserts.locked_amount = pre_locked_amount.checked_add(amount).ok_or("overflow in locking")?;

			<UserAssetsInfo<T>>::insert(owner.clone(), user_asserts);

			let fee = 1u128;//temporary fee
			// send token
			let token_amount = amount.checked_sub(fee).ok_or("overflow in token")?;
			let receiver_balance = Self::balance_of(owner.clone());
			let updated_to_balance = receiver_balance.checked_add(token_amount).ok_or("overflow in balance")?;
			<BalanceOf<T>>::insert(owner.clone(), updated_to_balance);

			let base_circulation = Self::circulation();
			let updated_circulation = base_circulation.checked_add(token_amount).ok_or("overflow in circulation")?;
			Circulation::put(updated_circulation);

			Self::deposit_event(RawEvent::Exchange(owner, amount, fee));
			Ok(())
		}

		pub fn set_fee(origin, fee: u8) -> Result {
			let sender = ensure_signed(origin)?;
            ensure!(sender == Self::admin(), "only owner can use!");

			RateFee1k::put(fee);
			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		Mint(AccountId, u128),
		Burn(AccountId, u128),
		Transfer(AccountId, AccountId, u128),
		Deposit(AccountId, u128, u128),
		Exchange(AccountId, u128, u128),
	}
);