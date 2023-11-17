## STORE TAMAGOCHI CONTRACT

## Directorio IO

### Agrega las siguientes dependencias.
**comando:**
```rust
#![no_std]
use gstd::{prelude::*, ActorId};
use primitive_types::{H256, H512};

pub type AttributeId = u32;
pub type Price = u128;
pub type TamagotchiId = ActorId;
```


### PASO 1 Definir las acciones de Store.
**comando:**
```rust
#[derive(Encode, Decode)]
pub enum StoreAction {
   
    CreateAttribute {
        attribute_id: AttributeId,
        metadata: Metadata,
        price: Price
    },
    BuyAttribute {
        attribute_id: AttributeId,
    }
}

```

### PASO 2 Definir las estructura del Store.
**comando:**
```rust

#[derive(Encode, Decode,Default)]
pub struct AttributeStore {
    admin: ActorId,
    ft_contract_id: ActorId,
    attributes: BTreeMap<AttributeId, (Metadata, Price)>,
    owners: BTreeMap<TamagotchiId, BTreeSet<AttributeId>>,
    transaction_id:  u64,
    transactions: BTreeMap<TamagotchiId, ( u64, AttributeId)>,
}

```


### PASO 3 Declarar los eventos 

**comando:**
```rust
#[derive(Encode, Decode)]
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

```

### PASO 4 Acciones y eventos del token fraccionado a controlar

**comando:**
```rust
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

```

## Directorio src

### Agrega las siguientes dependencias.
**comando:**
```rust
#![no_std]
use gstd::{msg,exec, prelude::*, ActorId};
use io::*;

```

### PASO 1 Definimos el estado principal como una variable estática
**comando:**
```rust
static mut STORE: Option<AttributeStore> = None;
```


### PASO 2 Definimos una estructura Tamagotchi para incorporar implementaciones

**comando:**
```rust
#[derive(Encode, Decode,Default)]
pub struct AttributeStore {
    admin: ActorId,
    ft_contract_id: ActorId,
    attributes: BTreeMap<AttributeId, (Metadata, Price)>,
    owners: BTreeMap<TamagotchiId, BTreeSet<AttributeId>>,
    transaction_id:  u64,
    transactions: BTreeMap<TamagotchiId, ( u64, AttributeId)>,
}

impl AttributeStore {
    fn create_attribute(
        &mut self,
        attribute_id: AttributeId,
        metadata: &Metadata,
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
```

### PASO 3 Agregamos la funcion init para inicializar las variables

**comando:**
```rust
#[no_mangle]
extern "C" fn init() {
    let ft_contract_id: ActorId = msg::load()
        .expect("Unable to decode `ActorId`");
    let store = AttributeStore {
        admin: msg::source(),
        ft_contract_id,
        ..Default::default()
    };
    unsafe { STORE = Some(store) };
}
```

### PASO 4 Definimos esta función main usando el macro async_main
**comando:**
```rust
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
```
