use http::{header::SET_COOKIE, Extensions, HeaderMap};

use crate::common::{id::AccountId, session::value::LoginToken};

pub fn insert_account_id(extensions: &mut Extensions, account_id: AccountId) {
    extensions.insert(account_id);
}

pub fn can_set_cookie_in_response_header(headers: &HeaderMap) -> bool {
    !headers.contains_key(SET_COOKIE)
}

pub fn is_same_token(request_token: &LoginToken, registered_token: &LoginToken) -> bool {
    request_token.value().value() == registered_token.value().value()
}

#[cfg(test)]
mod tests {
    mod insert_account_id_tests {
        use http::Extensions;

        use crate::{common::id::{uuid7::Uuid7, AccountId}, middlewares::session::dsl::misc::insert_account_id};

        #[test]
        fn insert_one() {
            let mut extensions = Extensions::new();
            let account_id = AccountId::new(Uuid7::now());
            insert_account_id(&mut extensions, account_id.clone());
            assert_eq!(extensions.get::<AccountId>(), Some(&account_id));
        }
    }

    mod test_can_set_cookie_in_response_header_tests {
        use http::{header::SET_COOKIE, HeaderMap, HeaderValue};

        use crate::middlewares::session::dsl::misc::can_set_cookie_in_response_header;

        #[test]
        fn can_set() {
            let headers = HeaderMap::new();
            assert_eq!(can_set_cookie_in_response_header(&headers), true);
        }

        #[test]
        fn cannot_set() {
            let mut headers = HeaderMap::new();
            headers.insert(SET_COOKIE, HeaderValue::from_static("dummy"));
            assert_eq!(can_set_cookie_in_response_header(&headers), false);
        }
    }

    mod is_same_token_tests {
        use crate::{common::session::value::LoginToken, middlewares::session::dsl::misc::is_same_token};

        #[test]
        fn same() {
            let token = LoginToken::gen();
            assert_eq!(is_same_token(&token, &token), true);
        }

        #[test]
        fn different() {
            let token = LoginToken::gen();
            let another_token = LoginToken::gen();
            assert_eq!(is_same_token(&token, &another_token), false);
        }
    }
}