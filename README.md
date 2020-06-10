# (Toy) Recommendation System

## Cloning with Git LFS

This repository uses Git LFS to keep track of some large files (those ones that contain the data), if you don't have LFS on your system you'll not have access to these files, which indeed may not be a problem since you can download them manually from the repository:

- `controllers/books/data.zip`

- `controllers/movie-lens/data.zip`

If you have Git LFS installed in your system it's enough to just clone this repo:

```shell
git clone https://github.com/white-leaf-devs/recommendation-system
```

## Installing Rust

Make sure that you have installed the latest stable Rust toolchain, if you
don't have Rust installed on your computer, you can install it using [`rustup`](https://rustup.rs/)

```shell
rustup install stable
rustup default stable
```

## Populating databases

In order to run anywhere you need to have PostgreSQL installed and running. After
that you have to install [diesel](http://diesel.rs/) CLI (an ORM manager), as you 
may already got `cargo` installed in your system do the following to install `diesel`.

```bash
cargo install diesel_cli --no-default-features --features postgres
```

#### Running migrations and loading data

Supported controllers are defined in `controllers` folder, to setup the database 
run `diesel setup` on each controller directory, this should create the database 
and create the tables. After that you just need to load data into the database by using 
the following command on each controller directory:

```bash
cargo run --release --bin load_data
```

**Note:**  If you don't have Git LFS  you need to download `data.zip` for `books` and `movie-lens` controllers manually from the repository as stated above, if you already have both zips you only need to unzip them and you're ready to go.

## Running and using the CLI

If you managed to get the above steps good you should be able to run the main CLI
tool under the root project just by typing:

```bash
cargo run --release
```

You will be prompted with something like this:

```
Welcome to recommendation-system 0.1.0
>> 
```

First of all, you must `connect` to a database by using the `connect` command, valid
databases names are `books`, `simple-movie` and `movie-lens-small`, for example:

```
>> connect(books)
```

After that you should note that the prompt has changed indicating the database you are
connected to, showing something like this:

```
(books) >>
```

Before you start digging into the provided functions you should now that some of the provided databases can query its users and items by `id` or `name`, we refer this as `searchby` and its syntax is pretty straightforward. You can write `id(<string>)` or `name(<string>)` to search an user or item by its id or name respectively. For distance methods you simply need to pass and identifier:

- Manhattan distance: `manhattan`
- Euclidean distance: `euclidean`
- Minkowski distance: `minkowski(<number>)`
- Jaccard index: `jacc_index`
- Jaccard distance: `jacc_distance`
- Cosine similarity: `cosine`
- Pearson's correlation: `pearson_c`
- Pearson's approximation: `pearson_a`

Now it's time to play with the following provided functions:

- `query_user(searchby)`: Query an user by its `id` or `name`, ex. `query_user(id('243'))`
- `query_item(searchby)`: Query an item by its `id` or `name`, ex. `query_item(name('Alien')`
- `query_ratings(searchby)`: Query the ratings from the user with `id` or `name`, ex. `query_ratings(id('243'))`
- `distance(searchby, searchby, method)`: Calculate distance between both users using the specified method, ex. `distance(id('243'), name('Alan'), minkowski(3))`
- `knn(k, searchby, method)`: Calculate the kNN for the specified user and method, ex. `knn(5, id('243'), euclidean)`
- `predict(k, searchby, searchby, method)`: Predict an item score for the user with the specified k and method to use in the inner knn, ex. `predict(5, id('243'), name('Alien'), euclidean)`

If you wish to try another database you can simple type `d<Enter>` and you will disconnect from the current database, `<CTRL+C>` and `<CTRL+D>` works as expected, cancelling current line and exiting.
