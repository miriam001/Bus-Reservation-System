#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_cdk::caller;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};
use std::borrow::{Borrow, BorrowMut};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Bus {
    id: u64,
    owner_principal: String,
    make: String,
    model: String,
    year: u32,
    color: String,
    created_at: u64,
    updated_at: Option<u64>,
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

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct BusPayload {
    make: String,
    model: String,
    year: u32,
    color: String,
    is_booked: bool, // Add is_booked field to payload
}

#[derive(candid::CandidType, Serialize, Deserialize, Default, Clone)]
struct Customer {
    id: u64,
    customer_principal: String,
    name: String,
    contact: String,
}

impl Storable for Customer {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Customer {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Serialize, Deserialize, Default, Clone)]
struct Reservation {
    bus_id: u64,
    customer_id: u64,
    reservation_time: u64,
}

impl Storable for Reservation {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Reservation {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

fn is_bus_principal(bus: &Bus) -> Result<(), Error> {
    if bus.owner_principal != caller().to_string(){
        return  Err(Error::FailedAuthentication);
    }else {
        Ok(())
    }
}

fn is_customer_principal(customer: &Customer) -> Result<(), Error> {
    if customer.customer_principal != caller().to_string(){
        return  Err(Error::FailedAuthentication);
    }else {
        Ok(())
    }
}

fn is_invalid_string(str: &String) -> bool {
    return str.trim().is_empty()
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
fn add_bus(bus: BusPayload) -> Result<Bus, Error> {
    if is_invalid_string(&bus.color) || is_invalid_string(&bus.make) || is_invalid_string(&bus.model)
    {
        return Err(Error::InvalidInputData { msg: format!("Payload cannot contain empty strings.") })
    }
    if bus.year < 1995 || bus.year > 2024{
        return Err(Error::InvalidInputData { msg: format!("Age of bus needs to be between 1995 and 2024") })
    }
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let bus = Bus {
        id,
        owner_principal: caller().to_string(),
        make: bus.make,
        model: bus.model,
        year: bus.year,
        color: bus.color,
        created_at: time(),
        updated_at: None,
        is_booked: bus.is_booked, // Set is_booked from payload
    };
    do_insert_bus(&bus);
    Ok(bus)
}

#[ic_cdk::update]
fn update_bus(id: u64, payload: BusPayload) -> Result<Bus, Error> {
    if is_invalid_string(&payload.color) || is_invalid_string(&payload.make) || is_invalid_string(&payload.model)
    {
        return Err(Error::InvalidInputData { msg: format!("Payload cannot contain empty strings.") })
    }
    if payload.year < 1995 || payload.year > 2024{
        return Err(Error::InvalidInputData { msg: format!("Age of bus needs to be between 1995 and 2024") })
    }
    match BUS_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut bus) => {
            is_bus_principal(&bus)?;
            bus.make = payload.make;
            bus.model = payload.model;
            bus.year = payload.year;
            bus.color = payload.color;
            bus.updated_at = Some(time());
            bus.is_booked = payload.is_booked; // Update is_booked field
            do_insert_bus(&bus);
            Ok(bus)
        }
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't update a bus with id={}. bus not found",
                id
            ),
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
    let bus = _get_bus(&id).ok_or_else(|| Error::NotFound { msg: format!("Bus with id={} not found.", id) })?;
    is_bus_principal(&bus)?;
    match BUS_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(bus) => Ok(bus),
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't delete a bus with id={}. bus not found.",
                id
            ),
        }),
    }
}

#[ic_cdk::update]
fn add_customer(name: String, contact: String) -> Result<Customer, Error> {
    if is_invalid_string(&name) || is_invalid_string(&contact){
        return Err(Error::InvalidInputData { msg: format!("Payload cannot contain empty strings.") })
    }
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let customer = Customer {
        id,
        customer_principal: caller().to_string(),
        name,
        contact,
    };
    do_insert_customer(&customer);
    Ok(customer)
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
            is_customer_principal(&customer)?;
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
        (Some(bus), Some(customer)) => {
            is_customer_principal(&customer)?;
            if bus.is_booked{
                return Err(Error::AlreadyBooked { msg: format!("Bus with id={} is already booked.", bus_id) })
            }
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
        Some(reservation) => {
            let customer = _get_customer(&reservation.customer_id);
            let bus =_get_bus(&reservation.bus_id);
            if customer.is_none() && bus.is_none(){
                return Err(Error::NotFound { msg: format!("Bus and Customer for this reservation not found.") })
            }
            let caller_to_string = caller().to_string();
            if customer.is_some_and(|customer| customer.customer_principal == caller_to_string)
             || bus.is_some_and(|bus| bus.owner_principal == caller_to_string) {
                // Assuming MemoryId::new(3) is reserved for reservation storage
                let reservation_storage = MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)));
                StableBTreeMap::<u64, Reservation, Memory>::init(reservation_storage)
                    .borrow_mut()
                    .remove(&bus_id);
                Ok(())
            }else{
                return Err(Error::FailedAuthentication)
            }

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
    InvalidInputData{ msg: String},
    AlreadyBooked{msg: String},
    FailedAuthentication
}

fn _get_bus(id: &u64) -> Option<Bus> {
    // Assuming MemoryId::new(1) is reserved for bus storage
    let bus_storage = MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)));
    StableBTreeMap::<u64, Bus, Memory>::init(bus_storage)
        .borrow()
        .get(id)
}

ic_cdk::export_candid!();
