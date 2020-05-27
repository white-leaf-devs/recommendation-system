use controller::Id;
use nom::{alt, char, delimited, tag, take_while, take_while1, tuple, IResult};
use recommend::distances::Method;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Database {
    Books,
    SimpleMovie,
    MovieLensSmall,
}

impl From<&str> for Database {
    fn from(s: &str) -> Self {
        match s {
            "books" => Self::Books,
            "simple-movie" => Self::SimpleMovie,
            "movie-lens-small" => Self::MovieLensSmall,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Index {
    Id(Id),
    Name(String),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Statement {
    Connect(Database),
    QueryUser(Index),
    QueryItem(Index),
    QueryRatings(Index),
    Distance(Index, Index, Method),
    KNN(usize, Index, Method),
    Predict(usize, Index, Index, Method),
}

#[inline(always)]
fn ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == ' '
}

#[inline(always)]
fn database(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-'
}

fn parse_ident(input: &str) -> IResult<&str, &str> {
    take_while1!(input, ident)
}

fn parse_database(input: &str) -> IResult<&str, &str> {
    take_while1!(input, database)
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

fn parse_index(input: &str) -> IResult<&str, Index> {
    let (input, index_type) = alt! {
        input,
        tag!("id") |
        tag!("name")
    }?;

    let (input, index) = delimited!(input, char!('('), parse_ident, char!(')'))?;

    let index = match index_type {
        "id" => Index::Id(index.into()),
        "name" => Index::Name(index.into()),
        _ => unreachable!(),
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
            let (input, database) = delimited!(input, char!('('), parse_database, char!(')'))?;
            (input, Statement::Connect(database.into()))
        }

        "query_user" => {
            let (input, index) = delimited!(input, char!('('), parse_index, char!(')'))?;
            (input, Statement::QueryUser(index))
        }

        "query_item" => {
            let (input, index) = delimited!(input, char!('('), parse_index, char!(')'))?;
            (input, Statement::QueryItem(index))
        }

        "query_ratings" => {
            let (input, index) = delimited!(input, char!('('), parse_index, char!(')'))?;
            (input, Statement::QueryRatings(index))
        }

        "distance" => {
            let (input, (index_a, _, index_b, _, method)) = delimited!(
                input,
                char!('('),
                tuple!(
                    parse_index,
                    parse_separator,
                    parse_index,
                    parse_separator,
                    parse_method
                ),
                char!(')')
            )?;

            (input, Statement::Distance(index_a, index_b, method))
        }

        "knn" => {
            let (input, (k, _, index, _, method)) = delimited!(
                input,
                char!('('),
                tuple!(
                    parse_number,
                    parse_separator,
                    parse_index,
                    parse_separator,
                    parse_method
                ),
                char!(')')
            )?;

            (input, Statement::KNN(k.parse().unwrap(), index, method))
        }

        "predict" => {
            let (input, (k, _, index_user, _, index_item, _, method)) = delimited!(
                input,
                char!('('),
                tuple!(
                    parse_number,
                    parse_separator,
                    parse_index,
                    parse_separator,
                    parse_index,
                    parse_separator,
                    parse_method
                ),
                char!(')')
            )?;

            (
                input,
                Statement::Predict(k.parse().unwrap(), index_user, index_item, method),
            )
        }

        function => todo!("Function {}", function),
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
        let parsed = parse_index("id(323)");
        let expected = ("", Index::Id(323.into()));

        assert_eq!(parsed, Ok(expected));

        let parsed = parse_index("name(Patrick C)");
        let expected = ("", Index::Name("Patrick C".into()));

        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn connect_statement() {
        let parsed = parse_statement("connect(books)");
        let expected = ("", Statement::Connect(Database::Books));

        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn query_user_statement() {
        let parsed = parse_statement("query_user(id(3))");
        let expected = ("", Statement::QueryUser(Index::Id(3.into())));

        assert_eq!(parsed, Ok(expected));

        let parsed = parse_statement("query_user(name(Patrick C))");
        let expected = ("", Statement::QueryUser(Index::Name("Patrick C".into())));

        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn query_item_statement() {
        let parsed = parse_statement("query_item(id(bx32a))");
        let expected = ("", Statement::QueryItem(Index::Id("bx32a".into())));

        assert_eq!(parsed, Ok(expected));

        let parsed = parse_statement("query_item(name(The Great Gatsby))");
        let expected = (
            "",
            Statement::QueryItem(Index::Name("The Great Gatsby".into())),
        );

        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn query_ratings_statement() {
        let parsed = parse_statement("query_ratings(id(12345))");
        let expected = ("", Statement::QueryRatings(Index::Id(12345.into())));

        assert_eq!(parsed, Ok(expected));

        let parsed = parse_statement("query_ratings(name(Patrick C))");
        let expected = ("", Statement::QueryRatings(Index::Name("Patrick C".into())));

        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn distance_statement() {
        let parsed = parse_statement("distance(id(32a), id(32b), euclidean)");
        let expected = (
            "",
            Statement::Distance(
                Index::Id("32a".into()),
                Index::Id("32b".into()),
                Method::Euclidean,
            ),
        );

        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn knn_statement() {
        let parsed = parse_statement("knn(id(324x), 4, minkowski(3))");
        let expected = (
            "",
            Statement::KNN(4, Index::Id("324x".into()), Method::Minkowski(3)),
        );

        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn parse_invalid_line() {
        let parsed = parse_line("query_user(id());");
        assert!(parsed.is_none());
    }

    #[test]
    fn parse_valid_line() {
        let parsed = parse_line("knn(name(Patrick C), 5, cosine)");
        assert_eq!(
            parsed,
            Some(Statement::KNN(
                5,
                Index::Name("Patrick C".into()),
                Method::CosineSimilarity
            ))
        );
    }
}
