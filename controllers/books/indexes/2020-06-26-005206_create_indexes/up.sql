-- Your SQL goes here

CREATE INDEX ratings_user_id_idx on ratings(user_id);
CREATE INDEX ratings_movie_id_idx on ratings(movie_id);
CREATE INDEX means_user_id_idx on means(user_id);