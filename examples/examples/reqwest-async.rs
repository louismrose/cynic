//! An example that shows how to send a GraphQL operation and deserialize
//! a GraphQL response using the cynic reqwest integration
//!
//! This example requires the `http-reqwest` feature to be active

mod schema {
    cynic::use_schema!("../schemas/starwars.schema.graphql");
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "../schemas/starwars.schema.graphql")]
struct Film {
    title: Option<String>,
    director: Option<String>,
}

#[derive(cynic::QueryVariables)]
struct FilmArguments {
    id: Option<cynic::Id>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "../schemas/starwars.schema.graphql",
    graphql_type = "Root",
    variables = "FilmArguments"
)]
struct FilmDirectorQuery {
    #[arguments(id: $id)]
    film: Option<Film>,
}

#[tokio::main]
async fn main() {
    match run_query().await.data {
        Some(FilmDirectorQuery { film: Some(film) }) => {
            println!("{:?} was directed by {:?}", film.title, film.director)
        }
        _ => {
            println!("No film found");
        }
    }
}

async fn run_query() -> cynic::GraphQlResponse<FilmDirectorQuery> {
    use cynic::http::ReqwestExt;

    let query = build_query();

    reqwest::Client::new()
        .post("https://swapi-graphql.netlify.app/.netlify/functions/index")
        .run_graphql(query)
        .await
        .unwrap()
}

fn build_query() -> cynic::Operation<FilmDirectorQuery, FilmArguments> {
    use cynic::QueryBuilder;

    FilmDirectorQuery::build(FilmArguments {
        id: Some("ZmlsbXM6MQ==".into()),
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn snapshot_test_query() {
        // Running a snapshot test of the query building functionality as that gives us
        // a place to copy and paste the actual GQL we're using for running elsewhere,
        // and also helps ensure we don't change queries by mistake

        let query = build_query();

        insta::assert_snapshot!(query.query);
    }

    #[tokio::test]
    async fn test_running_query() {
        let result = run_query().await;
        if result.errors.is_some() {
            assert_eq!(result.errors.unwrap().len(), 0);
        }
        insta::assert_debug_snapshot!(result.data);
    }
}
