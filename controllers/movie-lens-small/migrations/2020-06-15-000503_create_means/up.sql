-- Your SQL goes here
CREATE TABLE means
(
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id),
    val FLOAT NOT NULL,
    score_number INTEGER NOT NULL
)