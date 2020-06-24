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

After that you should note that the prompt has changed indicating the database you are connected to, showing something like this:

```
(books) >>
```

### Searchby

We will use the term `searchby` later on, it's used as an dynamic identifier for users and items and each database, this allows us to search one of those by `id` or `name`

```python
# Syntax
searchby = id('string') | name('string')


# Example
id('123')
name('Patrick C')
```

### User based distance methods

For some functions it's necessary to specify the distance method, those use the term `user_method` and accept the following values

- Manhattan distance: `manhattan`
- Euclidean distance: `euclidean`
- Minkowski distance: `minkowski(<number>)`
- Jaccard index: `jacc_index`
- Jaccard distance: `jacc_distance`
- Cosine similarity: `cosine`
- Pearson's correlation: `pearson_c`
- Pearson's approximation: `pearson_a`

### Item based distance methods

Just like the above section some other functions need a method, those use the term `item_method` and accept the following value

- Adjusted cosine: `adj_cosine`
- Slope one: `slope_one`

### Functions

In the following functions an argument with a `?` indicates it's optional.

###### `query_user`

Query an user by its `id` or `name` 

```python
# Syntax
query_user(searchby)

# Example
query_user(id('243'))
```

###### `query_item`

Query an item by its `id` or `name`

```python
# Syntax
query_item(searchby)

# Example
query_item(name('The Great Gatsby'))
```

###### `query_ratings`

Query the ratings for an user by its `id` or `name`

```python
# Syntax 
query_ratings(searchby)

# Example
query_ratings(id('12'))
```

###### `user_distance`

Compute the distance between two specified users

```python
# Syntax
user_distance(searchby, searchby, user_method)

# Example
user_distance(id('243'), name('Alan'), minkowski(3))
```

###### `item_distance`

Compute the distance between two specified items 

```python
# Syntax
item_distance(searchby, searchby, item_method)

# Example 
item_distance(id('12'), id('11'), adj_cosine)
```

###### `user_knn`

Find the `k` nearest neighbors for a given user, optionally by chunks of `chunk_size`

```python
# Syntax
user_knn(number, searchby, user_method, chunk_size?)

# Examples
user_knn(5, id('243'), euclidean) # without chunks
user_knn(5, id('411'), euclidean, 100) # chunk_size = 100
```

###### `user_based_predict`

Try to predict an item score for the specified user, this function works with a `knn` using distance between users so its signature receives some of the parameters needed by the underneath `knn`, this function can also work by chunks of `chunk_size`

```python
# Syntax
user_based_predict(number, searchby, searchby, user_method, chunk_size?)

# Examples
user_based_predict(50, id('123'), name('Alien'), euclidean)
user_based_predict(90, id('234'), name('Alien'), euclidean, 100)
```

###### `item_based_predict`

Try to predict an item score for the specified user, this function doesn't use a `knn` and instead use a distance between items, this function only works with chunks.

```python
# Syntax
item_based_predict(searchby, searchby, item_method, chunk_size)

# Example
item_based_predict(id('123'), name('The Great Gatsby'), adj_cosine, 100)
```

###### `enter_matrix`

Enter "the matrix" by chunks, this uses item distances. This puts you into a sub shell where you can move in the matrix and get some values

```python
# Syntax 
enter_matrix(vert_chunk_size, hori_chunk_size, item_method)

# Example
enter_matrix(100, 100, adj_cosine)
```

###### `move_to` (only in `matrix` shell)

Move to another chunk inside the matrix

```python
# Syntax
move_to(number, number)

# Example
move_to(0, 1)
```

###### `get` (only in `matrix` shell)

From the matrix get the value for two specified items

```python
# Syntax
get(searchby, searchby)

# Example
get(id('123'), id('243'))
```

### Disconnecting and exiting

If you wish to try another database you can simple type `d<Enter>` and you will disconnect from the current database, `<CTRL+C>` and `<CTRL+D>` works as expected, cancelling current line and exiting.
