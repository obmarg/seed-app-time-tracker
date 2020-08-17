use seed::{prelude::*};

use cynic;

pub type Result<T> = std::result::Result<T, GraphQLError>;

pub async fn send_query<'a, ResponseData: 'a, Root: cynic::QueryRoot>(
    selection_set: cynic::SelectionSet<'a, ResponseData, Root>
) -> Result<ResponseData> {
    let query = cynic::Query::new(selection_set);

    let graphql_response = 
        Request::new("https://time-tracker.eu-central-1.aws.cloud.dgraph.io/graphql")
            .method(Method::Post)
            .json(&query)?
            .fetch()
            .await?
            .check_status()?
            .json()
            .await?;

    let response_data = query.decode_response(graphql_response)?;
    if let Some(errors) = response_data.errors {
        Err(errors)?
    }
    Ok(response_data.data.expect("response data"))
}

// ------ Error ------

#[derive(Debug)]
pub enum GraphQLError {
    FetchError(FetchError),
    ResponseErrors(Vec<cynic::GraphQLError>),
    DecodeError(cynic::DecodeError)
}

impl From<FetchError> for GraphQLError {
    fn from(fetch_error: FetchError) -> Self {
        Self::FetchError(fetch_error)
    }
}

impl From<Vec<cynic::GraphQLError>> for GraphQLError {
    fn from(response_errors: Vec<cynic::GraphQLError>) -> Self {
        Self::ResponseErrors(response_errors)
    }
}

impl From<cynic::DecodeError> for GraphQLError {
    fn from(decode_error: cynic::DecodeError) -> Self {
        Self::DecodeError(decode_error)
    }
}

// ------ ------
// GraphQL items
// ------ ------

#[cynic::query_module(
    schema_path = "schema.graphql",
    query_module = "query_dsl",
)]
pub mod queries {
    use super::{query_dsl, types::*};

    /*
    ```graphql
    query ClientsWithProjects {
        queryClient {
            id
            name
            projects {
                id
                name
            }
        }
    }
    ```
    */
    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Query")]
    pub struct ClientsWithProjects {
        pub query_client: Option<Vec<Option<Client>>>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Client")]
    pub struct Client {
        pub id: String,
        pub name: String,
        pub projects: Vec<Project>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Project")]
    pub struct Project {
        pub id: String,
        pub name: String,
    }

}

mod types {
    #[derive(cynic::Scalar, Debug)]
    pub struct DateTime(String);
}

mod query_dsl {
    use super::types::*;
    cynic::query_dsl!("schema.graphql");
}
