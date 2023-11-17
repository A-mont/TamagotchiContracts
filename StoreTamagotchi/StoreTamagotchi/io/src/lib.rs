#![no_std]
use gstd::{ prelude::*, ActorId};
use gmeta::{In,Metadata,InOut};
use primitive_types::{H256, H512};

pub type AttributeId = u32;
pub type Price = u128;
pub type TamagotchiId = ActorId;

#[derive(Encode, Decode,Clone,TypeInfo)]
pub struct Data {
    /// The attribute title, for example: "Weapon".
    pub title: String,
    /// Description of the attribute.
    pub description: String,
    /// URL to associated media (here it should be an attribute picture).
    pub media: String,
}

#[derive(Decode, Encode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct InitFT {
   
    pub ft_contract_id: ActorId,
}

#[derive(Encode, Decode, TypeInfo, Debug)]
pub enum FTokenAction {
    Message {
        transaction_id: u64,
        payload: LogicAction,
    },
    UpdateLogicContract {
        ft_logic_code_hash: H256,
        storage_code_hash: H256,
    },
    GetBalance(ActorId),
    GetPermitId(ActorId),
    Clear(H256),
    MigrateStorageAddresses,
}

#[derive(Encode, Debug, Decode, TypeInfo, Copy, Clone)]
pub enum LogicAction {
    Mint {
        recipient: ActorId,
        amount: u128,
    },
    Burn {
        sender: ActorId,
        amount: u128,
    },
    Transfer {
        sender: ActorId,
        recipient: ActorId,
        amount: u128,
    },
    Approve {
        approved_account: ActorId,
        amount: u128,
    },
    Permit {
        owner_account: ActorId,
        approved_account: ActorId,
        amount: u128,
        permit_id: u128,
        sign: H512,
    },
}


#[derive(Debug, Encode, Decode, TypeInfo, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum FTokenEvent {
    Ok,
    Err,
    Balance(u128),
    PermitId(u128),
}


#[derive(Encode, Decode,TypeInfo)]
pub enum StoreAction {
   
    CreateAttribute {
        attribute_id: AttributeId,
        metadata: Data,
        price: Price
    },
    BuyAttribute {
        attribute_id: AttributeId,
    }
}

#[derive(Encode, Decode,TypeInfo)]
pub enum StoreEvent {
    AttributeCreated {
        attribute_id: AttributeId,
    },
   
    AttributeSold {
        success: bool,
    },
    CompletePrevTx {
        attribute_id: AttributeId,
    }
}


#[derive(Encode, Decode,TypeInfo,Default)]
pub struct AttributeStore {
   pub admin: ActorId,
   pub ft_contract_id: ActorId,
   pub attributes: BTreeMap<AttributeId, (Data, Price)>,
   pub owners: BTreeMap<TamagotchiId, BTreeSet<AttributeId>>,
   pub transaction_id:  u64,
   pub transactions: BTreeMap<TamagotchiId, ( u64, AttributeId)>,
}


pub struct ContractMetadata;

impl Metadata for ContractMetadata {
    type Init =In<InitFT>;
    type Handle = InOut< StoreAction,  StoreEvent>;
    type Reply = ();
    type Others = ();
    type Signal = ();
    type State = AttributeStore;
}
