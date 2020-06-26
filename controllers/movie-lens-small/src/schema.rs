table! {
    means (id) {
        id -> Int4,
        user_id -> Int4,
        val -> Float8,
        score_number -> Int4,
    }
}

table! {
    movies (id) {
        id -> Int4,
        title -> Varchar,
        genres -> Varchar,
    }
}

table! {
    ratings (id) {
        id -> Int4,
        user_id -> Int4,
        movie_id -> Int4,
        score -> Float8,
    }
}

table! {
    users (id) {
        id -> Int4,
    }
}

joinable!(means -> users (user_id));
joinable!(ratings -> movies (movie_id));
joinable!(ratings -> users (user_id));

allow_tables_to_appear_in_same_query!(
    means,
    movies,
    ratings,
    users,
);
