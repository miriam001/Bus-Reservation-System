#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};
use std::borrow::{Borrow, BorrowMut};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Bus {
    id: u64,
    make: String,
    model: String,
    year: u32,
    color: String,
    created_at: u64,
    updated_at: Option<u64>,
    owner: String,
    is_booked: bool, // New field for booking status
}

impl Storable for Bus {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Bus {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static BUS_STORAGE: RefCell<StableBTreeMap<u64, Bus, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
        ));
}

// Additional imports for Result and Error
use ic_cdk::export::candid::{CandidType, Deserialize, Func, Principal};
use ic_cdk::{api, export::candid, export::Principal};
use serde::export::Formatter;

// Error type
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    InvalidInput { msg: String },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotFound { msg } | Error::InvalidInput { msg } => write!(f, "{}", msg),
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Helper function for error conversion
fn error<T>(msg: &str) -> Result<T, Error> {
    Err(Error::InvalidInput {
        msg: msg.to_string(),
    })
}

#[ic_cdk::query]
fn get_bus(id: u64) -> Result<Bus, Error> {
    match _get_bus(&id) {
        Some(bus) => Ok(bus),
        None => Err(Error::NotFound {
            msg: format!("a bus with id={} not found", id),
        }),
    }
}

#[ic_cdk::update]
fn add_bus(bus: Bus) -> Result<Bus, Error> {
    // Validation checks
    if bus.year < 1900 || bus.year > time().into() {
        return error("Invalid year for the bus");
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let bus = Bus {
        id,
        created_at: time(),
        updated_at: None,
        ..bus
    };
    do_insert_bus(&bus);
    Ok(bus)
}

#[ic_cdk::update]
fn update_bus(id: u64, payload: Bus) -> Result<Bus, Error> {
    // Validation checks
    if payload.year < 1900 || payload.year > time().into() {
        return error("Invalid year for the bus");
    }

    match BUS_STORAGE.with(|service| service.borrow_mut().get_mut(&id)) {
        Some(mut bus) => {
            bus.make = payload.make;
            bus.model = payload.model;
            bus.year = payload.year;
            bus.color = payload.color;
            bus.updated_at = Some(time());
            bus.owner = payload.owner;
            bus.is_booked = payload.is_booked;
            do_insert_bus(&bus);
            Ok(bus.clone())
        }
        None => Err(Error::NotFound {
            msg: format!("couldn't update a bus with id={}. bus not found", id),
        }),
    }
}

#[ic_cdk::query]
fn is_booked(id: u64) -> Result<bool, Error> {
    match _get_bus(&id) {
        Some(bus) => Ok(bus.is_booked),
        None => Err(Error::NotFound {
            msg: format!("a bus with id={} not found", id),
        }),
    }
}

fn do_insert_bus(bus: &Bus) {
    BUS_STORAGE.with(|service| service.borrow_mut().insert(bus.id, bus.clone()));
}

#[ic_cdk::update]
fn delete_bus(id: u64) -> Result<Bus, Error> {
    match BUS_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(bus) => Ok(bus),
        None => Err(Error::NotFound {
            msg: format!("couldn't delete a bus with id={}. bus not found.", id),
        }),
    }
}

#[ic_cdk::update]
fn add_customer(name: String, contact: String) -> Option<Customer> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let customer = Customer {
        id,
        name,
        contact,
    };
    do_insert_customer(&customer);
    Some(customer)
}

fn do_insert_customer(customer: &Customer) {
    // Assuming MemoryId::new(2) is reserved for customer storage
    let customer_storage = MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)));
    StableBTreeMap::<u64, Customer, Memory>::init(customer_storage)
        .borrow_mut()
        .insert(customer.id, customer.clone());
}

#[ic_cdk::query]
fn get_customer(id: u64) -> Result<Customer, Error> {
    match _get_customer(&id) {
        Some(customer) => Ok(customer),
        None => Err(Error::NotFound {
            msg: format!("a customer with id={} not found", id),
        }),
    }
}

fn _get_customer(id: &u64) -> Option<Customer> {
    // Assuming MemoryId::new(2) is reserved for customer storage
    let customer_storage = MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)));
    StableBTreeMap::<u64, Customer, Memory>::init(customer_storage)
        .borrow()
        .get(id)
}

#[ic_cdk::update]
fn delete_customer(id: u64) -> Result<Customer, Error> {
    match _get_customer(&id) {
        Some(customer) => {
            // Assuming MemoryId::new(2) is reserved for customer storage
            let customer_storage = MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)));
            StableBTreeMap::<u64, Customer, Memory>::init(customer_storage)
                .borrow_mut()
                .remove(&id);
            Ok(customer)
        }
        None => Err(Error::NotFound {
            msg: format!("a customer with id={} not found", id),
        }),
    }
}

#[ic_cdk::update]
fn make_reservation(bus_id: u64, customer_id: u64) -> Result<Reservation, Error> {
    match (_get_bus(&bus_id), _get_customer(&customer_id)) {
        (Some(_), Some(_)) => {
            let reservation = Reservation {
                bus_id,
                customer_id,
                reservation_time: time(),
            };
            do_insert_reservation(&reservation);
            Ok(reservation)
        }
        _ => Err(Error::NotFound {
            msg: "Bus or customer not found for reservation".to_string(),
        }),
    }
}

fn do_insert_reservation(reservation: &Reservation) {
    // Assuming MemoryId::new(3) is reserved for reservation storage
    let reservation_storage = MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)));
    
    StableBTreeMap::<u64, Reservation, Memory>::init(reservation_storage)
        .borrow_mut()
        .insert(reservation.bus_id, reservation.clone());
}

#[ic_cdk::query]
fn get_reservation(bus_id: u64) -> Result<Reservation, Error> {
    match _get_reservation(&bus_id) {
        Some(reservation) => Ok(reservation),
        None => Err(Error::NotFound {
            msg: format!("a reservation for bus_id={} not found", bus_id),
        }),
    }
}

fn _get_reservation(bus_id: &u64) -> Option<Reservation> {
    // Assuming MemoryId::new(3) is reserved for reservation storage
    let reservation_storage = MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)));
    StableBTreeMap::<u64, Reservation, Memory>::init(reservation_storage)
        .borrow()
        .get(bus_id)
}

#[ic_cdk::update]
fn cancel_reservation(bus_id: u64) -> Result<(), Error> {
    match _get_reservation(&bus_id) {
        Some(_) => {
            // Assuming MemoryId::new(3) is reserved for reservation storage
            let reservation_storage = MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)));
            StableBTreeMap::<u64, Reservation, Memory>::init(reservation_storage)
                .borrow_mut()
                .remove(&bus_id);
            Ok(())
        }
        None => Err(Error::NotFound {
            msg: format!("a reservation for bus_id={} not found", bus_id),
        }),
    }
}

#[ic_cdk::query]
fn generate_report() -> Vec<Bus> {
    // Assuming MemoryId::new(1) is reserved for bus storage
    let bus_storage = MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)));
    StableBTreeMap::<u64, Bus, Memory>::init(bus_storage)
        .borrow()
        .iter()
        .map(|(_, bus)| bus.clone())
        .collect()
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
}

fn _get_bus(id: &u64) -> Option<Bus> {
    // Assuming MemoryId::new(1) is reserved for bus storage
    let bus_storage = MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)));
    StableBTreeMap::<u64, Bus, Memory>::init(bus_storage)
        .borrow()
        .get(id)
}

ic_cdk::export_candid!();
