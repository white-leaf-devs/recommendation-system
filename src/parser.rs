use controller::SearchBy;
use engine::distances::users::Method;
use nom::{alt, char, delimited, opt, tag, take_till1, take_while, take_while1, tuple, IResult};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Database {
    Books,
    SimpleMovie,
    MovieLens,
    MovieLensSmall,
}

impl From<&str> for Database {
    fn from(s: &str) -> Self {
        match s {
            "books" => Self::Books,
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
    Distance(SearchBy, SearchBy, Method),
    KNN(usize, SearchBy, Method, Option<usize>),
    Predict(usize, SearchBy, SearchBy, Method, Option<usize>),
}

fn parse_ident(input: &str) -> IResult<&str, &str> {
    take_while1!(input, |c: char| c.is_alphanumeric() || c == '_' || c == '-')
}

fn parse_string(input: &str) -> IResult<&str, &str> {
    delimited!(
        input,
        char!('\''),
        take_till1!(|c: char| c == '\''),
        char!('\'')
    )
}

fn parse_number(input: &str) -> IResult<&str, &str> {
    take_while1!(input, |c: char| c.is_ascii_digit())
}

fn parse_separator(input: &str) -> IResult<&str, &str> {
    delimited!(
        input,
        take_while!(|c: char| c == ' '),
        tag!(","),
        take_while!(|c: char| c == ' ')
    )
}

fn parse_method(input: &str) -> IResult<&str, Method> {
    let (input, method) = alt! {
        input,
        tag!("cosine")        |
        tag!("pearson_c")     |
        tag!("pearson_a")     |
        tag!("euclidean")     |
        tag!("manhattan")     |
        tag!("minkowski")     |
        tag!("jacc_index")    |
        tag!("jacc_distance")
    }?;

    let (input, method) = match method {
        "cosine" => (input, Method::CosineSimilarity),
        "pearson_c" => (input, Method::PearsonCorrelation),
        "pearson_a" => (input, Method::PearsonApproximation),
        "euclidean" => (input, Method::Euclidean),
        "manhattan" => (input, Method::Manhattan),
        "jacc_index" => (input, Method::JaccardIndex),
        "jacc_distance" => (input, Method::JaccardDistance),
        "minkowski" => {
            let (input, number) = delimited!(input, char!('('), parse_number, char!(')'))?;
            (
                input,
                Method::Minkowski(number.parse().expect("Parsing a number should not fail")),
            )
        }
        _ => unreachable!(),
    };

    Ok((input, method))
}

fn parse_searchby(input: &str) -> IResult<&str, SearchBy> {
    let (input, ident) = parse_ident(input)?;
    let (input, value) = delimited!(input, char!('('), parse_string, char!(')'))?;

    let index = match ident {
        "id" => SearchBy::id(value),
        "name" => SearchBy::name(value),
        custom => SearchBy::custom(custom, value),
    };

    Ok((input, index))
}

fn parse_statement(input: &str) -> IResult<&str, Statement> {
    let (input, statement_type) = alt! {
        input,
        tag!("knn")        |
        tag!("connect")    |
        tag!("predict")    |
        tag!("distance")   |
        tag!("query_user") |
        tag!("query_item") |
        tag!("query_ratings")
    }?;

    let (input, statement) = match statement_type {
        "connect" => {
            let (input, database) = delimited!(input, char!('('), parse_ident, char!(')'))?;
            (input, Statement::Connect(database.into()))
        }

        "query_user" => {
            let (input, index) = delimited!(input, char!('('), parse_searchby, char!(')'))?;
            (input, Statement::QueryUser(index))
        }

        "query_item" => {
            let (input, index) = delimited!(input, char!('('), parse_searchby, char!(')'))?;
            (input, Statement::QueryItem(index))
        }

        "query_ratings" => {
            let (input, index) = delimited!(input, char!('('), parse_searchby, char!(')'))?;
            (input, Statement::QueryRatings(index))
        }

        "distance" => {
            let (input, (index_a, _, index_b, _, method)) = delimited!(
                input,
                char!('('),
                tuple!(
                    parse_searchby,
                    parse_separator,
                    parse_searchby,
                    parse_separator,
                    parse_method
                ),
                char!(')')
            )?;

            (input, Statement::Distance(index_a, index_b, method))
        }

        "knn" => {
            let (input, (k, _, index, _, method, chunks_opt)) = delimited!(
                input,
                char!('('),
                tuple!(
                    parse_number,
                    parse_separator,
                    parse_searchby,
                    parse_separator,
                    parse_method,
                    opt!(tuple!(parse_separator, parse_number))
                ),
                char!(')')
            )?;

            match chunks_opt {
                Some((_, chunk_size)) => (
                    input,
                    Statement::KNN(
                        k.parse().unwrap(),
                        index,
                        method,
                        Some(chunk_size.parse().unwrap()),
                    ),
                ),
                None => (
                    input,
                    Statement::KNN(k.parse().unwrap(), index, method, None),
                ),
            }
        }

        "predict" => {
            let (input, (k, _, index_user, _, index_item, _, method, chunks_opt)) = delimited!(
                input,
                char!('('),
                tuple!(
                    parse_number,
                    parse_separator,
                    parse_searchby,
                    parse_separator,
                    parse_searchby,
                    parse_separator,
                    parse_method,
                    opt!(tuple!(parse_separator, parse_number))
                ),
                char!(')')
            )?;

            match chunks_opt {
                Some((_, chunk_size)) => (
                    input,
                    Statement::Predict(
                        k.parse().unwrap(),
                        index_user,
                        index_item,
                        method,
                        Some(chunk_size.parse().unwrap()),
                    ),
                ),
                None => (
                    input,
                    Statement::Predict(k.parse().unwrap(), index_user, index_item, method, None),
                ),
            }
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
    fn distance_statement() {
        let parsed = parse_statement("distance(id('32a'), id('32b'), euclidean)");
        let expected = (
            "",
            Statement::Distance(SearchBy::id("32a"), SearchBy::id("32b"), Method::Euclidean),
        );

        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn knn_statement() {
        let parsed = parse_statement("knn(4, id('324x'), minkowski(3))");
        let expected = (
            "",
            Statement::KNN(4, SearchBy::id("324x"), Method::Minkowski(3), None),
        );

        assert_eq!(parsed, Ok(expected));

        let parsed = parse_statement("knn(4, id('324x'), minkowski(3), 10)");
        let expected = (
            "",
            Statement::KNN(4, SearchBy::id("324x"), Method::Minkowski(3), Some(10)),
        );

        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn predict_statement() {
        let parsed = parse_statement("predict(4, id('324x'), name('Alien'), minkowski(3))");
        let expected = (
            "",
            Statement::Predict(
                4,
                SearchBy::id("324x"),
                SearchBy::name("Alien"),
                Method::Minkowski(3),
                None,
            ),
        );

        assert_eq!(parsed, Ok(expected));

        let parsed = parse_statement("predict(4, id('324x'), name('Alien'), minkowski(3), 100)");
        let expected = (
            "",
            Statement::Predict(
                4,
                SearchBy::id("324x"),
                SearchBy::name("Alien"),
                Method::Minkowski(3),
                Some(100),
            ),
        );

        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn parse_invalid_line() {
        let parsed = parse_line("query_user(id())xx");
        assert!(parsed.is_none());
    }

    #[test]
    fn parse_valid_line() {
        let parsed = parse_line("knn(5, name('Patrick C'), cosine)");
        assert_eq!(
            parsed,
            Some(Statement::KNN(
                5,
                SearchBy::name("Patrick C"),
                Method::CosineSimilarity,
                None
            ))
        );
    }
}
