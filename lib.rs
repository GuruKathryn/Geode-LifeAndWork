/*
ABOUT THIS CONTRACT...
This contract handles claims that can be made with metadata that can be 
stored in a single u8 vector. This covers the following use cases:
1 - Work History (as keywords such as 'CTO @ Wave Technologies')
2 - Education (as keywords such as 'PhD Math @ Univeristy of Florida')
3 - Expertise (as keywords such as 'Blockchain' or 'Personal Development')
4 - Good Deeds (as beneficiary - date - location - keywords)
5 - Original IP (as title - keywords - file hash)

IMPORTANT: THE GET_DETAILS MESSAGE RETURNS THE DETAILS OF A GIVEN CLAIM
HASH, STARTING WITH A CODE NUMBER (1-5) TO TELL YOU WHAT TYPE OF CLAIM IT WAS.

NOTE: This contract offers a uniquely titled copy of the same message 
and associated event for each use case when making a claim so that we 
will know how to parse the u8 vector of claim metadata later in the front
end. Only a single message and associated event type is needed for claim 
endorsements. 
*/

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod life_and_work {

    use ink::prelude::vec::Vec;
    use ink::prelude::vec;
    use ink::prelude::string::String;
    use ink::storage::Mapping;
    use ink::storage::StorageVec;
    use ink::env::hash::{Sha2x256, HashOutput};


    // PRELIMINARY DATA STRUCTURES >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    #[derive(Clone, Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std",derive(ink::storage::traits::StorageLayout,))]
    pub struct Details {
        claimtype: u8,
        claimant: AccountId,
        claim: Vec<u8>,
        claim_id: Hash,
        endorser_count: u128,
        link: Vec<u8>,
        show: bool,
        endorsers: Vec<AccountId>
    }

    impl Default for Details {
        fn default() -> Details {
            Details {
                claimtype: 0,
                claimant: AccountId::from([0x0; 32]),
                claim: <Vec<u8>>::default(),
                claim_id: Hash::default(),
                endorser_count: 0,
                link: <Vec<u8>>::default(),
                show: true,
                endorsers: <Vec<AccountId>>::default(),
            }
        }
    }
   

    #[derive(Clone, Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std",derive(ink::storage::traits::StorageLayout,))]
    pub struct Claims {
        claims: Vec<Hash>
    }

    impl Default for Claims {
        fn default() -> Claims {
            Claims {
                claims: <Vec<Hash>>::default(),
            }
        }
    }


    #[derive(Clone, Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std",derive(ink::storage::traits::StorageLayout,))]
    pub struct RewardSettings {
        reward_on: u8,
        reward_root_set: u8,
        reward_root: AccountId,
        reward_interval: u128,
        reward_amount: Balance,
        reward_balance: Balance,
        reward_payouts: Balance,
        claim_counter: u128,
    }

    impl Default for RewardSettings {
        fn default() -> RewardSettings {
            RewardSettings {
                reward_on: u8::default(),
                reward_root_set: u8::default(),
                reward_root: AccountId::from([0x0; 32]),
                reward_interval: u128::default(),
                reward_amount: Balance::default(),
                reward_balance: Balance::default(),
                reward_payouts: Balance::default(),
                claim_counter: u128::default(),
            }
        }
    }


    // EVENT DEFINITIONS >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    #[ink(event)]
    // Writes the new claim to the blockchain 
    pub struct ClaimMadeExpertise {
        #[ink(topic)]
        claimant: AccountId,
        #[ink(topic)]
        claim: Vec<u8>,
        #[ink(topic)]
        claim_id: Hash,
    }

    #[ink(event)]
    // Writes the new claim to the blockchain 
    pub struct ClaimMadeWorkHistory {
        #[ink(topic)]
        claimant: AccountId,
        #[ink(topic)]
        claim: Vec<u8>,
        #[ink(topic)]
        claim_id: Hash,
    }

    #[ink(event)]
    // Writes the new claim to the blockchain 
    pub struct ClaimMadeEducation {
        #[ink(topic)]
        claimant: AccountId,
        #[ink(topic)]
        claim: Vec<u8>,
        #[ink(topic)]
        claim_id: Hash,
    }

    #[ink(event)]
    // Writes the new claim to the blockchain 
    pub struct ClaimMadeGoodDeed {
        #[ink(topic)]
        claimant: AccountId,
        #[ink(topic)]
        claim: Vec<u8>,
        #[ink(topic)]
        claim_id: Hash,
    }

    #[ink(event)]
    // Writes the new claim to the blockchain 
    pub struct ClaimMadeIntellectualProperty {
        #[ink(topic)]
        claimant: AccountId,
        #[ink(topic)]
        claim: Vec<u8>,
        #[ink(topic)]
        claim_id: Hash,
    }

    #[ink(event)]
    // Writes the new endorsement to the blockchain 
    pub struct ClaimEndorsed {
        #[ink(topic)]
        claimant: AccountId,
        #[ink(topic)]
        claim_id: Hash,
        #[ink(topic)]
        endorser: AccountId
    }

    #[ink(event)]
    // Writes the new reward to the blockchain 
    pub struct AccountRewardedLifeAndWork {
        #[ink(topic)]
        claimant: AccountId,
        reward: Balance,
    }


    // ERROR DEFINITIONS >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    // Errors that can occur upon calling this contract...
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        // Returned if the new claim already exists.
        DuplicateClaim,
        // Returned if the endorsed claim does not exist.
        NonexistentClaim,
        // Returned if the caller has alredy endorsed this claim.
        DuplicateEndorsement,
        // Returned if the caller is not the owner of a claim.
        CallerNotOwner,
        // input data is too large
        DataTooLarge,
        // Caller doee not have permission
        PermissionDenied,
        // payout failed to root for reward program shut down
        PayoutFailed,
        // zero balance or not enough in the reward program
        ZeroBalance,
    }


    // CONTRACT LOGIC >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    // ACTUAL CONTRACT STORAGE >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
    #[ink(storage)]
    pub struct ContractStorage {
        claim_hashes: StorageVec<Hash>,
        claim_details: Mapping<Hash, Details>,
        account_claims_expertise: Mapping<AccountId, Claims>,
        account_claims_education: Mapping<AccountId, Claims>,
        account_claims_workhistory: Mapping<AccountId, Claims>,
        account_claims_gooddeeds: Mapping<AccountId, Claims>,
        account_claims_intellectualproperty: Mapping<AccountId, Claims>,
        reward_root_set: u8,
        reward_root: AccountId,
        reward_interval: u128,
        reward_amount: Balance,
        reward_on: u8,
        reward_balance: Balance,
        reward_payouts: Balance,
        claim_counter: u128,
    }

    impl ContractStorage {
        
        // CONSTRUCTORS >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        // Constructors are implicitly payable when the contract is instantiated.

        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                claim_hashes: StorageVec::default(),
                claim_details: Mapping::default(),
                account_claims_expertise: Mapping::default(),
                account_claims_education: Mapping::default(),
                account_claims_workhistory: Mapping::default(),
                account_claims_gooddeeds: Mapping::default(),
                account_claims_intellectualproperty: Mapping::default(),
                reward_root_set: 0,
                reward_root: AccountId::from([0x0; 32]),
                reward_interval: 1000000,
                reward_amount: 0,
                reward_on: 0,
                reward_balance: 0,
                reward_payouts: 0,
                claim_counter: 0,
            }
        }


        // MESSGE FUNCTIONS THAT ALTER CONTRACT STORAGE >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        
        #[ink(message)]
        // 🟢 0 EXPERTISE - Updates the storage map and emits an event to register the claim on chain
        pub fn make_claim_expertise(&mut self, 
            keywords_or_description: Vec<u8>, url_link_to_see_more: Vec<u8>
        ) -> Result<(), Error> {

            // define the caller...
            let caller = Self::env().caller();
            // get the current set of claims for this account
            let mut currentclaims = self.account_claims_expertise.get(caller).unwrap_or_default();
            // if the caller has too many claims in this area send an error
            if currentclaims.claims.len() > 490 {
                return Err(Error::DataTooLarge)
            }
            else {
                // set up the data that will go into the claim_hash
                let claimant = Self::env().caller();
                let claim_contents = keywords_or_description.clone();

                // create the claim_hash by hashing the claimant and claim data
                let encodable = (claimant, claim_contents); // Implements `scale::Encode`
                let mut claim_hash_u8 = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
                ink::env::hash_encoded::<Sha2x256, _>(&encodable, &mut claim_hash_u8);
                let claim_hash: Hash = Hash::from(claim_hash_u8);

                // Check to make sure the claim is not a duplicate
                if self.claim_details.contains(claim_hash) {
                    // if TRUE, issue an error
                    return Err(Error::DuplicateClaim)
                }
                else {
                    // set the contract storage for this claim...
                    let new_details = Details {
                        claimtype: 3,
                        claimant: Self::env().caller(),
                        claim: keywords_or_description,
                        claim_id: claim_hash,
                        endorser_count: 0,
                        link: url_link_to_see_more,
                        show: true,
                        endorsers: vec![Self::env().caller()]
                    };

                    // add this claim to the claim_details map
                    if self.claim_details.try_insert(claim_hash, &new_details).is_err() {
                        return Err(Error::DataTooLarge);
                    }

                    // add this claim to the claim_hashes storage vector
                    self.claim_hashes.push(&claim_hash);
                    
                    // add this claim hash to the set of claims for this account
                    // add the claim hash to the Claims.claims vector of claim_id hashes
                    currentclaims.claims.push(claim_hash);

                    // update the account_claims mapping
                    self.account_claims_expertise.insert(caller, &currentclaims);

                    // Emit an event to register the claim to the chain
                    // make a clone of claim_meta 
                    let claim_meta_clone = new_details.claim.clone();
                    Self::env().emit_event(ClaimMadeExpertise {
                        claimant: Self::env().caller(),
                        claim: claim_meta_clone,
                        claim_id: claim_hash
                    });

                    // REWARD PROGRAM ACTIONS... update the claim_counter 
                    self.claim_counter = self.claim_counter.saturating_add(1);
                    // IF conditions are met THEN payout a reward
                    let min = self.reward_amount.saturating_add(10);
                    let payout: Balance = self.reward_amount;
                    if self.reward_on == 1 && self.reward_balance > payout && self.env().balance() > min
                    && self.claim_counter.checked_rem_euclid(self.reward_interval) == Some(0) {
                        // payout
                        if self.env().transfer(caller, payout).is_err() {
                            return Err(Error::PayoutFailed);
                        }
                        // update reward_balance
                        self.reward_balance = self.reward_balance.saturating_sub(payout);
                        // update reward_payouts
                        self.reward_payouts = self.reward_payouts.saturating_add(payout);
                        // emit an event to register the reward to the chain
                        Self::env().emit_event(AccountRewardedLifeAndWork {
                            claimant: caller,
                            reward: payout
                        });
                    }
                    // END REWARD PROGRAM ACTIONS

                }
            }
            
            Ok(())
        }


        #[ink(message)]
        // 🟢 1 WORK - Updates the storage map and emits an event to register the claim on chain
        pub fn make_claim_workhistory(&mut self, 
            keywords_or_description: Vec<u8>, url_link_to_see_more: Vec<u8>
        ) -> Result<(), Error> {
            // define the caller...
            let caller = Self::env().caller();
            // get the current set of claims for this account
            let mut currentclaims = self.account_claims_workhistory.get(caller).unwrap_or_default();

            // if they have too many claims in this cateogry, send an error
            if currentclaims.claims.len() > 490 {
                return Err(Error::DataTooLarge)
            }
            else {
                // set up the data that will go into the claim_hash
                let claimant = Self::env().caller();
                let claim_contents = keywords_or_description.clone();

                // create the claim_hash by hashing the claimant and claim data
                let encodable = (claimant, claim_contents); // Implements `scale::Encode`
                let mut claim_hash_u8 = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
                ink::env::hash_encoded::<Sha2x256, _>(&encodable, &mut claim_hash_u8);
                let claim_hash: Hash = Hash::from(claim_hash_u8);

                // Check to make sure the claim is not a duplicate
                if self.claim_details.contains(claim_hash) {
                    // if TRUE, issue an error
                    return Err(Error::DuplicateClaim)
                }
                // if FALSE...set the contract storage for this claim...
                
                // add this claim to the claim_details map
                let new_details = Details {
                    claimtype: 1,
                    claimant: Self::env().caller(),
                    claim: keywords_or_description,
                    claim_id: claim_hash,
                    endorser_count: 0,
                    link: url_link_to_see_more,
                    show: true,
                    endorsers: vec![Self::env().caller()]
                };

                if self.claim_details.try_insert(claim_hash, &new_details).is_err() {
                    return Err(Error::DataTooLarge);
                }

                // add this claim to the claim_hashes vector
                self.claim_hashes.push(&claim_hash);
                
                // add this claim hash to the set of claims for this account
                // add the claim hash to the Claims.claims vector of claim_id hashes
                currentclaims.claims.push(claim_hash);
                // update the account_claims mapping
                self.account_claims_workhistory.insert(caller, &currentclaims);

                // then emit an event to register the claim to the chain
                // make a clone of claim_meta 
                let claim_meta_clone = new_details.claim.clone();
                Self::env().emit_event(ClaimMadeWorkHistory {
                    claimant: Self::env().caller(),
                    claim: claim_meta_clone,
                    claim_id: claim_hash
                });

                // REWARD PROGRAM ACTIONS... update the claim_counter 
                self.claim_counter = self.claim_counter.saturating_add(1);
                // IF conditions are met THEN payout a reward
                let min = self.reward_amount.saturating_add(10);
                let payout: Balance = self.reward_amount;
                if self.reward_on == 1 && self.reward_balance > payout && self.env().balance() > min
                && self.claim_counter.checked_rem_euclid(self.reward_interval) == Some(0) {
                    // payout
                    if self.env().transfer(caller, payout).is_err() {
                        return Err(Error::PayoutFailed);
                    }
                    // update reward_balance
                    self.reward_balance = self.reward_balance.saturating_sub(payout);
                    // update reward_payouts
                    self.reward_payouts = self.reward_payouts.saturating_add(payout);
                    // emit an event to register the reward to the chain
                    Self::env().emit_event(AccountRewardedLifeAndWork {
                        claimant: caller,
                        reward: payout
                    });
                }
                // END REWARD PROGRAM ACTIONS

            }
            
            Ok(())
        }


        #[ink(message)]
        // 🟢 2 EDUCATION - Updates the storage map and emits an event to register the claim on chain
        pub fn make_claim_education(&mut self, 
            keywords_or_description: Vec<u8>, url_link_to_see_more: Vec<u8>
        ) -> Result<(), Error> {
            // define the caller...
            let caller = Self::env().caller();
            // get the current set of claims for this account
            let mut currentclaims = self.account_claims_education.get(caller).unwrap_or_default();

            // if they have too many claims in this cateogry, send an error
            if currentclaims.claims.len() > 490 {
                return Err(Error::DataTooLarge)
            }
            else {
                // set up the data that will go into the claim_hash
                let claimant = Self::env().caller();
                let claim_contents = keywords_or_description.clone();

                // create the claim_hash by hashing the claimant and claim data
                let encodable = (claimant, claim_contents); // Implements `scale::Encode`
                let mut claim_hash_u8 = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
                ink::env::hash_encoded::<Sha2x256, _>(&encodable, &mut claim_hash_u8);
                let claim_hash: Hash = Hash::from(claim_hash_u8);

                // Check to make sure the claim is not a duplicate
                if self.claim_details.contains(claim_hash) {
                    // if TRUE, issue an error
                    return Err(Error::DuplicateClaim)
                }
                // if FALSE...set the contract storage for this claim...
                
                // add this claim to the claim_details map
                let new_details = Details {
                    claimtype: 2,
                    claimant: Self::env().caller(),
                    claim: keywords_or_description,
                    claim_id: claim_hash,
                    endorser_count: 0,
                    link: url_link_to_see_more,
                    show: true,
                    endorsers: vec![Self::env().caller()]
                };
                
                if self.claim_details.try_insert(claim_hash, &new_details).is_err() {
                    return Err(Error::DataTooLarge);
                }

                // add this claim to the claim_hashes vector
                self.claim_hashes.push(&claim_hash);
                
                // add this claim hash to the set of claims for this account
                // add the claim hash to the Claims.claims vector of claim_id hashes
                currentclaims.claims.push(claim_hash);
                // update the account_claims mapping
                self.account_claims_education.insert(caller, &currentclaims);

                // then emit an event to register the claim to the chain
                // make a clone of claim_meta 
                let claim_meta_clone = new_details.claim.clone();
                Self::env().emit_event(ClaimMadeEducation {
                    claimant: Self::env().caller(),
                    claim: claim_meta_clone,
                    claim_id: claim_hash
                });

                // REWARD PROGRAM ACTIONS... update the claim_counter 
                self.claim_counter = self.claim_counter.saturating_add(1);
                // IF conditions are met THEN payout a reward
                let min = self.reward_amount.saturating_add(10);
                let payout: Balance = self.reward_amount;
                if self.reward_on == 1 && self.reward_balance > payout && self.env().balance() > min
                && self.claim_counter.checked_rem_euclid(self.reward_interval) == Some(0) {
                    // payout
                    if self.env().transfer(caller, payout).is_err() {
                        return Err(Error::PayoutFailed);
                    }
                    // update reward_balance
                    self.reward_balance = self.reward_balance.saturating_sub(payout);
                    // update reward_payouts
                    self.reward_payouts = self.reward_payouts.saturating_add(payout);
                    // emit an event to register the reward to the chain
                    Self::env().emit_event(AccountRewardedLifeAndWork {
                        claimant: caller,
                        reward: payout
                    });
                }
                // END REWARD PROGRAM ACTIONS
            }
            
            Ok(())
        }


        #[ink(message)]
        // 🟢 3 GOOD DEEDS - Updates the storage map and emits an event to register the claim on chain
        pub fn make_claim_gooddeed(&mut self, 
            keywords_or_description: Vec<u8>, url_link_to_see_more: Vec<u8>
        ) -> Result<(), Error> {
            // define the caller...
            let caller = Self::env().caller();
            // get the current set of claims for this account
            let mut currentclaims = self.account_claims_gooddeeds.get(caller).unwrap_or_default();

            // if they have too many claims in this cateogry, send an error
            if currentclaims.claims.len() > 490 {
                return Err(Error::DataTooLarge)
            }
            else {
                // set up the data that will go into the claim_hash
                let claimant = Self::env().caller();
                let claim_contents = keywords_or_description.clone();

                // create the claim_hash by hashing the claimant and claim data
                let encodable = (claimant, claim_contents); // Implements `scale::Encode`
                let mut claim_hash_u8 = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
                ink::env::hash_encoded::<Sha2x256, _>(&encodable, &mut claim_hash_u8);
                let claim_hash: Hash = Hash::from(claim_hash_u8);

                // Check to make sure the claim is not a duplicate
                if self.claim_details.contains(claim_hash) {
                    // if TRUE, issue an error
                    return Err(Error::DuplicateClaim)
                }
                // if FALSE...set the contract storage for this claim...
                
                // add this claim to the claim_details map
                let new_details = Details {
                    claimtype: 4,
                    claimant: Self::env().caller(),
                    claim: keywords_or_description,
                    claim_id: claim_hash,
                    endorser_count: 0,
                    link: url_link_to_see_more,
                    show: true,
                    endorsers: vec![Self::env().caller()]
                };
                
                if self.claim_details.try_insert(claim_hash, &new_details).is_err() {
                    return Err(Error::DataTooLarge);
                }

                // add this claim to the claim_hashes vector
                self.claim_hashes.push(&claim_hash);
                
                // add this claim hash to the set of claims for this account
                // add the claim hash to the Claims.claims vector of claim_id hashes
                currentclaims.claims.push(claim_hash);
                // update the account_claims mapping
                self.account_claims_gooddeeds.insert(caller, &currentclaims);

                // then emit an event to register the claim to the chain
                // make a clone of claim_meta 
                let claim_meta_clone = new_details.claim.clone();
                Self::env().emit_event(ClaimMadeGoodDeed {
                    claimant: Self::env().caller(),
                    claim: claim_meta_clone,
                    claim_id: claim_hash
                });

                // REWARD PROGRAM ACTIONS... update the claim_counter 
                self.claim_counter = self.claim_counter.saturating_add(1);
                // IF conditions are met THEN payout a reward
                let min = self.reward_amount.saturating_add(10);
                let payout: Balance = self.reward_amount;
                if self.reward_on == 1 && self.reward_balance > payout && self.env().balance() > min
                && self.claim_counter.checked_rem_euclid(self.reward_interval) == Some(0) {
                    // payout
                    if self.env().transfer(caller, payout).is_err() {
                        return Err(Error::PayoutFailed);
                    }
                    // update reward_balance
                    self.reward_balance = self.reward_balance.saturating_sub(payout);
                    // update reward_payouts
                    self.reward_payouts = self.reward_payouts.saturating_add(payout);
                    // emit an event to register the reward to the chain
                    Self::env().emit_event(AccountRewardedLifeAndWork {
                        claimant: caller,
                        reward: payout
                    });
                }
                // END REWARD PROGRAM ACTIONS
            }
            
            Ok(())
        }

        #[ink(message)]
        // 🟢 4 IP - Updates the storage map and emits an event to register the claim on chain
        pub fn make_claim_intellectualproperty(&mut self, 
            keywords_or_description: Vec<u8>, url_link_to_see_more: Vec<u8>, 
            hash_your_intellectual_property_file_here: Hash
        ) -> Result<(), Error> {
            // define the caller...
            let caller = Self::env().caller();
            // get the current set of claims for this account
            let mut currentclaims = self.account_claims_intellectualproperty.get(caller).unwrap_or_default();

            // if they have too many claims in this cateogry, send an error
            if currentclaims.claims.len() > 490 {
                return Err(Error::DataTooLarge)
            }
            else {
                let claim_hash = hash_your_intellectual_property_file_here;

                // Check to make sure the claim is not a duplicate
                if self.claim_details.contains(claim_hash) {
                    // if TRUE, issue an error
                    return Err(Error::DuplicateClaim)
                }
                // if FALSE...set the contract storage for this claim...
                
                // add this claim to the claim_details map
                let new_details = Details {
                    claimtype: 5,
                    claimant: Self::env().caller(),
                    claim: keywords_or_description,
                    claim_id: claim_hash,
                    endorser_count: 0,
                    link: url_link_to_see_more,
                    show: true,
                    endorsers: vec![Self::env().caller()]
                };
                
                if self.claim_details.try_insert(claim_hash, &new_details).is_err() {
                    return Err(Error::DataTooLarge);
                }

                // add this claim to the claim_hashes vector
                self.claim_hashes.push(&claim_hash);
                
                // add this claim hash to the set of claims for this account
                // add the claim hash to the Claims.claims vector of claim_id hashes
                currentclaims.claims.push(claim_hash);
                // update the account_claims mapping
                self.account_claims_intellectualproperty.insert(caller, &currentclaims);

                // then emit an event to register the claim to the chain
                // make a clone of claim_meta 
                let claim_meta_clone = new_details.claim.clone();
                Self::env().emit_event(ClaimMadeIntellectualProperty {
                    claimant: Self::env().caller(),
                    claim: claim_meta_clone,
                    claim_id: claim_hash
                });

                // REWARD PROGRAM ACTIONS... update the claim_counter 
                self.claim_counter = self.claim_counter.saturating_add(1);
                // IF conditions are met THEN payout a reward
                let min = self.reward_amount.saturating_add(10);
                let payout: Balance = self.reward_amount;
                if self.reward_on == 1 && self.reward_balance > payout && self.env().balance() > min
                && self.claim_counter.checked_rem_euclid(self.reward_interval) == Some(0) {
                    // payout
                    if self.env().transfer(caller, payout).is_err() {
                        return Err(Error::PayoutFailed);
                    }
                    // update reward_balance
                    self.reward_balance = self.reward_balance.saturating_sub(payout);
                    // update reward_payouts
                    self.reward_payouts = self.reward_payouts.saturating_add(payout);
                    // emit an event to register the reward to the chain
                    Self::env().emit_event(AccountRewardedLifeAndWork {
                        claimant: caller,
                        reward: payout
                    });
                }
                // END REWARD PROGRAM ACTIONS
            }
            
            Ok(())
        }


        #[ink(message)]
        // 🟢 5 ENDORSE - Updates the storage map and emits an event to register the endorsement on chain 
        pub fn endorse_claim(&mut self, claim_id: Hash
        ) -> Result<(), Error> {

            // Does the claimhash exist in the mappings? If TRUE then proceed...
            if self.claim_details.contains(claim_id) {

                // Get the contract caller's Account ID
                let caller = Self::env().caller();
                // Get the list of endorsers for this claimID from the claim_details
                let mut current_details = self.claim_details.get(claim_id).unwrap_or_default();
                // Is the caller is already in the endorsers list for this claim?... 
                if current_details.endorsers.contains(&caller) {
                    // If TRUE, return an Error... DuplicateEndorsement
                    Err(Error::DuplicateEndorsement)
                } 
                else {
                    // If the caller is NOT already an endorser...
                    // if there are less than 490 endorsers, add this endorser to the vector
                    if current_details.endorsers.len() < 490 {

                        current_details.endorsers.push(caller);
                        // update the endorser count
                        let new_endorser_count = current_details.endorser_count.saturating_add(1);

                        // Update the details in storage for this claim
                        let updated_details: Details = Details {
                            claimtype: current_details.claimtype,
                            claimant: current_details.claimant,
                            claim: current_details.claim,
                            claim_id: claim_id,
                            endorser_count: new_endorser_count,
                            link: current_details.link,
                            show: current_details.show,
                            endorsers: current_details.endorsers
                        };

                        // Update the claim_map
                        if self.claim_details.try_insert(claim_id, &updated_details).is_err() {
                            return Err(Error::DataTooLarge);
                        }
                    }

                    // (2) emit an event to register the endorsement to the chain
                    // the event will register regardless of if we no longer have room
                    // for this endorsement in the contract storage
                    Self::env().emit_event(ClaimEndorsed {
                        claimant: current_details.claimant,
                        claim_id: claim_id,
                        endorser: Self::env().caller()
                    });
                    Ok(())
                }
            }
            else {
                // if the claimhash does not exist ...Error: Nonexistent Claim
                Err(Error::NonexistentClaim)
            }

        }


        // 🟢 6 SHOW/HIDE - Show or hide a given claimID hash IF the caller is the owner
        #[ink(message)]
        pub fn show_or_hide_claim(&mut self, claim_id: Hash, set_to_show: bool
        ) -> Result<(), Error> {
            
            // first, get the details and make sure the caller owns this claimID
            let caller = Self::env().caller();
            let details = self.claim_details.get(claim_id).unwrap_or_default();

            if details.claimant == caller {
                // set the show boolean to set_to_show
                let updated_details: Details = Details {
                    claimtype: details.claimtype,
                    claimant: details.claimant,
                    claim: details.claim,
                    claim_id: claim_id,
                    endorser_count: details.endorser_count,
                    link: details.link,
                    show: set_to_show,
                    endorsers: details.endorsers
                };
                
                // Update the claim_map
                if self.claim_details.try_insert(claim_id, &updated_details).is_err() {
                    return Err(Error::DataTooLarge);
                }    

                Ok(())
            }
            else {
                // send an error that this caller is not the claimant
                Err(Error::CallerNotOwner)
            }
        }


        // MESSAGE FUNCTIONS THAT RETRIEVE DATA FROM STORAGE  >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

        // 🟢 7 GET RESUME - Given an AccountID, return the detailed info for EVERY claim made by that account
        #[ink(message)]
        pub fn get_resume(&self, owner: AccountId) -> Vec<Details> {
            // given the AccountID, get the set of each type of claimIDs
            let idvec_work = self.account_claims_workhistory.get(owner).unwrap_or_default().claims;
            let idvec_ed = self.account_claims_education.get(owner).unwrap_or_default().claims;
            let idvec_expert = self.account_claims_expertise.get(owner).unwrap_or_default().claims;
            let idvec_deeds = self.account_claims_gooddeeds.get(owner).unwrap_or_default().claims;
            let idvec_ip = self.account_claims_intellectualproperty.get(owner).unwrap_or_default().claims;
            let mut resume: Vec<Details> = Vec::new();

            // Iterate over each idvec: for each claimID...

            for claimidhash in idvec_work.iter() {
                // get the details
                let resumeitem = self.claim_details.get(claimidhash).unwrap_or_default();
                // then add that resume item to the resume vector
                resume.push(resumeitem);
            }

            for claimidhash in idvec_ed.iter() {
                // get the details
                let resumeitem = self.claim_details.get(claimidhash).unwrap_or_default();
                // then add that resume item to the resume vector
                resume.push(resumeitem);
            }

            for claimidhash in idvec_expert.iter() {
                // get the details
                let resumeitem = self.claim_details.get(claimidhash).unwrap_or_default();
                // then add that resume item to the resume vector
                resume.push(resumeitem);
            }

            for claimidhash in idvec_deeds.iter() {
                // get the details
                let resumeitem = self.claim_details.get(claimidhash).unwrap_or_default();
                // then add that resume item to the resume vector
                resume.push(resumeitem);
            }
            
            for claimidhash in idvec_ip.iter() {
                // get the details
                let resumeitem = self.claim_details.get(claimidhash).unwrap_or_default();
                // then add that resume item to the resume vector
                resume.push(resumeitem);
            }
            
            // Return the vector of ResumeItem structs
            resume

        }


        // 🟢 8 Return the ENTIRE DETAILS struct for one claimID hash
        #[ink(message)]
        pub fn get_full_details(&self, claim_id: Hash) -> Details {
            let details = self.claim_details.get(claim_id).unwrap_or_default();
            details
        }

        // 🟢 9 GET ENDORSERS - for a given claim_id hash, get the ENDORSERS for that claim
        #[ink(message)]
        pub fn get_endorsers(&self, claim_id: Hash) -> Vec<AccountId> {
            let details = self.claim_details.get(claim_id).unwrap_or_default();
            details.endorsers
        }

        /*  🟢 10 KEYWORD SEARCH ...
        FOR A GIVEN KEYWORD OR KEY PHRASE, GET THE CLAIMS WHOSE CLAIM KEYWORDS
        INCLUDE THAT ENTIRE WORD OR PHRASE.
        Notes: You cannot iterate on a mapping BUT you can iterate on a VECTOR so we have 
        an additional storage line that looks like claim_hashes: StorageVec<Hash> where we 
        keep a running vector of all the claim id hashes and iterate over that instead.
        We have to convert the u8 vectors to strings so that we can use the contains()
        function on the whole set of u8 items in the keywords rather than just one letter. 
        */
        #[ink(message)]
        pub fn get_matching_claims(&self, keywords: Vec<u8>) -> Vec<Details> {
            // get a string for your keywords
            let searchstring = String::from_utf8(keywords).unwrap_or_default();
            // set up your results vector
            let mut matching_resume_items: Vec<Details> = Vec::new();

            // iterate over the claim_hashes vector to find claims that match
            if self.claim_hashes.len() > 0 {
                for i in 0..self.claim_hashes.len() {
                    let claimidhash = self.claim_hashes.get(i).unwrap_or_default();
                    let resumeitem = self.claim_details.get(claimidhash).unwrap_or_default();
                    let claimvecu8 = resumeitem.claim.clone();
                    let claimstring = String::from_utf8(claimvecu8).unwrap_or_default();

                    // if the keywords are in the claim keyword set...
                    if claimstring.contains(&searchstring) {
                        // add the details to the results vector
                        matching_resume_items.push(resumeitem);
                    }
                }
            }

            matching_resume_items

        }


        // REWARD PROGRAM MESSAGES >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        
        // 🟢 11 Verify Account - returns the total number of claims in a given resume
        #[ink(message)]
        pub fn verify_account(&self, owner: AccountId) -> (u32, u32, u32, u32, u32) {
            // given the AccountID, get the set of each type of claimIDs
            let work = self.account_claims_workhistory.get(owner).unwrap_or_default().claims.len();
            let ed = self.account_claims_education.get(owner).unwrap_or_default().claims.len();
            let expert = self.account_claims_expertise.get(owner).unwrap_or_default().claims.len();
            let deeds = self.account_claims_gooddeeds.get(owner).unwrap_or_default().claims.len();
            let ip = self.account_claims_intellectualproperty.get(owner).unwrap_or_default().claims.len();

            let result = (work.try_into().unwrap(), ed.try_into().unwrap(), expert.try_into().unwrap(), deeds.try_into().unwrap(), ip.try_into().unwrap());
            result
        }


        // 🟢 12 Rewards - Set Or Update Reward Root Account [RESTRICTED: ROOT]
        #[ink(message)]
        pub fn set_reward_roots(&mut self, newroot: AccountId) -> Result<(), Error> {
            let caller = Self::env().caller();
            // if the root is already set, send an error
            if self.reward_root_set != 1 || self.reward_root == caller {
                // proceed - set the roots and update the storage
                self.reward_root = newroot;
                self.reward_root_set = 1;
            }
            else {
                // error PermissionDenied
                return Err(Error::PermissionDenied)
            }

            Ok(())
        }


        // 🟢 13 Rewards - Set/Update Reward Interval and Ammount [RESTRICTED: ROOT]
        // Reward coin will be given to the account that makes the Xth claim in the system
        #[ink(message)]
        pub fn set_reward(&mut self, on: u8, interval: u128, amount: Balance) -> Result<(), Error> {
            let caller = Self::env().caller();
            if self.reward_root == caller {
                // proceed to set the reward program paramteters
                self.reward_on = on;
                self.reward_interval = interval;
                self.reward_amount = amount;
            }
            else {
                // error PermissionDenied
                return Err(Error::PermissionDenied)
            }
            
            Ok(())
        }

        // 🟢 14 ADD COIN TO REWARD ACCOUNT [RESTRICTED: ROOT]
        #[ink(message, payable)]
        pub fn add_reward_balance(&mut self) -> Result<(), Error> {
            let caller = Self::env().caller();
            if self.reward_root == caller {
                // add the paid in value to the reward_balance
                let staked: Balance = self.env().transferred_value();
                let newbalance: Balance = self.reward_balance.saturating_add(staked);
                self.reward_balance = newbalance;
            }
            else {
                // error PermissionDenied
                return Err(Error::PermissionDenied)
            }
            
            Ok(())
        }


        // 🟢 15 RETREIVE COIN FROM REWARD ACCOUNT [RESTRICTED: ROOT]
        // turns reward program off and returns funds to the root
        #[ink(message)]
        pub fn shut_down_reward(&mut self) -> Result<(), Error> {
            let caller = Self::env().caller();
            if self.reward_root == caller {
                // set the reward program to off
                self.reward_on = 0;
                // refund the coin to the reward root
                // Check that there is a nonzero balance on the contract > existential deposit
                if self.env().balance() > 10 && self.reward_balance > 0 {
                    // pay the root the reward_balance minus 10
                    let payout: Balance = self.reward_balance.saturating_sub(10);
                    if self.env().transfer(caller, payout).is_err() {
                        return Err(Error::PayoutFailed);
                    }
                }
                // if the balance is < 10, Error (ZeroBalance)
                else {
                    return Err(Error::ZeroBalance);
                }
            }
            else {
                // error PermissionDenied
                return Err(Error::PermissionDenied)
            }
            
            Ok(())
        }


        // 🟢 16 GET CURRENT REWARD BALANCE AND SETTINGS [RESTRICTED: ROOT]
        #[ink(message)]
        pub fn get_reward_settings(&self) -> RewardSettings {
            let caller = Self::env().caller();
            let mut results = RewardSettings::default();
            if self.reward_root == caller {
                let settings = RewardSettings {
                    reward_on: self.reward_on,
                    reward_root_set: self.reward_root_set,
                    reward_root: self.reward_root,
                    reward_interval: self.reward_interval,
                    reward_amount: self.reward_amount,
                    reward_balance: self.reward_balance,
                    reward_payouts: self.reward_payouts,
                    claim_counter: self.claim_counter,
                };
                results = settings;
            }

            results
        }




    }
    // END OF CONTRACT LOGIC

}
