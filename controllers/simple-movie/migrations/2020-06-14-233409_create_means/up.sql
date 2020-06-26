-- Your SQL goes here

CREATE TABLE means
(
    user_id INTEGER REFERENCES users(id),
    val FLOAT NOT NULL,
    score_number INTEGER NOT NULL,
    PRIMARY KEY (user_id)
)