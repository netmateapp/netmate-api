use crate::{common::{id::account_id::AccountId, session::{refresh_token::RefreshToken, session_id::SessionId, session_series::SessionSeries}}, helper::redis::{Namespace, NAMESPACE_SEPARATOR}};

pub const SESSION_ID_NAMESPACE: Namespace = Namespace::of("sid");

pub const REFRESH_PAIR_NAMESPACE: Namespace = Namespace::of("rfp");
pub const REFRESH_PAIR_VALUE_SEPARATOR: char = '$';

pub fn format_session_id_key(session_id: &SessionId) -> String {
    format!("{}{}{}", SESSION_ID_NAMESPACE, NAMESPACE_SEPARATOR, session_id)
}

pub fn format_refresh_pair_key(session_series: &SessionSeries) -> String {
    format!("{}{}{}", REFRESH_PAIR_NAMESPACE, NAMESPACE_SEPARATOR, session_series)
}

pub fn format_refresh_pair_value(new_refresh_token: &RefreshToken, session_account_id: AccountId) -> String {
    format!("{}{}{}", new_refresh_token, REFRESH_PAIR_VALUE_SEPARATOR, session_account_id)
}

#[cfg(test)]
mod tests {
    use crate::{common::{id::account_id::AccountId, session::{refresh_token::RefreshToken, session_id::SessionId, session_series::SessionSeries}}, helper::redis::NAMESPACE_SEPARATOR, middlewares::value::{format_refresh_pair_key, format_refresh_pair_value, format_session_id_key, REFRESH_PAIR_NAMESPACE, REFRESH_PAIR_VALUE_SEPARATOR, SESSION_ID_NAMESPACE}};

    #[test]
    fn test_format_session_id_key() {
        let session_id = SessionId::gen();
        let key = format_session_id_key(&session_id);
        let expected = format!("{}{}{}", SESSION_ID_NAMESPACE, NAMESPACE_SEPARATOR, session_id);
        assert_eq!(key, expected);
    }

    #[test]
    fn test_format_refresh_pair_key() {
        let session_series = SessionSeries::gen();
        let key = format_refresh_pair_key(&session_series);
        let expected = format!("{}{}{}", REFRESH_PAIR_NAMESPACE, NAMESPACE_SEPARATOR, session_series);
        assert_eq!(key, expected);
    }

    #[test]
    fn test_format_refresh_pair_value() {
        let new_refresh_token = RefreshToken::gen();
        let session_account_id = AccountId::gen();
        let value = format_refresh_pair_value(&new_refresh_token, session_account_id);
        let expected = format!("{}{}{}", new_refresh_token, REFRESH_PAIR_VALUE_SEPARATOR, session_account_id);
        assert_eq!(value, expected);
    }
}