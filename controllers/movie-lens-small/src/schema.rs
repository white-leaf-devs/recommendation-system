table! {
    movies (id) {
        id -> Int4,
        title -> Varchar,
        genres -> Varchar,
    }
}

table! {
    users (id) {
        id -> Int4,
    }
}

allow_tables_to_appear_in_same_query!(
    movies,
    users,
);
