use dotenvy::dotenv;
use user_auth::wx_auth::*;

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_wx_auth_correct() {
        dotenv().ok();
        if std::env::var("SERVER_WX_BASEURL").is_ok() {
            let js_code = std::env::var("TEST_SERVER_WX_JS_CODE").unwrap();
            let result = wx_auth_session_to_json(&js_code).await;
            assert!(result.is_ok());
        } else {
            let js_code = std::env::var("TEST_SERVER_WX_JS_CODE").unwrap();
            let result = wx_auth_session_to_json(&js_code).await;
            assert!(result.is_err());
        }
    }
}
