use sqlx::{Executor, MySql};

use tokens::tokens::oidc_sign_in_accounts::OidcSignInAccountToken;
use tokens::tokens::users::UserToken;

pub struct InsertOidcSignInAccountArgs<'a, T> {
  pub token: &'a OidcSignInAccountToken,
  pub issuer: &'a str,
  pub subject: &'a str,
  pub maybe_user_token: Option<&'a UserToken>,
  pub maybe_email_address: Option<&'a str>,
  pub is_email_verified: bool,
  pub maybe_name: Option<&'a str>,
  pub ip_address: &'a str,
  pub mysql_executor: T,
}

pub async fn insert_oidc_sign_in_account<'c, T>(
  args: InsertOidcSignInAccountArgs<'_, T>,
) -> Result<(), sqlx::Error>
where
  T: Executor<'c, Database = MySql>,
{
  let is_email_verified = if args.is_email_verified { 1i8 } else { 0i8 };

  sqlx::query(
    r#"
      INSERT INTO oidc_sign_in_accounts (
        token,
        issuer,
        subject,
        maybe_user_token,
        maybe_email_address,
        is_email_verified,
        maybe_name,
        ip_address_creation,
        ip_address_update
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
    "#,
  )
    .bind(args.token.as_str())
    .bind(args.issuer)
    .bind(args.subject)
    .bind(args.maybe_user_token.map(|token| token.as_str()))
    .bind(args.maybe_email_address)
    .bind(is_email_verified)
    .bind(args.maybe_name)
    .bind(args.ip_address)
    .bind(args.ip_address)
    .execute(args.mysql_executor)
    .await?;

  Ok(())
}
