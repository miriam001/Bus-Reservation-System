import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface Bus {
  'id' : bigint,
  'is_booked' : boolean,
  'model' : string,
  'updated_at' : [] | [bigint],
  'owner' : string,
  'make' : string,
  'color' : string,
  'year' : number,
  'created_at' : bigint,
}
export interface BusPayload {
  'is_booked' : boolean,
  'model' : string,
  'owner' : string,
  'make' : string,
  'color' : string,
  'year' : number,
}
export interface Customer { 'id' : bigint, 'contact' : string, 'name' : string }
export type Error = { 'NotFound' : { 'msg' : string } };
export interface Reservation {
  'reservation_time' : bigint,
  'customer_id' : bigint,
  'bus_id' : bigint,
}
export type Result = { 'Ok' : null } |
  { 'Err' : Error };
export type Result_1 = { 'Ok' : Bus } |
  { 'Err' : Error };
export type Result_2 = { 'Ok' : Customer } |
  { 'Err' : Error };
export type Result_3 = { 'Ok' : Reservation } |
  { 'Err' : Error };
export type Result_4 = { 'Ok' : boolean } |
  { 'Err' : Error };
export interface _SERVICE {
  'add_bus' : ActorMethod<[BusPayload], [] | [Bus]>,
  'add_customer' : ActorMethod<[string, string], [] | [Customer]>,
  'cancel_reservation' : ActorMethod<[bigint], Result>,
  'delete_bus' : ActorMethod<[bigint], Result_1>,
  'delete_customer' : ActorMethod<[bigint], Result_2>,
  'generate_report' : ActorMethod<[], Array<Bus>>,
  'get_bus' : ActorMethod<[bigint], Result_1>,
  'get_customer' : ActorMethod<[bigint], Result_2>,
  'get_reservation' : ActorMethod<[bigint], Result_3>,
  'is_booked' : ActorMethod<[bigint], Result_4>,
  'make_reservation' : ActorMethod<[bigint, bigint], Result_3>,
  'update_bus' : ActorMethod<[bigint, BusPayload], Result_1>,
}
