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
    use ink_prelude::vec::Vec;
    use ink_prelude::string::String;
    type CampaignId = Vec<u8>;
    type CampaignCount = u64;

    #[derive(PackedLayout, SpreadLayout, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Campaign {
        title: String,
        description: String,
        is_live: bool,
        initiator: AccountId,
        deadline: u64,
        balance: Balance,
    }

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Charity {
        /// Stores a single `bool` value on the storage.
        campaign_list: Mapping<CampaignCount, CampaignId>,
        campaigns: Mapping<CampaignId, Campaign>,
        user_campaign_donations: Mapping<AccountId, (CampaignId, Balance)>,
        campaign_count: CampaignCount,
    }

    impl Charity {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new() -> Self {
            ink_lang::utils::initialize_contract(|_| {
                // Self::new_init();
            })
        }

        // fn new_init() -> Self {
        //     Self {
        //         campaign_count: u64::default(),
        //         campaign_list: Mapping::default(),
        //         campaigns: Mapping::default(),
        //         user_campaign_donations: Mapping::default(),
        //     }
        // }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        // #[ink(constructor)]
        // pub fn default() -> Self {
        //     Self::new(Default::default())
        // }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn create_campaign(&mut self, title: String, description: String, deadline: u64) {
            self.campaign_count += 1;
            let id_campaign = self.campaign_count.to_be_bytes().to_vec();

            self.campaign_list.insert(self.campaign_count, &id_campaign);

            let campaign = Campaign {
                balance: 0,
                description,
                is_live: true,
                title,
                initiator: self.env().caller(),
                deadline,
            };

            self.campaigns.insert(&id_campaign, &campaign);

            // self.user_campaign_donations.insert(self.env().caller(), (id_campaign, ))
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn get_campaign_count(&self) -> CampaignCount {
            self.campaign_count
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn get_campaign_id(&self, count: CampaignCount) -> CampaignId {
            self.campaign_list.get(count).unwrap_or_default()
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn get_campaign(&self, campaign_id: CampaignId) -> Campaign {
            self.campaigns.get(campaign_id).unwrap()
        }

        // fn random_id_campaign() -> Vec<u8> {
            // let random: u32 = rng.gen();
            // random.to_be_bytes().to_vec()
        // }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {}

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {}
    }
}
