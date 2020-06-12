pub mod basics;

use crate::parser::basics::{parse_ident, parse_number, parse_separator, parse_string};
use controller::SearchBy;
use engine::distances::items::Method as ItemMethod;
use engine::distances::users::Method as UserMethod;
use nom::combinator::opt;
use nom::sequence::{delimited, tuple};
use nom::{branch::alt, character::complete::char};
use nom::{bytes::complete::tag, IResult};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Database {
    Books,
    Shelves,
    SimpleMovie,
    MovieLens,
    MovieLensSmall,
}

impl From<&str> for Database {
    fn from(s: &str) -> Self {
        match s {
            "books" => Self::Books,
            "shelves" => Self::Shelves,
            "simple-movie" => Self::SimpleMovie,
            "movie-lens" => Self::MovieLens,
            "movie-lens-small" => Self::MovieLensSmall,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Statement {
    Connect(Database),
    QueryUser(SearchBy),
    QueryItem(SearchBy),
    QueryRatings(SearchBy),
    UserDistance(SearchBy, SearchBy, UserMethod),
    ItemDistance(SearchBy, SearchBy, ItemMethod),
    UserKnn(usize, SearchBy, UserMethod, Option<usize>),
    UserPredict(usize, SearchBy, SearchBy, UserMethod, Option<usize>),
    ItemPredict(SearchBy, SearchBy, ItemMethod, Option<usize>),

    // Specific for similarity matrix
    EnterSimMatrix(usize, usize, usize, ItemMethod),
    SimMatrixGet(SearchBy, SearchBy),
    SimMatrixMoveTo(usize, usize),
}

fn parse_user_method(input: &str) -> IResult<&str, UserMethod> {
    let (input, method) = alt((
        tag("cosine"),
        tag("pearson_c"),
        tag("pearson_a"),
        tag("euclidean"),
        tag("manhattan"),
        tag("minkowski"),
        tag("jacc_index"),
        tag("jacc_distance"),
    ))(input)?;

    let (input, method) = match method {
        "cosine" => (input, UserMethod::CosineSimilarity),
        "pearson_c" => (input, UserMethod::PearsonCorrelation),
        "pearson_a" => (input, UserMethod::PearsonApproximation),
        "euclidean" => (input, UserMethod::Euclidean),
        "manhattan" => (input, UserMethod::Manhattan),
        "minkowski" => {
            let (input, number) = delimited(char('('), parse_number, char(')'))(input)?;
            (input, UserMethod::Minkowski(number as usize))
        }
        "jacc_index" => (input, UserMethod::JaccardIndex),
        "jacc_distance" => (input, UserMethod::JaccardDistance),
        _ => unreachable!(),
    };

    Ok((input, method))
}

fn parse_item_method(input: &str) -> IResult<&str, ItemMethod> {
    let (input, method) = alt((tag("slope_one"), tag("adj_cosine")))(input)?;

    let (input, method) = match method {
        "slope_one" => (input, ItemMethod::SlopeOne),
        "adj_cosine" => (input, ItemMethod::AdjCosine),
        _ => unreachable!(),
    };

    Ok((input, method))
}

fn parse_searchby(input: &str) -> IResult<&str, SearchBy> {
    let (input, ident) = parse_ident(input)?;
    let (input, value) = delimited(char('('), parse_string, char(')'))(input)?;

    let index = match ident {
        "id" => SearchBy::id(value),
        "name" => SearchBy::name(value),
        custom => SearchBy::custom(custom, value),
    };

    Ok((input, index))
}

fn parse_statement(input: &str) -> IResult<&str, Statement> {
    let (input, statement_type) = alt((
        tag("connect"),
        tag("user_knn"),
        tag("get"),
        tag("move_to"),
        tag("query_user"),
        tag("query_item"),
        tag("query_ratings"),
        tag("user_distance"),
        tag("item_distance"),
        tag("enter_sim_matrix"),
        tag("user_based_predict"),
        tag("item_based_predict"),
    ))(input)?;

    let (input, statement) = match statement_type {
        "connect" => {
            let (input, database) = delimited(char('('), parse_ident, char(')'))(input)?;
            (input, Statement::Connect(database.into()))
        }

        "query_user" => {
            let (input, user_searchby) = delimited(char('('), parse_searchby, char(')'))(input)?;
            (input, Statement::QueryUser(user_searchby))
        }

        "query_item" => {
            let (input, item_searchby) = delimited(char('('), parse_searchby, char(')'))(input)?;
            (input, Statement::QueryItem(item_searchby))
        }

        "query_ratings" => {
            let (input, user_searchby) = delimited(char('('), parse_searchby, char(')'))(input)?;
            (input, Statement::QueryRatings(user_searchby))
        }

        "user_distance" => {
            let (input, (user_a_searchby, _, user_b_searchby, _, user_method)) =
                delimited(
                    char('('),
                    tuple((
                        parse_searchby,
                        parse_separator,
                        parse_searchby,
                        parse_separator,
                        parse_user_method,
                    )),
                    char(')'),
                )(input)?;

            (
                input,
                Statement::UserDistance(user_a_searchby, user_b_searchby, user_method),
            )
        }

        "item_distance" => {
            let (input, (item_a_searchby, _, item_b_searchby, _, item_method)) =
                delimited(
                    char('('),
                    tuple((
                        parse_searchby,
                        parse_separator,
                        parse_searchby,
                        parse_separator,
                        parse_item_method,
                    )),
                    char(')'),
                )(input)?;

            (
                input,
                Statement::ItemDistance(item_a_searchby, item_b_searchby, item_method),
            )
        }

        "user_knn" => {
            let (input, (k, _, user_searchby, _, user_method, chunks_opt)) = delimited(
                char('('),
                tuple((
                    parse_number,
                    parse_separator,
                    parse_searchby,
                    parse_separator,
                    parse_user_method,
                    opt(tuple((parse_separator, parse_number))),
                )),
                char(')'),
            )(input)?;

            (
                input,
                Statement::UserKnn(
                    k as usize,
                    user_searchby,
                    user_method,
                    chunks_opt.map(|(_, chunk_size)| chunk_size as usize),
                ),
            )
        }

        "enter_sim_matrix" => {
            let (input, (m, _, n, _, threshold, _, item_method)) = delimited(
                char('('),
                tuple((
                    parse_number,
                    parse_separator,
                    parse_number,
                    parse_separator,
                    parse_number,
                    parse_separator,
                    parse_item_method,
                )),
                char(')'),
            )(input)?;

            (
                input,
                Statement::EnterSimMatrix(m as usize, n as usize, threshold as usize, item_method),
            )
        }

        "get" => {
            let (input, (item_a_searchby, _, item_b_searchby)) = delimited(
                char('('),
                tuple((parse_searchby, parse_separator, parse_searchby)),
                char(')'),
            )(input)?;

            (
                input,
                Statement::SimMatrixGet(item_a_searchby, item_b_searchby),
            )
        }

        "move_to" => {
            let (input, (i, _, j)) = delimited(
                char('('),
                tuple((parse_number, parse_separator, parse_number)),
                char(')'),
            )(input)?;

            (input, Statement::SimMatrixMoveTo(i as usize, j as usize))
        }

        "user_based_predict" => {
            let (input, (k, _, user_searchby, _, item_searchby, _, user_method, chunks_opt)) =
                delimited(
                    char('('),
                    tuple((
                        parse_number,
                        parse_separator,
                        parse_searchby,
                        parse_separator,
                        parse_searchby,
                        parse_separator,
                        parse_user_method,
                        opt(tuple((parse_separator, parse_number))),
                    )),
                    char(')'),
                )(input)?;

            (
                input,
                Statement::UserPredict(
                    k as usize,
                    user_searchby,
                    item_searchby,
                    user_method,
                    chunks_opt.map(|(_, chunk_size)| chunk_size as usize),
                ),
            )
        }

        "item_based_predict" => {
            let (input, (user_searchby, _, item_searchby, _, item_method, chunks_opt)) =
                delimited(
                    char('('),
                    tuple((
                        parse_searchby,
                        parse_separator,
                        parse_searchby,
                        parse_separator,
                        parse_item_method,
                        opt(tuple((parse_separator, parse_number))),
                    )),
                    char(')'),
                )(input)?;

            (
                input,
                Statement::ItemPredict(
                    user_searchby,
                    item_searchby,
                    item_method,
                    chunks_opt.map(|(_, chunk_size)| chunk_size as usize),
                ),
            )
        }

        function => unimplemented!("Unimplemented parser for {}", function),
    };

    Ok((input, statement))
}

pub fn parse_line(input: &str) -> Option<Statement> {
    let input = input.trim();
    let (rest, statement) = parse_statement(input).ok()?;

    if rest.is_empty() {
        Some(statement)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_tests() {
        let parsed = parse_searchby("id('323')");
        let expected = ("", SearchBy::id("323"));

        assert_eq!(parsed, Ok(expected));

        let parsed = parse_searchby("name('Patrick C')");
        let expected = ("", SearchBy::name("Patrick C"));

        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn connect_statement() {
        let parsed = parse_statement("connect(simple-movie)");
        let expected = ("", Statement::Connect(Database::SimpleMovie));

        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn query_user_statement() {
        let parsed = parse_statement("query_user(id('3'))");
        let expected = ("", Statement::QueryUser(SearchBy::id("3")));

        assert_eq!(parsed, Ok(expected));

        let parsed = parse_statement("query_user(name('Patrick C'))");
        let expected = ("", Statement::QueryUser(SearchBy::name("Patrick C")));

        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn query_item_statement() {
        let parsed = parse_statement("query_item(id('bx32a'))");
        let expected = ("", Statement::QueryItem(SearchBy::id("bx32a")));

        assert_eq!(parsed, Ok(expected));

        let parsed = parse_statement("query_item(name('The Great Gatsby (1925)'))");
        let expected = (
            "",
            Statement::QueryItem(SearchBy::name("The Great Gatsby (1925)")),
        );

        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn query_ratings_statement() {
        let parsed = parse_statement("query_ratings(id('12345'))");
        let expected = ("", Statement::QueryRatings(SearchBy::id("12345")));

        assert_eq!(parsed, Ok(expected));

        let parsed = parse_statement("query_ratings(name('Patrick C'))");
        let expected = ("", Statement::QueryRatings(SearchBy::name("Patrick C")));

        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn user_distance_statement() {
        let parsed = parse_statement("user_distance(id('32a'), id('32b'), euclidean)");
        let expected = (
            "",
            Statement::UserDistance(
                SearchBy::id("32a"),
                SearchBy::id("32b"),
                UserMethod::Euclidean,
            ),
        );

        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn item_distance_statement() {
        let parsed = parse_statement("item_distance(id('32a'), id('32b'), adj_cosine)");
        let expected = (
            "",
            Statement::ItemDistance(
                SearchBy::id("32a"),
                SearchBy::id("32b"),
                ItemMethod::AdjCosine,
            ),
        );

        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn user_knn_statement() {
        let parsed = parse_statement("user_knn(4, id('324x'), minkowski(3))");
        let expected = (
            "",
            Statement::UserKnn(4, SearchBy::id("324x"), UserMethod::Minkowski(3), None),
        );

        assert_eq!(parsed, Ok(expected));

        let parsed = parse_statement("user_knn(4, id('324x'), minkowski(3), 10)");
        let expected = (
            "",
            Statement::UserKnn(4, SearchBy::id("324x"), UserMethod::Minkowski(3), Some(10)),
        );

        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn user_predict_statement() {
        let parsed =
            parse_statement("user_based_predict(4, id('324x'), name('Alien'), minkowski(3))");
        let expected = (
            "",
            Statement::UserPredict(
                4,
                SearchBy::id("324x"),
                SearchBy::name("Alien"),
                UserMethod::Minkowski(3),
                None,
            ),
        );

        assert_eq!(parsed, Ok(expected));

        let parsed =
            parse_statement("user_based_predict(4, id('324x'), name('Alien'), minkowski(3), 100)");
        let expected = (
            "",
            Statement::UserPredict(
                4,
                SearchBy::id("324x"),
                SearchBy::name("Alien"),
                UserMethod::Minkowski(3),
                Some(100),
            ),
        );

        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn item_predict_statement() {
        let parsed = parse_statement("item_based_predict(id('324x'), name('Alien'), adj_cosine)");
        let expected = (
            "",
            Statement::ItemPredict(
                SearchBy::id("324x"),
                SearchBy::name("Alien"),
                ItemMethod::AdjCosine,
                None,
            ),
        );

        assert_eq!(parsed, Ok(expected));

        let parsed =
            parse_statement("item_based_predict(id('324x'), name('Alien'), adj_cosine, 100)");
        let expected = (
            "",
            Statement::ItemPredict(
                SearchBy::id("324x"),
                SearchBy::name("Alien"),
                ItemMethod::AdjCosine,
                Some(100),
            ),
        );

        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn enter_sim_matrix_statement() {
        let parsed = parse_statement("enter_sim_matrix(100, 100, 50, adj_cosine)");
        let expected = (
            "",
            Statement::EnterSimMatrix(100, 100, 50, ItemMethod::AdjCosine),
        );

        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn sim_matrix_get_statement() {
        let parsed = parse_statement("get(id('10'), name('Alien'))");
        let expected = (
            "",
            Statement::SimMatrixGet(SearchBy::id("10"), SearchBy::name("Alien")),
        );

        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn sim_matrix_move_to_statement() {
        let parsed = parse_statement("move_to(10, 1)");
        let expected = ("", Statement::SimMatrixMoveTo(10, 1));

        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn parse_invalid_line() {
        let parsed = parse_line("query_user(id())xx");
        assert!(parsed.is_none());
    }

    #[test]
    fn parse_valid_line() {
        let parsed = parse_line("user_knn(5, name('Patrick C'), cosine)");
        assert_eq!(
            parsed,
            Some(Statement::UserKnn(
                5,
                SearchBy::name("Patrick C"),
                UserMethod::CosineSimilarity,
                None
            ))
        );
    }
}
