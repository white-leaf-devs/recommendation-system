-- Your SQL goes here

CREATE TABLE books (
    id VARCHAR UNIQUE PRIMARY KEY,
    title VARCHAR NOT NULL,
    author VARCHAR NOT NULL,
    year SMALLINT NOT NULL,
    publisher VARCHAR NOT NULL
)