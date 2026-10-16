use chrono::{DateTime, Utc};
use sqlx::{Executor, FromRow, MySql};

use crate::helpers::boolean_converters::i8_to_bool;
use errors::AnyhowResult;
use tokens::tokens::oidc_sign_in_accounts::OidcSignInAccountToken;
use tokens::tokens::users::UserToken;

#[derive(Serialize, Debug)]
pub struct OidcSignInAccount {
  pub token: OidcSignInAccountToken,
  pub issuer: String,
  pub subject: String,
  pub maybe_user_token: Option<UserToken>,
  pub maybe_email_address: Option<String>,
  pub is_email_verified: bool,
  pub maybe_name: Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(FromRow)]
struct OidcSignInAccountRaw {
  token: OidcSignInAccountToken,
  issuer: String,
  subject: String,
  maybe_user_token: Option<UserToken>,
  maybe_email_address: Option<String>,
  is_email_verified: i8,
  maybe_name: Option<String>,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
}

pub struct GetOidcSignInAccountByIssuerSubjectArgs<'a, T> {
  pub issuer: &'a str,
  pub subject: &'a str,
  pub mysql_executor: T,
}

pub async fn get_oidc_sign_in_account_by_issuer_subject<'c, T>(
  args: GetOidcSignInAccountByIssuerSubjectArgs<'_, T>,
) -> Result<Option<OidcSignInAccount>, sqlx::Error>
where
  T: Executor<'c, Database = MySql>,
{
  let record = sqlx::query_as::<_, OidcSignInAccountRaw>(
    r#"
      SELECT
        token,
        issuer,
        subject,
        maybe_user_token,
        maybe_email_address,
        is_email_verified,
        maybe_name,
        created_at,
        updated_at
      FROM oidc_sign_in_accounts
      WHERE issuer = ? AND subject = ?
      LIMIT 1
    "#,
  )
    .bind(args.issuer)
    .bind(args.subject)
    .fetch_optional(args.mysql_executor)
    .await?;

  Ok(record.map(|record| OidcSignInAccount {
    token: record.token,
    issuer: record.issuer,
    subject: record.subject,
    maybe_user_token: record.maybe_user_token,
    maybe_email_address: record.maybe_email_address,
    is_email_verified: i8_to_bool(record.is_email_verified),
    maybe_name: record.maybe_name,
    created_at: record.created_at,
    updated_at: record.updated_at,
  }))
}
