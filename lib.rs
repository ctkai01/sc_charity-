#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod charity {
    // use rand::{self, prelude::ThreadRng};
    use ink_storage::{
        traits::{PackedLayout, SpreadAllocate, SpreadLayout},
        Mapping,
    };
    // use scale_info::TypeInfo;
    use ink_prelude::string::String;
    use ink_prelude::vec::Vec;
    use scale::{Decode, Encode};
    type CampaignId = Vec<u8>;
    type CampaignCount = u64;

    #[derive(PackedLayout, SpreadLayout, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Campaign {
        title: String,
        description: String,
        initiator: AccountId,
        deadline: u64,
        balance: Balance,
    }

    /// Event emitted when anyone donated for a campaign
    #[ink(event)]
    pub struct FundsDonated {
        id_campaign: CampaignId,
        sender: AccountId,
        value: Balance,
    }

    /// Event emitted when an Campaign ended
    #[ink(event)]
    pub struct CampaignEnded {
        id_campaign: CampaignId,
        initiator: AccountId,
    }

    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Campaign Ended
        CampaignEnded,
        /// Campaign not found
        NotCampaign,
        /// Not Campaign initiator
        NotCampaignInitiator,
        /// Campaign is not live"
        CampaignNotLive,
        /// Campaign is  live"
        CampaignIsLive,
        /// No funds withdraw"
        NoFundsWithdraw,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Charity {
        /// Stores a single `bool` value on the storage.
        campaign_list: Mapping<CampaignCount, CampaignId>,
        campaigns: Mapping<CampaignId, Campaign>,
        campaign_count: CampaignCount,
        // user_campaign_donations: Mapping<AccountId, Mapping<CampaignId, Balance>>,
    }

    impl Charity {
        ///
        #[ink(constructor)]
        pub fn new() -> Self {
            // ink_lang::utils::initialize_contract(|_| {
            //     // Self::new_init();
            // })
            ink_lang::utils::initialize_contract(|_| {})
        }

        /// A message created campaign
        #[ink(message)]
        pub fn create_campaign(&mut self, title: String, description: String, deadline: u64) {
            self.campaign_count += 1;
            let id_campaign = self.campaign_count.to_be_bytes().to_vec();

            self.campaign_list.insert(self.campaign_count, &id_campaign);

            let campaign = Campaign {
                balance: 0,
                description,
                title,
                initiator: self.env().caller(),
                deadline,
            };

            self.campaigns.insert(&id_campaign, &campaign);
        }

        /// A message that can be donated campaign
        #[ink(message, payable)]
        pub fn donate_campaign(&mut self, id_campaign: CampaignId) -> Result<()> {
            let mut campaign_donated =
                self.campaigns.get(&id_campaign).ok_or(Error::NotCampaign)?;

            let caller = self.env().caller();
            let money_donated = self.env().transferred_value();

            if campaign_donated.deadline < self.env().block_timestamp() / 1000 {
                return Err(Error::CampaignEnded);
            }

            campaign_donated.balance += money_donated;

            self.campaigns.insert(&id_campaign, &campaign_donated);

            self.env().emit_event(FundsDonated {
                id_campaign,
                sender: caller,
                value: money_donated,
            });
            Ok(())
        }

        /// A message that can be ended campaign by initiator
        #[ink(message)]
        pub fn ended_campaign(&mut self, id_campaign: CampaignId) -> Result<()> {
            let mut campaign_ended = self.campaigns.get(&id_campaign).ok_or(Error::NotCampaign)?;
            let caller = self.env().caller();
            if caller != campaign_ended.initiator {
                return Err(Error::NotCampaignInitiator);
            }
            if campaign_ended.deadline < self.env().block_timestamp() / 1000 {
                return Err(Error::CampaignNotLive);
            }

            campaign_ended.deadline = self.env().block_timestamp() / 1000;
            self.campaigns.insert(&id_campaign, &campaign_ended);

            self.env().emit_event(CampaignEnded {
                id_campaign,
                initiator: caller,
            });
            Ok(())
        }

        /// A message that can be withdraw campaign funds for initiator
        #[ink(message)]
        pub fn withdraw_campaign_funds(&mut self, id_campaign: CampaignId) -> Result<()> {
            let mut campaign = self.campaigns.get(&id_campaign).ok_or(Error::NotCampaign)?;
            let caller = self.env().caller();
            if caller != campaign.initiator {
                return Err(Error::NotCampaignInitiator);
            }
            if campaign.deadline > self.env().block_timestamp() / 1000 {
                return Err(Error::CampaignIsLive);
            }

            if campaign.balance == 0 {
                return Err(Error::NoFundsWithdraw);
            }

            let amount_withdraw = campaign.balance;

            campaign.balance = 0;

            if self
                .env()
                .transfer(campaign.initiator, amount_withdraw)
                .is_err()
            {
                panic!(
                    "requested transfer failed. this can be the case if the contract does not\
                         have sufficient free funds or if the transfer would have brought the\
                         contract's balance below minimum balance."
                )
            }
            Ok(())
        }

        /// A message that can be get campaign count
        #[ink(message)]
        pub fn get_campaign_count(&self) -> CampaignCount {
            self.campaign_count
        }

        /// A message that can be get_campaign_id by count
        #[ink(message)]
        pub fn get_campaign_id(&self, count: CampaignCount) -> CampaignId {
            self.campaign_list.get(count).unwrap_or_default()
        }

        /// A message that can be get_campaign by campaign id
        #[ink(message)]
        pub fn get_campaign(&self, campaign_id: CampaignId) -> Campaign {
            self.campaigns.get(campaign_id).unwrap()
        }

        pub fn current_timestamp_block(&self) -> u64 {
            self.env().block_timestamp()
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use ink_env::test;
        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        fn set_caller(sender: AccountId) {
            ink_env::test::set_caller::<Environment>(sender);
        }

        fn default_accounts() -> test::DefaultAccounts<Environment> {
            ink_env::test::default_accounts::<Environment>()
        }

        fn set_alice_caller() {
            let accounts = default_accounts();
            set_caller(accounts.alice);
        }

        fn get_address_alice() -> AccountId {
            let accounts = default_accounts();
            accounts.alice
        }

        // fn get_address_bob() -> AccountId {
        //     let accounts = default_accounts();
        //     accounts.bob
        // }

        fn set_bob_caller() {
            let accounts = default_accounts();
            set_caller(accounts.bob);
        }

        fn set_balance(account_id: AccountId, balance: Balance) {
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(account_id, balance)
        }

        fn contract_id() -> AccountId {
            ink_env::test::callee::<ink_env::DefaultEnvironment>()
        }

        fn create_contract(initial_balance: Balance) -> Charity {
            set_alice_caller();
            set_balance(contract_id(), initial_balance);
            Charity::new()
        }

        fn get_balance(account_id: AccountId) -> Balance {
            ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(account_id)
                .expect("Cannot get account balance")
        }

        /// Test created campaign
        #[ink::test]
        fn create_campaign() {
            let mut charity = create_contract(0);
            set_alice_caller();
            let title_test = String::from("Kitty");
            let description_test = String::from("Hello Kitty");
            let deadline_test = charity.current_timestamp_block() + 1000;

            charity.create_campaign(
                title_test.clone(),
                description_test.clone(),
                deadline_test.clone(),
            );
            let campaign_id = charity.get_campaign_id(1);
            let campaign_created = charity.get_campaign(campaign_id);

            println!("created new instance at {:?}", charity.get_campaign_count());

            assert_eq!(campaign_created.title, title_test);
            assert_eq!(campaign_created.balance, 0);
            assert_eq!(campaign_created.description, description_test);
            assert_eq!(campaign_created.deadline, deadline_test);
            assert_eq!(campaign_created.initiator, get_address_alice());
            assert_eq!(charity.get_campaign_count(), 1);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn donated_campaign() {
            let mut charity = create_contract(0);
            set_alice_caller();
            let title_test = String::from("Kitty");
            let description_test = String::from("Hello Kitty");
            let deadline_test = charity.current_timestamp_block() + 1000;

            charity.create_campaign(
                title_test.clone(),
                description_test.clone(),
                deadline_test.clone(),
            );
            set_bob_caller();
            let campaign_id = charity.get_campaign_id(1);

            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(200);

            match charity.donate_campaign(campaign_id.clone()).err() {
                Some(_) => {
                    assert!(false)
                }
                None => {}
            };

            let campaign = charity.get_campaign(campaign_id);

            assert_eq!(campaign.balance, 200);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn ended_campaign() {
            let mut charity = create_contract(0);
            set_alice_caller();

            let title_test = String::from("Kitty");
            let description_test = String::from("Hello Kitty");
            let deadline_test = charity.current_timestamp_block() + 1000;

            charity.create_campaign(
                title_test.clone(),
                description_test.clone(),
                deadline_test.clone(),
            );
            let campaign_id = charity.get_campaign_id(1);
            match charity.ended_campaign(campaign_id.clone()).err() {
                Some(_) => {
                    assert!(false)
                }
                None => {}
            };

            let campaign = charity.get_campaign(campaign_id);

            assert_eq!(campaign.deadline, charity.current_timestamp_block());
        }

        /// We test a simple use case of our contract.
        ///
        #[ink::test]
        fn ended_campaign_must_initiator() {
            let mut charity = create_contract(0);
            set_alice_caller();

            let title_test = String::from("Kitty");
            let description_test = String::from("Hello Kitty");
            let deadline_test = charity.current_timestamp_block() + 1000;

            charity.create_campaign(
                title_test.clone(),
                description_test.clone(),
                deadline_test.clone(),
            );
            let campaign_id = charity.get_campaign_id(1);

            set_bob_caller();

            match charity.ended_campaign(campaign_id.clone()).err() {
                Some(err) => {
                    assert!(err == crate::charity::Error::NotCampaignInitiator)
                }
                None => {
                    assert!(false)
                }
            };
          
        }

        /// We test a simple use case of our contract.
        ///
        #[ink::test]
        fn withdraw_campaign_funds() {
            let mut charity = create_contract(0);
            set_alice_caller();

            let title_test = String::from("Kitty");
            let description_test = String::from("Hello Kitty");
            let deadline_test = charity.current_timestamp_block() + 1000;
            let balance_before = get_balance(get_address_alice());
            println!("Balance before: {}", balance_before);
            charity.create_campaign(
                title_test.clone(),
                description_test.clone(),
                deadline_test.clone(),
            );
            let campaign_id = charity.get_campaign_id(1);

            set_bob_caller();
            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(200);

            match charity.donate_campaign(campaign_id.clone()).err() {
                Some(_) => {
                    assert!(false)
                }
                None => {}
            };

            set_alice_caller();

            match charity.ended_campaign(campaign_id.clone()).err() {
                Some(_) => {
                    assert!(false)
                }
                None => {}
            };
            charity.withdraw_campaign_funds(campaign_id.clone());
            let balance_after = get_balance(get_address_alice());
            println!("Balance before: {}", balance_after);
        }
    }
}
