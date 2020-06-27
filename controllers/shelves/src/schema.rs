table! {
    books (id) {
        id -> Int4,
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
        book_id -> Int4,
        score -> Float8,
    }
}

table! {
    users (id) {
        id -> Int4,
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
