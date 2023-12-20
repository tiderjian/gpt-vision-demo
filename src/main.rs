use axum::{Json, extract::DefaultBodyLimit, http::StatusCode, routing::{get, post}, Router, response::{Html, IntoResponse, Response}};
use serde::{Deserialize, Serialize};
use serde_json::json;
use async_openai::{
    types::{
        ChatCompletionRequestSystemMessageArgs,ChatCompletionRequestMessageContentPartImageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,ChatCompletionRequestMessageContentPartTextArgs,
        ChatCompletionRequestMessageContentPart
    },
    Client,
    config::AzureConfig
};
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let app = Router::new().route("/", get(image))
    .route("/ai", post(send_ai))
    .layer(DefaultBodyLimit::max(1024 * 1024 * 1000));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MyData {
    // 在此定义JSON中需要的字段
    prompt: String,
    image: Vec<String>,
}


async fn send_ai(Json(payload): Json<MyData>) -> impl IntoResponse{
    let config = AzureConfig::new()
        .with_api_base(env::var("API_BASE").unwrap())
        .with_api_version(env::var("API_VERSION").unwrap())
        .with_deployment_id(env::var("DEPLOYMENT_ID").unwrap())
        .with_api_key(env::var("AZURE_OPENAI_API_KEY").unwrap());

    let mut user_messages: Vec<ChatCompletionRequestMessageContentPart> = vec!(ChatCompletionRequestMessageContentPartTextArgs::default().text(payload.prompt).build().unwrap().into());

    let mut user_images: Vec<ChatCompletionRequestMessageContentPart> = payload.image.into_iter().map(|x| ChatCompletionRequestMessageContentPartImageArgs::default().image_url(x).build().unwrap().into()).collect();

    user_messages.append(&mut user_images);

    let client = Client::with_config(config);
    let request = CreateChatCompletionRequestArgs::default()
    .max_tokens(4096u16)
    .model("gpt-4")
    .messages([
        ChatCompletionRequestSystemMessageArgs::default()
            .content("You are a helpful assistant.")
            .build().unwrap()
            .into(),
        ChatCompletionRequestUserMessageArgs::default()
            .content(user_messages)
            .build().unwrap()
            .into()
    ])
    .build().unwrap();

    println!("{}", serde_json::to_string(&request).unwrap());

    let response = client.chat().create(request).await.unwrap();

    println!("{:#?}", response);
    
    // 考虑到安全和错误处理，应当在实际应用中添加适量的错误处理机制
    (StatusCode::OK, Json(json!({"message": response.choices[0].message.content})))
}

async fn image() -> Response{
    Html(std::include_str!("./index.html")).into_response()
}