use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;

use crate::prefixes::TokenPrefix;

/// Primary key for `oidc_sign_in_accounts`.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Default, ToSchema)]
#[cfg_attr(feature = "database", derive(sqlx::Type))]
#[cfg_attr(feature = "database", sqlx(transparent))]
pub struct OidcSignInAccountToken(String);

impl_crockford_generator!(OidcSignInAccountToken, 32usize, TokenPrefix::OidcSignInAccount, CrockfordLower);
impl_mysql_token_from_row!(OidcSignInAccountToken);
impl_string_token!(OidcSignInAccountToken);
