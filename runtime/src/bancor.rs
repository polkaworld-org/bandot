
use support::{
	decl_module, decl_storage, decl_event, ensure,
	StorageValue, StorageMap, dispatch::Result
};
use system::ensure_signed;

/// The module's configuration trait.
pub trait Trait: balances::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as BancorModule {
		Base get(base_supply): u128;
        Token get(token_supply): u128;
        Cw get(cw1k): u64;
        Admin get(admin): T::AccountId;
        OwnedToken get(owned_token): map T::AccountId => u128; //T::Balance;
	}
}

// The module's dispatchable functions.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		fn set_bancor(origin, init_base:u128, init_token:u128, init_cw1k:u64) -> Result {
            let sender = ensure_signed(origin)?;
            ensure!(init_base >= 1000, "base must >1000");
            ensure!(init_token >= 1000, "token must >1000");
            ensure!(init_cw1k > 0 && init_cw1k <= 1000 , "cw1k must 0-1000");

            Base::put(init_base);
            Token::put(init_token);
            Cw::put(init_cw1k);
            <Admin<T>>::put(&sender);

            //let token = <T::Balance as As<u64>>::sa(init_token as u64);
            <OwnedToken<T>>::insert(&sender, init_token);

            Self::deposit_event(RawEvent::Created(sender, init_base, init_token, init_cw1k));
            Ok(())
        }

		fn buy(origin, base: u128, token: u128) -> Result{
            let sender = ensure_signed(origin)?;
            let admin = Self::admin();
            ensure!(sender != admin , "Admin cannot buy");

            let admin_token = Self::owned_token(&admin);
            let sender_token = Self::owned_token(&sender);
            ensure!(admin_token >= token, "Not enough supply.");

            let base_sup = Self::base_supply();
            let token_sup = Self::token_supply();
            ensure!(base > 0, "Invalid base");
            ensure!(token > 0 && token <= token_sup, "Invalid token");

            Base::put(base_sup + base);
            Token::put(token_sup - token);
            <OwnedToken<T>>::insert(&admin, admin_token - token);
            <OwnedToken<T>>::insert(&sender, sender_token + token);

            Self::deposit_event(RawEvent::Buy(sender, base, token));

            Ok(())
        }

		fn sell(origin, base: u128, token: u128) -> Result {
            let sender = ensure_signed(origin)?;
            let admin = Self::admin();
            ensure!(sender != admin , "Admin cannot sell");

            let admin_token = Self::owned_token(&admin);
            let sender_token = Self::owned_token(&sender);
            ensure!(sender_token >= token, "Not enough balance.");

            let base_sup = Self::base_supply();
            let token_sup = Self::token_supply();
            ensure!(base > 0 && base <= base_sup, "Invalid base");
            ensure!(token > 0, "Invalid token");

            Base::put(base_sup - base);
            Token::put(token_sup + token);
            <OwnedToken<T>>::insert(&admin, admin_token + token);
            <OwnedToken<T>>::insert(&sender, sender_token - token);
            
            Self::deposit_event(RawEvent::Sell(sender, base, token));
            Ok(())
        }
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		Created(AccountId, u128, u128, u64),
        Buy(AccountId, u128, u128),
        Sell(AccountId, u128, u128),
	}
);