mod proto;
pub mod server;
use dotenv::dotenv;
use server::logger;

pub fn run() -> () {
    dotenv().ok();
    logger::init().expect("日志系统初始化失败");
    
}
