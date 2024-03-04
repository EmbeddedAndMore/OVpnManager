// @generated automatically by Diesel CLI.

diesel::table! {
    hosts (id) {
        id -> Integer,
        name -> Text,
        ip_address -> Text,
        username -> Text,
        password -> Text,
    }
}

diesel::table! {
    vpns (id) {
        id -> Integer,
        host_id -> Integer,
        name -> Text,
        port -> SmallInt,
        subnet -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        vpn_id -> Integer,
        username -> Text,
        is_plant -> Bool,
        ca_key -> Text,
        password -> Text,
    }
}


diesel::joinable!(vpns -> hosts (host_id));
diesel::joinable!(users -> vpns (vpn_id));



diesel::allow_tables_to_appear_in_same_query!(
    hosts,
    users,
    vpns,
);