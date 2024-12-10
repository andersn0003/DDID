use anchor_lang::prelude::*;

#[error_code]
pub enum AuthenticationError {
  #[msg("Hash too long")]
  HashTooLong,
}