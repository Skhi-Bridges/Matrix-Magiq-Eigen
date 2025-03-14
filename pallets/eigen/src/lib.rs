//! Eigenlayer Security Implementation for Matrix-Magiq ecosystem
//!
//! This pallet implements security measures for the entire Matrix-Magiq ecosystem
//! through validator coordination and restaking mechanisms.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;
    
    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        
        /// Validator ID type
        type ValidatorId: Member + Parameter + MaxEncodedLen + Copy;
        
        /// Stake currency
        type Currency: Currency<Self::AccountId>;
        
        /// Max validators per set
        #[pallet::constant]
        type MaxValidatorsPerSet: Get<u32>;
        
        /// Minimum stake amount
        #[pallet::constant]
        type MinStakeAmount: Get<BalanceOf<T>>;
        
        /// Session duration in blocks
        #[pallet::constant]
        type SessionDuration: Get<T::BlockNumber>;
    }

    /// Active validator sets for each parachain
    #[pallet::storage]
    pub type ValidatorSets<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // Parachain ID
        BoundedVec<Validator<T>, T::MaxValidatorsPerSet>,
        ValueQuery,
    >;
    
    /// Validator profiles
    #[pallet::storage]
    pub type Validators<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::ValidatorId,
        ValidatorProfile<T>,
    >;
    
    /// Stakes by account
    #[pallet::storage]
    pub type Stakes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId, // Staker
        Blake2_128Concat,
        T::ValidatorId, // Validator
        StakeInfo<T>,
    >;
    
    /// Total stake per validator
    #[pallet::storage]
    pub type TotalStake<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::ValidatorId,
        BalanceOf<T>,
        ValueQuery,
    >;
    
    /// Current session
    #[pallet::storage]
    pub type CurrentSession<T: Config> = StorageValue<_, SessionInfo<T>>;
    
    /// Validator representation
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct Validator<T: Config> {
        /// Validator ID
        pub id: T::ValidatorId,
        /// Validator account
        pub account: T::AccountId,
        /// Total stake amount
        pub total_stake: BalanceOf<T>,
        /// Validator status
        pub status: ValidatorStatus,
    }
    
    /// Validator profile
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ValidatorProfile<T: Config> {
        /// Validator ID
        pub id: T::ValidatorId,
        /// Validator account
        pub account: T::AccountId,
        /// Commission rate in basis points (1/10000)
        pub commission_rate: u16,
        /// Parachains this validator is active on
        pub active_parachains: BoundedVec<u32, T::MaxValidatorsPerSet>,
        /// Validator performance metrics
        pub performance: ValidatorPerformance,
        /// Validator status
        pub status: ValidatorStatus,
        /// Joined at block
        pub joined_at: T::BlockNumber,
        /// Last updated at block
        pub last_updated: T::BlockNumber,
    }
    
    /// Validator status
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum ValidatorStatus {
        /// Active validator
        Active,
        /// Pending validator
        Pending,
        /// Jailed validator
        Jailed,
        /// Exited validator
        Exited,
    }
    
    /// Validator performance metrics
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ValidatorPerformance {
        /// Uptime percentage (0-10000, representing 0-100.00%)
        pub uptime: u16,
        /// Blocks proposed
        pub blocks_proposed: u64,
        /// Blocks finalized
        pub blocks_finalized: u64,
        /// Slashes received
        pub slashes: u32,
        /// Performance score (0-10000, representing 0-100.00%)
        pub score: u16,
    }
    
    /// Stake information
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct StakeInfo<T: Config> {
        /// Staker account
        pub staker: T::AccountId,
        /// Validator ID
        pub validator: T::ValidatorId,
        /// Stake amount
        pub amount: BalanceOf<T>,
        /// Stake status
        pub status: StakeStatus,
        /// Staked at block
        pub staked_at: T::BlockNumber,
        /// Unlocked at block (if applicable)
        pub unlocked_at: Option<T::BlockNumber>,
    }
    
    /// Stake status
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum StakeStatus {
        /// Active stake
        Active,
        /// Unstaking (in progress)
        Unstaking,
        /// Withdrawn
        Withdrawn,
        /// Slashed
        Slashed,
    }
    
    /// Session information
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct SessionInfo<T: Config> {
        /// Session index
        pub index: u32,
        /// Start block
        pub start: T::BlockNumber,
        /// End block
        pub end: T::BlockNumber,
    }
    
    /// Alias for balance type
    pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    
    /// Currency trait
    pub trait Currency<AccountId> {
        /// Balance type
        type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;
        
        /// Get free balance
        fn free_balance(who: &AccountId) -> Self::Balance;
        
        /// Transfer balance
        fn transfer(
            source: &AccountId,
            dest: &AccountId,
            value: Self::Balance,
            existence_requirement: ExistenceRequirement,
        ) -> DispatchResult;
    }
    
    /// Existence requirement for currency transfers
    pub enum ExistenceRequirement {
        /// Keep alive
        KeepAlive,
        /// Allow death
        AllowDeath,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A validator was registered
        ValidatorRegistered {
            validator_id: T::ValidatorId,
            account: T::AccountId,
        },
        
        /// Stake was added to a validator
        StakeAdded {
            staker: T::AccountId,
            validator: T::ValidatorId,
            amount: BalanceOf<T>,
        },
        
        /// Stake was withdrawn from a validator
        StakeWithdrawn {
            staker: T::AccountId,
            validator: T::ValidatorId,
            amount: BalanceOf<T>,
        },
        
        /// A validator was slashed
        ValidatorSlashed {
            validator: T::ValidatorId,
            amount: BalanceOf<T>,
            reason: Vec<u8>,
        },
        
        /// A new session started
        NewSession {
            session_index: u32,
            start: T::BlockNumber,
            end: T::BlockNumber,
        },
        
        /// Validator set was updated for a parachain
        ValidatorSetUpdated {
            parachain_id: u32,
            validator_count: u32,
        },
    }
    
    #[pallet::error]
    pub enum Error<T> {
        /// Validator already exists
        ValidatorAlreadyExists,
        
        /// Validator not found
        ValidatorNotFound,
        
        /// Insufficient balance for staking
        InsufficientBalance,
        
        /// Stake amount below minimum
        StakeBelowMinimum,
        
        /// No stake found
        NoStakeFound,
        
        /// Unstaking in progress
        UnstakingInProgress,
        
        /// Validator is jailed
        ValidatorJailed,
        
        /// Too many validators
        TooManyValidators,
        
        /// Invalid commission rate
        InvalidCommissionRate,
        
        /// Invalid parachain ID
        InvalidParachainId,
    }

    // Implementation code will be added in separate files
}
