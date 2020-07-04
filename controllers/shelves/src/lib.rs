// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

use crate::models::{
    books::Book,
    ratings::Rating,
    users::{Mean, User},
};
use crate::schema::{books, ratings, users};
use anyhow::Error;
use config::Config;
use controller::{eid, error::ErrorKind, maped_ratings, means, ratings, Controller, SearchBy};
use diesel::pg::PgConnection;
use diesel::{delete, insert_into, prelude::*, update};
use models::ratings::NewRating;
use mongodb::bson::doc;
use mongodb::{
    options::{FindOptions, UpdateOptions},
    sync::{Client, Database},
};

use num_traits::Zero;
use std::collections::HashMap;

pub fn establish_connection(url: &str) -> Result<PgConnection, Error> {
    Ok(PgConnection::establish(&url)?)
}

pub struct ShelvesController {
    users_who_rated_mongo: bool,
    pg_conn: PgConnection,
    mongo_db: Database,
}

impl ShelvesController {
    pub fn new() -> Result<Self, Error> {
        let cfg = Config::default();

        Self::from_config(&cfg, "shelves")
    }

    pub fn from_config(config: &Config, name: &str) -> Result<Self, Error> {
        let db = config
            .databases
            .get(name)
            .ok_or_else(|| ErrorKind::DbConfigError(name.into()))?;

        let users_who_rated_mongo = db.users_who_rated_mongo;
        let psql_url = &db.psql_url;
        let mongo_url = &db.mongo_url;
        let mongo_db = &db.mongo_db;

        let pg_conn = establish_connection(psql_url)?;
        let client = Client::with_uri_str(mongo_url)?;
        let mongo_db = client.database(mongo_db);

        Ok(Self {
            users_who_rated_mongo,
            pg_conn,
            mongo_db,
        })
    }
}

impl Controller for ShelvesController {
    type User = User;
    type Item = Book;
    type Rating = Rating;

    fn users(&self) -> Result<Vec<Self::User>, Error> {
        let users = users::table.load::<User>(&self.pg_conn)?;
        Ok(users)
    }

    fn users_by(&self, by: &SearchBy) -> Result<Vec<Self::User>, Error> {
        match by {
            SearchBy::Id(id) => {
                let id: i32 = id.parse()?;
                let users = users::table.filter(users::id.eq(id)).load(&self.pg_conn)?;

                if users.is_empty() {
                    Err(ErrorKind::NotFoundById(id.to_string()).into())
                } else {
                    Ok(users)
                }
            }

            SearchBy::Name(name) => Err(ErrorKind::NotFoundByName(name.clone()).into()),
            SearchBy::Custom(k, v) => Err(ErrorKind::NotFoundByCustom(k.clone(), v.clone()).into()),
        }
    }

    fn users_offset_limit(&self, offset: usize, limit: usize) -> Result<Vec<Self::User>, Error> {
        let users = users::table
            .offset(offset as i64)
            .limit(limit as i64)
            .load::<User>(&self.pg_conn)?;

        Ok(users)
    }

    fn items(&self) -> Result<Vec<Self::Item>, Error> {
        let items = books::table.load::<Book>(&self.pg_conn)?;
        Ok(items)
    }

    fn items_by(&self, by: &SearchBy) -> Result<Vec<Self::Item>, Error> {
        match by {
            SearchBy::Id(id) => {
                let id: i32 = id.parse()?;
                let books = books::table.filter(books::id.eq(id)).load(&self.pg_conn)?;

                if books.is_empty() {
                    Err(ErrorKind::NotFoundById(id.to_string()).into())
                } else {
                    Ok(books)
                }
            }

            SearchBy::Name(name) => Err(ErrorKind::NotFoundByName(name.clone()).into()),
            SearchBy::Custom(k, v) => Err(ErrorKind::NotFoundByCustom(k.clone(), v.clone()).into()),
        }
    }

    fn items_offset_limit(&self, offset: usize, limit: usize) -> Result<Vec<Self::Item>, Error> {
        let items = books::table
            .offset(offset as i64)
            .limit(limit as i64)
            .load::<Book>(&self.pg_conn)?;

        Ok(items)
    }

    fn create_partial_users(
        &self,
        user_ids: &[eid!(Self::User)],
    ) -> Result<Vec<Self::User>, Error> {
        user_ids
            .iter()
            .map(|id| -> Result<User, Error> { Ok(User { id: *id }) })
            .collect()
    }

    fn create_partial_items(
        &self,
        item_ids: &[eid!(Self::Item)],
    ) -> Result<Vec<Self::Item>, Error> {
        item_ids
            .iter()
            .map(|id| -> Result<Book, Error> { Ok(Book { id: *id }) })
            .collect()
    }

    #[allow(clippy::type_complexity)]
    fn users_who_rated(
        &self,
        items: &[Self::Item],
    ) -> Result<maped_ratings!(Self::Item => Self::User), Error> {
        if !self.users_who_rated_mongo {
            let ratings = Rating::belonging_to(items).load::<Rating>(&self.pg_conn)?;

            let mut items_users = HashMap::new();
            for rating in ratings {
                items_users
                    .entry(rating.book_id)
                    .or_insert_with(HashMap::new)
                    .insert(rating.user_id, rating.score);
            }

            Ok(items_users)
        } else {
            let collection = self.mongo_db.collection("users_who_rated");
            let ids: Vec<_> = items.iter().map(|m| m.id).collect();
            let options = FindOptions::builder().show_record_id(false).build();

            let cursor = collection.find(
                doc! {
                    "item_id": { "$in": ids }
                },
                options,
            )?;

            let mut items_users = HashMap::new();
            for doc in cursor {
                let doc = doc?;
                let item_id = doc.get_i32("item_id")?;

                for (user_id, score) in doc.get_document("scores")? {
                    let user_id: i32 = user_id.parse()?;
                    let score = score.as_f64().ok_or_else(|| ErrorKind::BsonConvert)?;
                    items_users
                        .entry(item_id)
                        .or_insert_with(HashMap::new)
                        .insert(user_id, score);
                }
            }

            Ok(items_users)
        }
    }

    fn user_ratings(&self, user: &Self::User) -> Result<ratings!(Self::Item), Error> {
        let ratings = Rating::belonging_to(user)
            .load::<Rating>(&self.pg_conn)?
            .into_iter()
            .map(|rating| (rating.book_id, rating.score))
            .collect();

        Ok(ratings)
    }

    #[allow(clippy::type_complexity)]
    fn all_users_ratings(&self) -> Result<maped_ratings!(Self::User => Self::Item), Error> {
        let ratings = ratings::table.load::<Rating>(&self.pg_conn)?;

        let mut maped_ratings = HashMap::new();
        for rating in ratings {
            maped_ratings
                .entry(rating.user_id)
                .or_insert_with(HashMap::new)
                .insert(rating.book_id, rating.score);
        }

        Ok(maped_ratings)
    }

    #[allow(clippy::type_complexity)]
    fn users_ratings(
        &self,
        users: &[Self::User],
    ) -> Result<maped_ratings!(Self::User => Self::Item), Error> {
        let ratings = Rating::belonging_to(users).load::<Rating>(&self.pg_conn)?;

        let mut maped_ratings = HashMap::new();
        for rating in ratings {
            maped_ratings
                .entry(rating.user_id)
                .or_insert_with(HashMap::new)
                .insert(rating.book_id, rating.score);
        }

        Ok(maped_ratings)
    }

    #[allow(clippy::type_complexity)]
    fn users_ratings_except(
        &self,
        user: &Self::User,
    ) -> Result<maped_ratings!(Self::User => Self::Item), Error> {
        let ratings = ratings::table
            .filter(ratings::user_id.ne(user.id))
            .load::<Rating>(&self.pg_conn)?;

        let mut maped_ratings = HashMap::new();
        for rating in ratings {
            maped_ratings
                .entry(rating.user_id)
                .or_insert_with(HashMap::new)
                .insert(rating.book_id, rating.score);
        }

        Ok(maped_ratings)
    }

    fn users_means(&self, users: &[Self::User]) -> Result<means!(Self::User), Error> {
        let means = Mean::belonging_to(users).load::<Mean>(&self.pg_conn)?;

        let means_by_user = means
            .into_iter()
            .map(|mean| (mean.user_id, mean.val))
            .collect();

        Ok(means_by_user)
    }

    fn score_range(&self) -> (f64, f64) {
        (0., 5.)
    }

    fn fields_for_users(&self) -> Vec<controller::Field> {
        vec![]
    }

    fn fields_for_items(&self) -> Vec<controller::Field> {
        vec![]
    }

    fn insert_user<'a>(
        &self,
        _: HashMap<&'a str, controller::Value>,
    ) -> controller::Result<Self::User> {
        Ok(insert_into(users::table)
            .default_values()
            .get_result(&self.pg_conn)?)
    }

    fn insert_item<'a>(
        &self,
        _: HashMap<&'a str, controller::Value>,
    ) -> controller::Result<Self::Item> {
        Ok(insert_into(books::table)
            .default_values()
            .get_result(&self.pg_conn)?)
    }

    fn insert_rating(
        &self,
        user_id: &eid!(Self::User),
        item_id: &eid!(Self::Item),
        score: f64,
    ) -> Result<Self::Rating, Error> {
        let users_who_rated = self.mongo_db.collection("users_who_rated");

        let query = doc! {
            "item_id": item_id,
            format!("scores.{}", user_id) : doc!{
                "$exists": true
            }
        };

        let rating = users_who_rated.find_one(query, None)?;
        if rating.is_some() {
            return Err(
                ErrorKind::InsertRatingFailed(user_id.to_string(), item_id.to_string()).into(),
            );
        }

        let update = doc! {
            "$set": doc!{
                format!("scores.{}",user_id): score
            }
        };

        let options = UpdateOptions::builder().upsert(true).build();
        users_who_rated.update_one(doc! { "item_id": item_id }, update, options)?;

        let new_rating = NewRating {
            user_id: *user_id,
            book_id: *item_id,
            score,
        };

        let psql_result = insert_into(ratings::table)
            .values(new_rating)
            .get_result(&self.pg_conn);

        match psql_result {
            Ok(rating) => Ok(rating),
            Err(e) => {
                let delete_doc = doc! {
                    "$unset": doc!{
                        format!("scores.{}", user_id): ""
                    }
                };

                users_who_rated.update_one(doc! { "item_id": item_id }, delete_doc, None)?;
                Err(e.into())
            }
        }
    }

    fn remove_rating(
        &self,
        user_id: &eid!(Self::User),
        item_id: &eid!(Self::Item),
    ) -> Result<Self::Rating, Error> {
        let users_who_rated = self.mongo_db.collection("users_who_rated");

        let delete_doc = doc! {
            "$unset": doc!{
                format!("scores.{}", user_id): ""
            }
        };

        let result = users_who_rated.update_one(doc! { "item_id": item_id }, delete_doc, None)?;
        if result.matched_count.is_zero() || result.modified_count.is_zero() {
            return Err(
                ErrorKind::RemoveRatingFailed(user_id.to_string(), item_id.to_string()).into(),
            );
        }

        let old_score: f64 = ratings::table
            .filter(ratings::user_id.eq(user_id))
            .filter(ratings::book_id.eq(item_id))
            .select(ratings::score)
            .first(&self.pg_conn)?;

        let psql_result = delete(ratings::table)
            .filter(ratings::user_id.eq(user_id))
            .filter(ratings::book_id.eq(item_id))
            .get_result(&self.pg_conn);

        match psql_result {
            Ok(rating) => Ok(rating),
            Err(e) => {
                let update_doc = doc! {
                    "$set": doc!{
                        format!("scores.{}",user_id): old_score
                    }
                };

                let options = UpdateOptions::builder().upsert(true).build();
                users_who_rated.update_one(doc! { "item_id": item_id }, update_doc, options)?;

                Err(e.into())
            }
        }
    }

    fn update_rating(
        &self,
        user_id: &eid!(Self::User),
        item_id: &eid!(Self::Item),
        score: f64,
    ) -> Result<Self::Rating, Error> {
        let users_who_rated = self.mongo_db.collection("users_who_rated");

        let update_doc = doc! {
            "$set": doc!{
                format!("scores.{}", user_id): score
            }
        };

        let result = users_who_rated.update_one(doc! { "item_id": item_id }, update_doc, None)?;
        if result.modified_count.is_zero() || result.matched_count.is_zero() {
            return Err(
                ErrorKind::UpdateRatingFailed(user_id.to_string(), item_id.to_string()).into(),
            );
        }

        let old_score: f64 = ratings::table
            .filter(ratings::user_id.eq(user_id))
            .filter(ratings::book_id.eq(item_id))
            .select(ratings::score)
            .first(&self.pg_conn)?;

        let psql_res = update(ratings::table)
            .filter(ratings::user_id.eq(user_id))
            .filter(ratings::book_id.eq(item_id))
            .set(ratings::score.eq(score))
            .get_result::<Rating>(&self.pg_conn);

        match psql_res {
            Ok(rating) => Ok(rating),
            Err(e) => {
                let update_doc = doc! {
                    "$set": doc! {
                        format!("score.{}", user_id): old_score
                    }
                };

                users_who_rated.update_one(doc! { "item_id": item_id }, update_doc, None)?;

                Err(e.into())
            }
        }
    }
}

#[cfg(feature = "test-controller")]
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Error;
    use controller::Entity;

    #[test]
    fn query_user_by_id() -> Result<(), Error> {
        let controller = ShelvesController::new()?;

        let users = controller.users_by(&SearchBy::id("2"))?;
        assert_eq!(users[0].get_id(), 2);

        Ok(())
    }

    #[test]
    fn query_item_by_id() -> Result<(), Error> {
        let controller = ShelvesController::new()?;

        let book = controller.items_by(&SearchBy::name("0"))?;
        assert_eq!(book[0].get_id(), 0);

        Ok(())
    }

    #[test]
    fn chunked_users() -> Result<(), Error> {
        let controller = ShelvesController::new()?;
        let mut chunk_iter = controller.users_by_chunks(80000);

        assert_eq!(80000, chunk_iter.next().unwrap().len());
        assert_eq!(80000, chunk_iter.next().unwrap().len());
        assert_eq!(80000, chunk_iter.next().unwrap().len());
        assert_eq!(38858, chunk_iter.next().unwrap().len());
        assert!(chunk_iter.next().is_none());

        Ok(())
    }
}
