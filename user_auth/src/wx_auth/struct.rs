use serde::Deserialize;

/// 微信授权服务器配置
/// 包括 appid、secret和base_url
pub struct WxAuthServerConfig {
    pub appid: String,
    pub secret: String,
    pub base_url: String,
}

impl WxAuthServerConfig {
    pub fn new(appid: String, secret: String, base_url: String) -> Self {
        WxAuthServerConfig {
            appid,
            secret,
            base_url,
        }
    }

    /// 从环境变量中读取配置
    ///
    /// `SERVER_WX_APPID`和`SERVER_WX_SECRET`环境变量必须存在
    pub fn from_env() -> Self {
        let appid = std::env::var("SERVER_WX_APPID").expect("SERVER_WX_APPID not found");
        let secret = std::env::var("SERVER_WX_SECRET").expect("SERVER_WX_SECRET not found");
        let base_url = std::env::var("SERVER_WX_BASEURL")
            .unwrap_or_else(|_| "https://api.weixin.qq.com".to_string());
        WxAuthServerConfig::new(appid, secret, base_url)
    }
}

/// 微信授权错误
#[derive(Debug)]
pub enum WxAuthError {
    WxSystemError,
    CodeError,
    UserBlockedError,
    TooMuchRequestError,
    UnknownError(i32),
    NetworkError(reqwest::Error),
}

impl From<reqwest::Error> for WxAuthError {
    fn from(error: reqwest::Error) -> Self {
        WxAuthError::NetworkError(error)
    }
}

/// 微信登录接口返回结构体
///
/// |参数名|	类型|	说明|
/// |------|------|------|
/// |session_key|string|会话密钥|
/// |unionid|string|用户在开放平台的唯一标识符，若当前小程序已绑定到微信开放平台帐号下会返回，详见 UnionID 机制说明。|
/// |openid|string|用户唯一标识|
/// |errcode|number|错误码，请求失败时返回|
/// |errmsg|string|错误信息，请求失败时返回|
#[derive(Debug, Clone, Deserialize)]
pub struct WxAuthResponse {
    pub session_key: Option<String>,
    pub unionid: Option<String>,
    pub openid: Option<String>,
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
}
