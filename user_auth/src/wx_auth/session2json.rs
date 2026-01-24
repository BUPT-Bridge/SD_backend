use super::r#struct::{WxAuthError, WxAuthResponse, WxAuthServerConfig};
use reqwest;

/// 微信登录接口（异步）
///
/// 用于将用户的js_code转换为session_key和openid
///
/// 注意将
///
/// GET <BASE_URL>/sns/jscode2session?appid=APPID&secret=SECRET&js_code=JSCODE&grant_type=authorization_code>
pub async fn wx_auth_session_to_json(js_code: &str) -> Result<WxAuthResponse, WxAuthError> {
    let wx_config = WxAuthServerConfig::from_env();
    let url = format!(
        "{}/sns/jscode2session?appid={}&secret={}&js_code={}&grant_type=authorization_code",
        wx_config.base_url, wx_config.appid, wx_config.secret, js_code
    );
    let client = reqwest::Client::new();
    let response: WxAuthResponse = client.get(&url).send().await?.json().await?;
    match response.errcode {
        Some(-1) => Err(WxAuthError::WxSystemError),
        Some(40029) => Err(WxAuthError::CodeError),
        Some(40226) => Err(WxAuthError::UserBlockedError),
        Some(45011) => Err(WxAuthError::TooMuchRequestError),
        Some(_) => Err(WxAuthError::UnknownError(response.errcode.unwrap())),
        None => Ok(response),
    }
}
