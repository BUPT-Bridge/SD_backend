use dotenv::dotenv;
use user_auth::wx_auth::*;

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_wx_auth_correct() {
        dotenv().ok();
        let result = wx_auth_session_to_json("XXXXXXXX").await;
        assert!(result.is_err());
    }
}
