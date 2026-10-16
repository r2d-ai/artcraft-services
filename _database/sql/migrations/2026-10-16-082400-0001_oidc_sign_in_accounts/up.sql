-- noinspection SqlDialectInspectionForFile
-- noinspection SqlNoDataSourceInspectionForFile
-- noinspection SqlResolveForFile

-- Generic OIDC / Keycloak sign-in accounts. User linkage is by (issuer, subject) only.
CREATE TABLE oidc_sign_in_accounts(
  id BIGINT(20) UNSIGNED NOT NULL AUTO_INCREMENT,

  token VARCHAR(32) NOT NULL,

  issuer VARCHAR(512) NOT NULL,
  subject VARCHAR(255) NOT NULL,

  maybe_user_token VARCHAR(32) DEFAULT NULL,

  maybe_email_address VARCHAR(255) DEFAULT NULL,
  is_email_verified BOOLEAN NOT NULL DEFAULT false,

  maybe_locale VARCHAR(255) DEFAULT NULL,
  maybe_name VARCHAR(255) DEFAULT NULL,
  maybe_given_name VARCHAR(255) DEFAULT NULL,
  maybe_family_name VARCHAR(255) DEFAULT NULL,

  ip_address_creation VARCHAR(40) NOT NULL,
  ip_address_update VARCHAR(40) NOT NULL,

  version INT NOT NULL DEFAULT 0,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

  PRIMARY KEY (id),
  UNIQUE KEY (token),
  UNIQUE KEY index_issuer_subject (issuer, subject),
  KEY index_maybe_user_token (maybe_user_token),
  KEY index_created_at (created_at),
  KEY index_updated_at (updated_at)

) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;
