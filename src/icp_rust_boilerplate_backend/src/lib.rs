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
    static BUS_MAINTENANCE_STORAGE: RefCell<StableBTreeMap<u64, BusMaintenanceRecord, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))) // Assuming MemoryId::new(4) for bus maintenance storage
        ));
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct BusPayload {
    make: String,
    model: String,
    year: u32,
    color: String,
    owner: String,
    is_booked: bool, // Add is_booked field to payload
}

#[derive(candid::CandidType, Serialize, Deserialize, Default, Clone)]
struct Customer {
    id: u64,
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
#[derive(candid::CandidType, Serialize, Deserialize, Default, Clone)]
struct BusMaintenanceRecord {
    id: u64,
    bus_id: u64,
    maintenance_date: u64,
    details: String,
}

impl Storable for BusMaintenanceRecord {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for BusMaintenanceRecord {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
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
fn add_bus(bus: BusPayload) -> Option<Bus> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let bus = Bus {
        id,
        make: bus.make,
        model: bus.model,
        year: bus.year,
        color: bus.color,
        created_at: time(),
        updated_at: None,
        owner: bus.owner,
        is_booked: bus.is_booked, // Set is_booked from payload
    };
    do_insert_bus(&bus);
    Some(bus)
}

#[ic_cdk::update]
fn update_bus(id: u64, payload: BusPayload) -> Result<Bus, Error> {
    match BUS_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut bus) => {
            bus.make = payload.make;
            bus.model = payload.model;
            bus.year = payload.year;
            bus.color = payload.color;
            bus.updated_at = Some(time());
            bus.owner = payload.owner;
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
#[ic_cdk::query]
fn search_buses(make: Option<String>, model: Option<String>, year: Option<u32>, color: Option<String>, is_booked: Option<bool>) -> Vec<Bus> {
    BUS_STORAGE
        .with(|service| {
            service.borrow()
                .iter()
                .filter(|(_, bus)| {
                    make.as_ref().map_or(true, |m| &bus.make == m) &&
                    model.as_ref().map_or(true, |m| &bus.model == m) &&
                    year.map_or(true, |y| bus.year == y) &&
                    color.as_ref().map_or(true, |c| &bus.color == c) &&
                    is_booked.map_or(true, |b| bus.is_booked == b)
                })
                .map(|(_, bus)| bus.clone())
                .collect()
        })
}
#[ic_cdk::query]
fn get_customer_reservations(customer_id: u64) -> Vec<Reservation> {
    // Assuming MemoryId::new(3) is used for reservation storage
    let reservation_storage = MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)));
    StableBTreeMap::<u64, Reservation, Memory>::init(reservation_storage)
        .borrow()
        .iter()
        .filter(|(_, reservation)| reservation.customer_id == customer_id)
        .map(|(_, reservation)| reservation.clone())
        .collect()
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
#[ic_cdk::update]
fn add_bus_maintenance_record(record: BusMaintenanceRecord) -> Result<BusMaintenanceRecord, Error> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let record = BusMaintenanceRecord { id, ..record };
    BUS_MAINTENANCE_STORAGE.with(|service| service.borrow_mut().insert(record.id, record.clone()));
    Ok(record)
}

#[ic_cdk::query]
fn get_bus_maintenance_records(bus_id: u64) -> Vec<BusMaintenanceRecord> {
    BUS_MAINTENANCE_STORAGE.with(|service| {
        service.borrow()
            .iter()
            .filter(|(_, record)| record.bus_id == bus_id)
            .map(|(_, record)| record.clone())
            .collect()
    })
}

#[ic_cdk::update]
fn delete_bus_maintenance_record(record_id: u64) -> Result<(), Error> {
    let removed = BUS_MAINTENANCE_STORAGE.with(|service| service.borrow_mut().remove(&record_id));
    match removed {
        Some(_) => Ok(()),
        None => Err(Error::NotFound { msg: format!("Maintenance record with id={} not found.", record_id) }),
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
#[derive(candid::CandidType, Serialize, Deserialize, Default, Clone)]
struct ReservationTimeRange {
    start_time: u64,
    end_time: u64,
}

#[ic_cdk::query]
fn check_bus_availability(bus_id: u64, time_range: ReservationTimeRange) -> Result<bool, Error> {
    let reservations = get_reservations_for_bus(bus_id)?;
    Ok(!reservations.iter().any(|reservation| {
        let reservation_time = reservation.reservation_time;
        reservation_time >= time_range.start_time && reservation_time <= time_range.end_time
    }))
}

fn get_reservations_for_bus(bus_id: u64) -> Result<Vec<Reservation>, Error> {
    // Assuming MemoryId::new(3) is used for reservation storage
    let reservation_storage = MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)));
    let reservations = StableBTreeMap::<u64, Reservation, Memory>::init(reservation_storage)
        .borrow()
        .iter()
        .filter(|(_, reservation)| reservation.bus_id == bus_id)
        .map(|(_, reservation)| reservation.clone())
        .collect();
    Ok(reservations)
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
