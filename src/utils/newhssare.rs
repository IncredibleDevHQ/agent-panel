#[derive(Deserialize)]
struct QueryParams {
    query: String,
    repo_name: String,
}

async fn retrieve_answer(Query(params): Query<QueryParams>) -> Result<Json<String>, (StatusCode, String)> {
    // Implement your logic here. For now, we're just echoing back the query.
    let response = format!("Query: {}, Repo Name: {}", params.query, params.repo_name);
    Ok(Json(response))
}