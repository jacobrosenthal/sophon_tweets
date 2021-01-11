use reqwest::multipart;
use reqwest_oauth1::OAuthClientProvider;
use tokio_compat_02::FutureExt;

pub async fn send(tweet: String) -> Result<(), TwitterError> {
    let args: Vec<String> = std::env::args().collect();
    let consumer_key = args[1].clone();
    let consumer_secret_key = args[2].clone();
    let access_token = args[3].clone();
    let secret_access_token = args[4].clone();

    let secrets = reqwest_oauth1::Secrets::new(consumer_key, consumer_secret_key)
        .token(access_token, secret_access_token);

    let endpoint = "https://api.twitter.com/1.1/statuses/update.json";

    let content = multipart::Form::new().text("status", tweet);

    let response = reqwest::Client::new()
        // enable OAuth1 request
        .oauth1(secrets)
        .post(endpoint)
        .multipart(content)
        .send()
        .compat()
        .await
        .unwrap();

    //todo, TwitterApiError here. but duplicate tweets, like if no planet totals have changed, will error, so just print
    if response.status() != 200 {
        dbg!(response.text().await.unwrap());
    }
    Ok(())
}

#[derive(Debug)]
pub enum TwitterError {
    Internal,
    HttpError,
    OAuth,
    TwitterUrl,
}

impl From<std::io::Error> for TwitterError {
    fn from(_err: std::io::Error) -> Self {
        TwitterError::Internal
    }
}

impl From<url::ParseError> for TwitterError {
    fn from(_err: url::ParseError) -> Self {
        TwitterError::TwitterUrl
    }
}

impl From<reqwest::Error> for TwitterError {
    fn from(_err: reqwest::Error) -> Self {
        TwitterError::HttpError
    }
}

impl From<reqwest_oauth1::Error> for TwitterError {
    fn from(_err: reqwest_oauth1::Error) -> Self {
        TwitterError::OAuth
    }
}
