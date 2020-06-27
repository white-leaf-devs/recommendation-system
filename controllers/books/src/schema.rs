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
    means (user_id) {
        user_id -> Int4,
        val -> Float8,
        score_number -> Int4,
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

joinable!(means -> users (user_id));
joinable!(ratings -> books (book_id));
joinable!(ratings -> users (user_id));

allow_tables_to_appear_in_same_query!(
    books,
    means,
    ratings,
    users,
);
