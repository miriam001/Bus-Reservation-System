export const idlFactory = ({ IDL }) => {
  const BusPayload = IDL.Record({
    'is_booked' : IDL.Bool,
    'model' : IDL.Text,
    'owner' : IDL.Text,
    'make' : IDL.Text,
    'color' : IDL.Text,
    'year' : IDL.Nat32,
  });
  const Bus = IDL.Record({
    'id' : IDL.Nat64,
    'is_booked' : IDL.Bool,
    'model' : IDL.Text,
    'updated_at' : IDL.Opt(IDL.Nat64),
    'owner' : IDL.Text,
    'make' : IDL.Text,
    'color' : IDL.Text,
    'year' : IDL.Nat32,
    'created_at' : IDL.Nat64,
  });
  const Customer = IDL.Record({
    'id' : IDL.Nat64,
    'contact' : IDL.Text,
    'name' : IDL.Text,
  });
  const Error = IDL.Variant({ 'NotFound' : IDL.Record({ 'msg' : IDL.Text }) });
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : Error });
  const Result_1 = IDL.Variant({ 'Ok' : Bus, 'Err' : Error });
  const Result_2 = IDL.Variant({ 'Ok' : Customer, 'Err' : Error });
  const Reservation = IDL.Record({
    'reservation_time' : IDL.Nat64,
    'customer_id' : IDL.Nat64,
    'bus_id' : IDL.Nat64,
  });
  const Result_3 = IDL.Variant({ 'Ok' : Reservation, 'Err' : Error });
  const Result_4 = IDL.Variant({ 'Ok' : IDL.Bool, 'Err' : Error });
  return IDL.Service({
    'add_bus' : IDL.Func([BusPayload], [IDL.Opt(Bus)], []),
    'add_customer' : IDL.Func([IDL.Text, IDL.Text], [IDL.Opt(Customer)], []),
    'cancel_reservation' : IDL.Func([IDL.Nat64], [Result], []),
    'delete_bus' : IDL.Func([IDL.Nat64], [Result_1], []),
    'delete_customer' : IDL.Func([IDL.Nat64], [Result_2], []),
    'generate_report' : IDL.Func([], [IDL.Vec(Bus)], ['query']),
    'get_bus' : IDL.Func([IDL.Nat64], [Result_1], ['query']),
    'get_customer' : IDL.Func([IDL.Nat64], [Result_2], ['query']),
    'get_reservation' : IDL.Func([IDL.Nat64], [Result_3], ['query']),
    'is_booked' : IDL.Func([IDL.Nat64], [Result_4], ['query']),
    'make_reservation' : IDL.Func([IDL.Nat64, IDL.Nat64], [Result_3], []),
    'update_bus' : IDL.Func([IDL.Nat64, BusPayload], [Result_1], []),
  });
};
export const init = ({ IDL }) => { return []; };
