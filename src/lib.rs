mod proto;
use proto::auth::person::PhoneNumber;

pub fn help() -> proto::auth::Person {
    let person = proto::auth::Person {
        id: 1,
        name: "John Doe".to_string(),
        email: "john.doe@example.com".to_string(),
        phones: vec![PhoneNumber {
            number: "555-1234".to_string(),
            r#type: proto::auth::person::PhoneType::Mobile as i32,
        }],
    };
    person
}
