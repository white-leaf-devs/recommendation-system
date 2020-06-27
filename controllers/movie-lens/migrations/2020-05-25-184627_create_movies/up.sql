-- Your SQL goes here

CREATE TABLE movies (
    id SERIAL PRIMARY KEY,
    title VARCHAR NOT NULL,
    genres VARCHAR NOT NULL
)