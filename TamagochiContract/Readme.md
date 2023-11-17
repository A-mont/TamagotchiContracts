# TAMAGOCHI CONTRACT


## Directorio IO

### Agrega las siguientes dependencias.
**comando:**
```rust
#![no_std]

use codec::{Decode, Encode};
use gmeta::{InOut, Metadata};
use gstd::{prelude::*, ActorId};
pub type AttributeId = u32;
pub type TransactionId = u64;
```



### PASO 1 Definir las acciones.
**comando:**
```rust
#[derive(Encode, Decode, TypeInfo, Debug)]
pub enum TmgAction {
    Name,
    Age,
    Feed,
    Play,
    Sleep,
    TmgInfo,
}
```

### PASO 2 Definir las estructura del tamagochi.
**comando:**
```rust
#[derive(Default, Encode, Decode, TypeInfo)]
pub struct Tamagotchi {
    pub name: String,
    pub date_of_birth: u64,
    pub owner: ActorId,
    pub fed: u64,
    pub fed_block: u64,
    pub entertained: u64,
    pub entertained_block: u64,
    pub rested: u64,
    pub rested_block: u64,
    pub allowed_account: Option<ActorId>,
}

```


### PASO 3 Declarar los eventos 

**comando:**
```rust
#[derive(Encode, Debug, PartialEq, Eq, Decode, TypeInfo)]
pub enum TmgReply {
    Name(String),
    Age(u64),
    Fed,
    Entertained,
    Slept,
    TmgInfo {
        owner: ActorId,
        name: String,
        date_of_birth: u64,
    },
}

```

### PASO 4 Definimos ContractMetadata y el estado

**comando:**
```rust
pub struct ProgramMetadata;

impl Metadata for ProgramMetadata {
    type Init = InOut<String, ()>;
    type Handle = InOut<TmgAction, TmgReply>;
    type Reply = ();
    type Others = ();
    type Signal = ();
    type State = InOut<(), Tamagotchi>;
}

```

## Directorio src

### Agrega las siguientes dependencias.
**comando:**
```rust
use gstd::{exec, msg, prelude::*, ActorId};
use tmg_io::*;

```

### PASO 1 Definimos las variables y constantes a usar
**comando:**
```rust
// Definimos las varibles.
pub const HUNGER_PER_BLOCK: u64 = 1;
pub const BOREDOM_PER_BLOCK: u64 = 2;
pub const ENERGY_PER_BLOCK: u64 = 2;

pub const FILL_PER_FEED: u64 = 2_000;
pub const FILL_PER_ENTERTAINMENT: u64 = 2_000;
pub const FILL_PER_SLEEP: u64 = 2_000;

pub const MAX_VALUE: u64 = 10_000;

static mut TAMAGOTCHI: Option<Tamagotchi> = None;
```


### PASO 2 Definimos una estructura Tamagotchi para incorporar implementaciones

**comando:**
```rust
#[derive(Default, Encode, Decode, TypeInfo)]
pub struct Tamagotchi {
    pub name: String,
    pub date_of_birth: u64,
    pub owner: ActorId,
    pub fed: u64,
    pub fed_block: u64,
    pub entertained: u64,
    pub entertained_block: u64,
    pub rested: u64,
    pub rested_block: u64,
    pub allowed_account: Option<ActorId>,
}

static mut TAMAGOTCHI: Option<Tamagotchi> = None;

impl Tamagotchi {
    fn feed(&mut self) {
        assert!(!self.tmg_is_dead(), "Tamagotchi has died");
        self.fed_block = exec::block_timestamp();
        self.fed += FILL_PER_FEED - self.calculate_hunger();
        self.fed = if self.fed > MAX_VALUE {
            MAX_VALUE
        } else {
            self.fed
        };
        msg::reply(TmgReply::Fed, 0).expect("Error in a reply `TmgEvent::Fed`");
    }

    fn play(&mut self) {
        assert!(!self.tmg_is_dead(), "Tamagotchi has died");
        self.entertained_block = exec::block_timestamp();
        self.entertained += FILL_PER_ENTERTAINMENT - self.calculate_boredom();
        self.entertained = if self.entertained > MAX_VALUE {
            MAX_VALUE
        } else {
            self.entertained
        };
        msg::reply(TmgReply::Entertained, 0).expect("Error in a reply `TmgEvent::Entertained`");
    }

    fn sleep(&mut self) {
        assert!(!self.tmg_is_dead(), "Tamagotchi has died");
        self.rested_block = exec::block_timestamp();
        self.rested += FILL_PER_SLEEP - self.calculate_energy();
        self.rested = if self.rested > MAX_VALUE {
            MAX_VALUE
        } else {
            self.rested
        };
        msg::reply(TmgReply::Slept, 0).expect("Error in a reply `TmgEvent::Slept`");
    }

    fn calculate_hunger(&self) -> u64 {
        HUNGER_PER_BLOCK * ((exec::block_timestamp() - self.fed_block) / 1_000)
    }

    fn calculate_boredom(&self) -> u64 {
        BOREDOM_PER_BLOCK * ((exec::block_timestamp() - self.entertained_block) / 1000)
    }

    fn calculate_energy(&self) -> u64 {
        ENERGY_PER_BLOCK * ((exec::block_timestamp() - self.rested_block) / 1000)
    }

    fn tmg_info(&self) {
        msg::reply(
            TmgReply::TmgInfo {
                owner: self.owner,
                name: self.name.clone(),
                date_of_birth: self.date_of_birth,
            },
            0,
        )
        .expect("Error in a reply `TmgEvent::TmgInfo");
    }

    fn tmg_is_dead(&self) -> bool {
        let fed = self.fed.saturating_sub(self.calculate_hunger());
        let entertained = self.entertained.saturating_sub(self.calculate_boredom());
        let rested = self.rested.saturating_sub(self.calculate_energy());
        fed == 0 && entertained == 0 && rested == 0
    }
}
```

### PASO 3 Agregamos la funcion init para inicializar las variables

**comando:**
```rust
#[no_mangle]
unsafe extern "C" fn init() {
    let name: String = msg::load().expect("Failed to decode Tamagotchi name");
    // // ⚠️ TODO: Change the tamagotchi name
    // let name = String::from("Best-Tamagotchi");

    let current_block = exec::block_timestamp();

    let tmg = Tamagotchi {
        name,
        date_of_birth: current_block,
        owner: msg::source(),
        fed: MAX_VALUE,
        fed_block: current_block,
        entertained: MAX_VALUE,
        entertained_block: current_block,
        rested: MAX_VALUE,
        rested_block: current_block,
        ..Default::default()
    };
    TAMAGOTCHI = Some(tmg);
}

```

### PASO 4 Definimos esta función handle()

**comando:**
```rust
#[no_mangle]
extern "C" fn handle() {
    let action: TmgAction = msg::load().expect("Unable to decode `TmgAction`");
    let tmg = unsafe { TAMAGOTCHI.get_or_insert(Default::default()) };
    match action {
        TmgAction::Name => {
            msg::reply(TmgReply::Name(tmg.name.clone()), 0)
                .expect("Error in a reply `TmgEvent::Name`");
        }
        TmgAction::Age => {
            let age = exec::block_timestamp() - tmg.date_of_birth;
            msg::reply(TmgReply::Age(age), 0).expect("Error in a reply `TmgEvent::Age`");
            // ⚠️ TODO: Send a reply about the Tamagotchi age
            // Hint: the message payload must be TmgReply::Age(age)
        }
        TmgAction::Feed => tmg.feed(),
        TmgAction::Play => tmg.play(),
        TmgAction::Sleep => tmg.sleep(),
        TmgAction::TmgInfo => tmg.tmg_info(),
    }
}
```

### PASO 5 Definimos la funcion estado
**comando:**
```rust
 #[no_mangle]
extern "C" fn state() {
    let tmg = unsafe { TAMAGOTCHI.get_or_insert(Default::default()) };
    msg::reply(tmg, 0).expect("Failed to share state");
}

```


## Directorio State

### PASO 1 Definimos el estado 
**comando:**
```rust
#![no_std]
use gmeta::metawasm;
use gstd::{exec, prelude::*};
use tmg_io::Tamagotchi;

pub const HUNGER_PER_BLOCK: u64 = 1;
pub const BOREDOM_PER_BLOCK: u64 = 2;
pub const ENERGY_PER_BLOCK: u64 = 2;

#[metawasm]
pub mod metafns {
    pub type State = Tamagotchi;

    pub fn current_state(state: State) -> TmgCurrentState {
        let fed = state.fed.saturating_sub(
            HUNGER_PER_BLOCK * ((exec::block_timestamp() - state.fed_block) / 1_000),
        );
        let entertained = state.entertained.saturating_sub(
            BOREDOM_PER_BLOCK * ((exec::block_timestamp() - state.entertained_block) / 1_000),
        );
        let rested = state.rested.saturating_sub(
            ENERGY_PER_BLOCK * ((exec::block_timestamp() - state.rested_block) / 1_000),
        );
        TmgCurrentState {
            fed,
            entertained,
            rested,
        }
    }
}

#[derive(Encode, Decode, TypeInfo)]
pub struct TmgCurrentState {
    pub fed: u64,
    pub entertained: u64,
    pub rested: u64,
}

```
