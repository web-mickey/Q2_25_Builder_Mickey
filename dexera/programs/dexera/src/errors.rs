use anchor_lang::prelude::*;
use constant_product_curve::CurveError;

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid Config")]
    InvalidConfig,

    #[msg("Invalid Amount")]
    InvalidAmount,

    #[msg("AMM is Locked")]
    AMMLocked,

    #[msg("Insufficient amount of token X")]
    InsufficientTokenX,

    #[msg("Insufficient amount of token Y")]
    InsufficientTokenY,

    #[msg("Insufficient Balance")]
    InsufficientBalance,

    #[msg("Slippage exceeded")]
    SlippageExceeded,

    #[msg("Invalid Referrer Profile")]
    InvalidReferrerProfile,

    #[msg("Missing Referrer Profile")]
    MissingReferrerProfile,

    #[msg("Invalid Referrer Ata")]
    InvalidReferrerAta,
}

// TODO: Understand how the From trait works in Rust
impl From<CurveError> for ErrorCode {
    fn from(error: CurveError) -> ErrorCode {
        match error {
            CurveError::InvalidPrecision => ErrorCode::InvalidAmount,
            CurveError::Overflow => ErrorCode::InvalidAmount,
            CurveError::Underflow => ErrorCode::InvalidAmount,
            CurveError::InvalidFeeAmount => ErrorCode::InvalidAmount,
            CurveError::InsufficientBalance => ErrorCode::InsufficientBalance,
            CurveError::ZeroBalance => ErrorCode::InvalidAmount,
            CurveError::SlippageLimitExceeded => ErrorCode::InvalidAmount,
        }
    }
}
