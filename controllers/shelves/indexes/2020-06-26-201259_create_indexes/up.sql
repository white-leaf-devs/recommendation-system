-- Your SQL goes here

CREATE INDEX ratings_user_id_idx on ratings(user_id);
CREATE INDEX ratings_book_id_idx on ratings(book_id);
CREATE INDEX means_user_id_idx on means(user_id);