#![no_std]
use gstd::{msg,exec, prelude::*, ActorId};
use io::*;

static mut STORE: Option<AttributeStore> = None;
static mut ADDRESSFT:Option<InitFT> = None;


#[derive(Encode, Decode,Default)]
pub struct AttributeStore {
   pub admin: ActorId,
   pub ft_contract_id: ActorId,
   pub attributes: BTreeMap<AttributeId, (Data, Price)>,
   pub owners: BTreeMap<TamagotchiId, BTreeSet<AttributeId>>,
   pub transaction_id:  u64,
   pub transactions: BTreeMap<TamagotchiId, ( u64, AttributeId)>,
}

impl AttributeStore {
    fn create_attribute(
        &mut self,
        attribute_id: AttributeId,
        metadata: &Data,
        price: Price
    ) {
        assert_eq!(msg::source(), self.admin,
            "Only admin can add attributes");
    
        if self
            .attributes
            .insert(attribute_id, (metadata.clone(), price))
            .is_some()
        {
            panic!("Attribute with that ID already exists");
        }
    
        msg::reply(StoreEvent::AttributeCreated { attribute_id }, 0)
            .expect("Error in sending a reply StoreEvent::AttributeCreated");
    }


    async fn buy_attribute(&mut self, attribute_id: AttributeId) {
        let (transaction_id, attribute_id) = if let Some((transaction_id, prev_attribute_id)) =
            self.transactions.get(&msg::source())
        {
            // If `prev_attribute_id` is not equal to `attribute_id`, it means the transaction wasn't completed
            // We'll ask the Tamagotchi contract to complete the previous transaction
            if attribute_id != *prev_attribute_id {
                msg::reply(
                    StoreEvent::CompletePrevTx {
                        attribute_id: *prev_attribute_id,
                    },
                    0,
                )
                .expect("Error in sending a reply `StoreEvent::CompletePrevTx`");
                return;
            }
                (*transaction_id, *prev_attribute_id)
            } else {
                let current_transaction_id = self.transaction_id;
                self.transaction_id = self.transaction_id.wrapping_add(1);
                self.transactions
                    .insert(msg::source(), (current_transaction_id, attribute_id));
                (current_transaction_id, attribute_id)
            };
    
            let result = self.sell_attribute(transaction_id, attribute_id).await;
            self.transactions.remove(&msg::source());
    
            msg::reply(StoreEvent::AttributeSold { success: result }, 0)
                .expect("Error in sending a reply `StoreEvent::AttributeSold`");
    }


    async fn transfer_tokens(
        transaction_id: u64,
        token_address: &ActorId,
        from: &ActorId,
        to: &ActorId,
        amount_tokens: u128,
    ) -> Result<(), ()> {
        let reply = msg::send_for_reply_as::<_, FTokenEvent>(
            *token_address,
            FTokenAction::Message {
                transaction_id,
                payload: LogicAction::Transfer {
                    sender: *from,
                    recipient: *to,
                    amount: amount_tokens,
                },
            },
            0,
            0,
         )
        .expect("Error in sending a message `FTokenAction::Message`")
        .await;
    
        match reply {
            Ok(FTokenEvent::Ok) => Ok(()),
            _ => Err(()),
        }
    }


    async fn sell_attribute(
        &mut self,
        transaction_id: u64,
        attribute_id: AttributeId,
    ) -> bool {
        let (_, price) = self
            .attributes
            .get(&attribute_id)
            .expect("Can't get attribute_id");
    
        if Self::transfer_tokens(
            transaction_id,
            &self.ft_contract_id,
            &msg::source(),
            &exec::program_id(),
            *price,
        )
        .await
        .is_ok()
        {
            self.owners
                .entry(msg::source())
                .and_modify(|attributes| {
                    attributes.insert(attribute_id);
                })
                .or_insert_with(|| [attribute_id].into());
            return true;
        }
        false
    }
}

#[gstd::async_main]
async fn main() {
    let action: StoreAction = msg::load()
        .expect("Unable to decode `StoreAction`");
    let store: &mut AttributeStore = unsafe {
        STORE.get_or_insert(Default::default())
    };
    match action {
        StoreAction::CreateAttribute {
            attribute_id,
            metadata,
            price
        } => store.create_attribute(attribute_id, &metadata, price),
        StoreAction::BuyAttribute { attribute_id } =>
            store.buy_attribute(attribute_id).await,
    }
}

#[no_mangle]
extern "C" fn init() {

    let config: InitFT = msg::load().expect("Unable to decode InitFT");

    if config.ft_contract_id.is_zero() {
        panic!("FT program address can't be 0");
    }

    let initft = InitFT {
        ft_contract_id: config.ft_contract_id
    };

    unsafe {
        ADDRESSFT = Some(initft);
    }

  
    let store = AttributeStore {
        admin: msg::source(),
        ft_contract_id: config.ft_contract_id,
        ..Default::default()
    };
    unsafe { STORE = Some(store) };
}


// 5. Create the state() function of your contract.
#[no_mangle]
extern "C" fn state() {
    let state = unsafe {
        STORE.get_or_insert(Default::default())
    };
    msg::reply(state, 0).expect("Failed to share state");
}