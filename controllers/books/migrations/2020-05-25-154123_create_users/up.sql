-- Your SQL goes here

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    location VARCHAR NOT NULL,
    age SMALLINT DEFAULT NULL 
)