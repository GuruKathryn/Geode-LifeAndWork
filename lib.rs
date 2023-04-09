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

#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod life_and_work {

    use ink::prelude::vec::Vec;
    use ink::prelude::vec;
    use ink::prelude::string::String;
    use ink::storage::Mapping;
    use ink::env::hash::{Sha2x256, HashOutput};


    // PRELIMINARY DATA STRUCTURES >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(
            ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo,
            Debug,
            PartialEq,
            Eq
        )
    )]
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
            let default_addy = "000000000000000000000000000000000000000000000000";
            let default_addy_id32: AccountId = default_addy.as_bytes().try_into().unwrap();
            Details {
                claimtype: 0,
                claimant: default_addy_id32,
                claim: <Vec<u8>>::default(),
                claim_id: Hash::default(),
                endorser_count: 0,
                link: <Vec<u8>>::default(),
                show: true,
                endorsers: <Vec<AccountId>>::default(),
            }
        }
    }
   

    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(
            ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo,
            Debug,
            PartialEq,
            Eq
        )
    )]
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


    // ACTUAL CONTRACT STORAGE >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
    // #[derive(Default, scale::Decode, scale::Encode)]
    #[ink(storage)]
    pub struct ContractStorage {
        claim_hashes: Vec<Hash>,
        claim_details: Mapping<Hash, Details>,
        account_claims_expertise: Mapping<AccountId, Claims>,
        account_claims_education: Mapping<AccountId, Claims>,
        account_claims_workhistory: Mapping<AccountId, Claims>,
        account_claims_gooddeeds: Mapping<AccountId, Claims>,
        account_claims_intellectualproperty: Mapping<AccountId, Claims>
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
    }


    // CONTRACT LOGIC >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    impl ContractStorage {
        
        // CONSTRUCTORS >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        // Constructors are implicitly payable when the contract is instantiated.

        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                claim_hashes: <Vec<Hash>>::default(),
                claim_details: Mapping::default(),
                account_claims_expertise: Mapping::default(),
                account_claims_education: Mapping::default(),
                account_claims_workhistory: Mapping::default(),
                account_claims_gooddeeds: Mapping::default(),
                account_claims_intellectualproperty: Mapping::default()
            }
        }


        // MESSGE FUNCTIONS THAT ALTER CONTRACT STORAGE >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        
        #[ink(message)]
        // Updates the storage map and emits an event to register the claim on chain
        pub fn make_claim_expertise(&mut self, 
            keywords_or_description: Vec<u8>, url_link_to_see_more: Vec<u8>
        ) -> Result<(), Error> {
            
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
                claimtype: 3,
                claimant: Self::env().caller(),
                claim: keywords_or_description,
                claim_id: claim_hash,
                endorser_count: 0,
                link: url_link_to_see_more,
                show: true,
                endorsers: vec![Self::env().caller()]
            };
            self.claim_details.insert(&claim_hash, &new_details);

            // add this claim to the claim_hashes vector
            self.claim_hashes.push(claim_hash);
            
            // add this claim hash to the set of claims for this account
            // define the caller...
            let caller = Self::env().caller();
            // get the current set of claims for this account
            let mut current_claims = self.account_claims_expertise.get(&caller).unwrap_or_default();
            // add the claim hash to the Claims.claims vector of claim_id hashes
            current_claims.claims.push(claim_hash);
            // update the account_claims mapping
            self.account_claims_expertise.insert(&caller, &current_claims);

            // Emit an event to register the claim to the chain
            // make a clone of claim_meta 
            let claim_meta_clone = new_details.claim.clone();
            Self::env().emit_event(ClaimMadeExpertise {
                claimant: Self::env().caller(),
                claim: claim_meta_clone,
                claim_id: claim_hash
            });
            
            Ok(())
        }

        #[ink(message)]
        // Updates the storage map and emits an event to register the claim on chain
        pub fn make_claim_workhistory(&mut self, 
            keywords_or_description: Vec<u8>, url_link_to_see_more: Vec<u8>
        ) -> Result<(), Error> {

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
            self.claim_details.insert(&claim_hash, &new_details);

            // add this claim to the claim_hashes vector
            self.claim_hashes.push(claim_hash);
            
            // add this claim hash to the set of claims for this account
            // define the caller...
            let caller = Self::env().caller();
            // get the current set of claims for this account
            let mut current_claims = self.account_claims_workhistory.get(&caller).unwrap_or_default();
            // add the claim hash to the Claims.claims vector of claim_id hashes
            current_claims.claims.push(claim_hash);
            // update the account_claims mapping
            self.account_claims_workhistory.insert(&caller, &current_claims);

            // then emit an event to register the claim to the chain
            // make a clone of claim_meta 
            let claim_meta_clone = new_details.claim.clone();
            Self::env().emit_event(ClaimMadeWorkHistory {
                claimant: Self::env().caller(),
                claim: claim_meta_clone,
                claim_id: claim_hash
            });
            
            Ok(())
        }


        #[ink(message)]
        // Updates the storage map and emits an event to register the claim on chain
        pub fn make_claim_education(&mut self, 
            keywords_or_description: Vec<u8>, url_link_to_see_more: Vec<u8>
        ) -> Result<(), Error> {

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
            self.claim_details.insert(&claim_hash, &new_details);

            // add this claim to the claim_hashes vector
            self.claim_hashes.push(claim_hash);
            
            // add this claim hash to the set of claims for this account
            // define the caller...
            let caller = Self::env().caller();
            // get the current set of claims for this account
            let mut current_claims = self.account_claims_education.get(&caller).unwrap_or_default();
            // add the claim hash to the Claims.claims vector of claim_id hashes
            current_claims.claims.push(claim_hash);
            // update the account_claims mapping
            self.account_claims_education.insert(&caller, &current_claims);

            // then emit an event to register the claim to the chain
            // make a clone of claim_meta 
            let claim_meta_clone = new_details.claim.clone();
            Self::env().emit_event(ClaimMadeEducation {
                claimant: Self::env().caller(),
                claim: claim_meta_clone,
                claim_id: claim_hash
            });
            
            Ok(())
        }


        #[ink(message)]
        // Updates the storage map and emits an event to register the claim on chain
        pub fn make_claim_gooddeed(&mut self, 
            keywords_or_description: Vec<u8>, url_link_to_see_more: Vec<u8>
        ) -> Result<(), Error> {

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
            self.claim_details.insert(&claim_hash, &new_details);

            // add this claim to the claim_hashes vector
            self.claim_hashes.push(claim_hash);
            
            // add this claim hash to the set of claims for this account
            // define the caller...
            let caller = Self::env().caller();
            // get the current set of claims for this account
            let mut current_claims = self.account_claims_gooddeeds.get(&caller).unwrap_or_default();
            // add the claim hash to the Claims.claims vector of claim_id hashes
            current_claims.claims.push(claim_hash);
            // update the account_claims mapping
            self.account_claims_gooddeeds.insert(&caller, &current_claims);

            // then emit an event to register the claim to the chain
            // make a clone of claim_meta 
            let claim_meta_clone = new_details.claim.clone();
            Self::env().emit_event(ClaimMadeGoodDeed {
                claimant: Self::env().caller(),
                claim: claim_meta_clone,
                claim_id: claim_hash
            });
            
            Ok(())
        }

        #[ink(message)]
        // Updates the storage map and emits an event to register the claim on chain
        pub fn make_claim_intellectualproperty(&mut self, 
            keywords_or_description: Vec<u8>, url_link_to_see_more: Vec<u8>, 
            hash_your_intellectual_property_file_here: Hash
        ) -> Result<(), Error> {
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
            self.claim_details.insert(&claim_hash, &new_details);

            // add this claim to the claim_hashes vector
            self.claim_hashes.push(claim_hash);
            
            // add this claim hash to the set of claims for this account
            // define the caller...
            let caller = Self::env().caller();
            // get the current set of claims for this account
            let mut current_claims = self.account_claims_intellectualproperty.get(&caller).unwrap_or_default();
            // add the claim hash to the Claims.claims vector of claim_id hashes
            current_claims.claims.push(claim_hash);
            // update the account_claims mapping
            self.account_claims_intellectualproperty.insert(&caller, &current_claims);

            // then emit an event to register the claim to the chain
            // make a clone of claim_meta 
            let claim_meta_clone = new_details.claim.clone();
            Self::env().emit_event(ClaimMadeIntellectualProperty {
                claimant: Self::env().caller(),
                claim: claim_meta_clone,
                claim_id: claim_hash
            });
            
            Ok(())
        }


        #[ink(message)]
        // Updates the storage map and emits an event to register the endorsement on chain 
        pub fn endorse_claim(&mut self, claim_id: Hash
        ) -> Result<(), Error> {
            let claim_hash = claim_id;

            // Does the claimhash exists in the mappings? If TRUE then proceed...
            if self.claim_details.contains(claim_hash) {

                // Get the contract caller's Account ID
                let caller = Self::env().caller();
                // Get the list of endorsers for this claimID from the claim_details
                let mut current_details = self.claim_details.get(&claim_hash).unwrap_or_default();
                // Is the caller is already in the endorsers list for this claim?... 
                if current_details.endorsers.contains(&caller) {
                    // If TRUE, return an Error... DuplicateEndorsement
                    return Err(Error::DuplicateEndorsement)
                } 

                else {
                    // If the caller is NOT already an endorser...

                    // Add this endorser to the vector of endorsing accounts
                    current_details.endorsers.push(caller);
                    // update the endorser count
                    let new_endorser_count = current_details.endorser_count + 1;

                    // Update the details in storage for this claim
                    let updated_details: Details = Details {
                        claimtype: current_details.claimtype,
                        claimant: current_details.claimant,
                        claim: current_details.claim,
                        claim_id: claim_hash,
                        endorser_count: new_endorser_count,
                        link: current_details.link,
                        show: current_details.show,
                        endorsers: current_details.endorsers
                    };

                    // Update the claim_map
                    self.claim_details.insert(&claim_hash, &updated_details);

                    // (2) emit an event to register the endorsement to the chain
                    Self::env().emit_event(ClaimEndorsed {
                        claimant: current_details.claimant,
                        claim_id: claim_hash,
                        endorser: Self::env().caller()
                    });
                    Ok(())
                }
            }

            else {
                // if the claimhash does not exist ...Error: Nonexistent Claim
                return Err(Error::NonexistentClaim);
            }

        }


        // Show or hide a given claimID hash IF the caller is the owner
        #[ink(message)]
        pub fn show_or_hide_claim(&mut self, claim_id: Hash, set_to_show: bool
        ) -> Result<(), Error> {
            let claim_hash = claim_id;
            // first, get the details and make sure the caller owns this claimID
            let caller = Self::env().caller();
            let details = self.claim_details.get(&claim_hash).unwrap_or_default();

            if details.claimant == caller {
                // set the show boolean to set_to_show
                let updated_details: Details = Details {
                    claimtype: details.claimtype,
                    claimant: details.claimant,
                    claim: details.claim,
                    claim_id: claim_hash,
                    endorser_count: details.endorser_count,
                    link: details.link,
                    show: set_to_show,
                    endorsers: details.endorsers
                };
                // Update the claim_map
                self.claim_details.insert(&claim_hash, &updated_details);

                Ok(())
            }
            else {
                // send an error that this caller is not the claimant
                return Err(Error::CallerNotOwner);
            }
        }


        // MESSAGE FUNCTIONS THAT RETRIEVE DATA FROM STORAGE  >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

        // Given an AccountID, return the detailed info for EVERY claim made by that account
        #[ink(message)]
        pub fn get_resume(&self, owner: AccountId) -> Vec<Details> {
            // given the AccountID, get the set of each type of claimIDs
            let idvec_work = self.account_claims_workhistory.get(&owner).unwrap_or_default().claims;
            let idvec_ed = self.account_claims_education.get(&owner).unwrap_or_default().claims;
            let idvec_expert = self.account_claims_expertise.get(&owner).unwrap_or_default().claims;
            let idvec_deeds = self.account_claims_gooddeeds.get(&owner).unwrap_or_default().claims;
            let idvec_ip = self.account_claims_intellectualproperty.get(&owner).unwrap_or_default().claims;
            let mut resume: Vec<Details> = Vec::new();

            // Iterate over each idvec: for each claimID...

            for claimidhash in idvec_work.iter() {
                // get the details
                let resumeitem = self.claim_details.get(&claimidhash).unwrap_or_default();
                // then add that resume item to the resume vector
                resume.push(resumeitem);
            }

            for claimidhash in idvec_ed.iter() {
                // get the details
                let resumeitem = self.claim_details.get(&claimidhash).unwrap_or_default();
                // then add that resume item to the resume vector
                resume.push(resumeitem);
            }

            for claimidhash in idvec_expert.iter() {
                // get the details
                let resumeitem = self.claim_details.get(&claimidhash).unwrap_or_default();
                // then add that resume item to the resume vector
                resume.push(resumeitem);
            }

            for claimidhash in idvec_deeds.iter() {
                // get the details
                let resumeitem = self.claim_details.get(&claimidhash).unwrap_or_default();
                // then add that resume item to the resume vector
                resume.push(resumeitem);
            }
            
            for claimidhash in idvec_ip.iter() {
                // get the details
                let resumeitem = self.claim_details.get(&claimidhash).unwrap_or_default();
                // then add that resume item to the resume vector
                resume.push(resumeitem);
            }
            
            // Return the vector of ResumeItem structs
            resume

        }


        // Return the ENTIRE DETAILS struct for one claimID hash
        #[ink(message)]
        pub fn get_full_details(&self, claim_id: Hash) -> Details {
            let details = self.claim_details.get(&claim_id).unwrap_or_default();
            details
        }

        // for a given claim_id hash, get the ENDORSERS for that claim
        #[ink(message)]
        pub fn get_endorsers(&self, claim_id: Hash) -> Vec<AccountId> {
            let details = self.claim_details.get(&claim_id).unwrap_or_default();
            details.endorsers
        }

        /*  
        FOR A GIVEN KEYWORD OR KEY PHRASE, GET THE CLAIMS WHOSE CLAIM KEYWORDS
        INCLUDE THAT ENTIRE WORD OR PHRASE.
        Notes: You cannot iterate on a mapping BUT you can iterate on a VECTOR so we have 
        an additional storage line that looks like claim_hashes: Vec<Hash> where we 
        keep a running vector of all the claim id hashes and iterate over that instead.
        We have to convert the u8 vectors to strings so that we can use the contains()
        function on the whole set of u8 items in the keywords rather than just one letter. 
        */
        #[ink(message)]
        pub fn get_matching_claims(&self, keywords: Vec<u8>) -> Vec<Details> {
            // set up your results vector
            let mut matching_claims: Vec<Hash> = Vec::new(); 
            // iterate over the claim_hashes vector to find claims that match
            for claimidhash in self.claim_hashes.iter() {
                // if the keywords are in the claim keyword set...
                let claimvecu8 = self.claim_details.get(&claimidhash).unwrap_or_default().claim;
                let claimstring = String::from_utf8(claimvecu8).unwrap_or_default();
                let keywordsvecu8 = keywords.clone();
                let searchstring = String::from_utf8(keywordsvecu8).unwrap_or_default(); 
                if claimstring.contains(&searchstring) {
                    // add the claimid hash to the results vector
                    matching_claims.push(*claimidhash);
                }
            }
            // for each claimID in matching_claims, get the details
            // and add it to a vector of resume item details
            let mut matching_resume_items: Vec<Details> = Vec::new();

            // Iterate over each claimID: 
            for claimidhash in matching_claims.iter() {
                // get the details
                let resumeitem = self.claim_details.get(&claimidhash).unwrap_or_default();
                // then add that item to the results vector
                matching_resume_items.push(resumeitem);
            }

            matching_resume_items

        }

    }
    // END OF CONTRACT LOGIC

}
