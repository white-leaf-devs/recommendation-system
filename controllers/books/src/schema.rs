table! {
    books (id) {
        id -> Varchar,
        title -> Varchar,
        author -> Varchar,
        year -> Int2,
        publisher -> Varchar,
    }
}

table! {
    ratings (id) {
        id -> Int4,
        user_id -> Int4,
        book_id -> Varchar,
        score -> Float8,
    }
}

table! {
    users (id) {
        id -> Int4,
        location -> Varchar,
        age -> Nullable<Int2>,
    }
}

joinable!(ratings -> books (book_id));
joinable!(ratings -> users (user_id));

allow_tables_to_appear_in_same_query!(
    books,
    ratings,
    users,
);
