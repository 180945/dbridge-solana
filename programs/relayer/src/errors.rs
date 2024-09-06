use anchor_lang::prelude::*;

#[error_code]
pub enum RelayError {
    #[msg("Invalid block header size")]
    InvalidHeaderSize,

    #[msg("Invalid genesis height")]
    InvalidGenesisHeight,

    #[msg("Invalid block header batch")]
    InvalidHeaderBatch,

    #[msg("Block already stored")]
    DuplicateBlock,

    #[msg("Previous block hash not found")]
    PreviousBlockNotFound,

    #[msg("Insufficient difficulty")]
    LowDifficulty,

    #[msg("Incorrect difficulty target")]
    IncorrectDifficultyTarget,

    #[msg("Invalid difficulty period")]
    InvalidDifficultyPeriod,

    #[msg("Not extension of chain")]
    NotChainExtension,

    #[msg("Block not found")]
    BlockNotFound,

    #[msg("Insufficient confirmations")]
    InsufficientConfirmations,

    #[msg("Incorrect merkle proof")]
    IncorrectMerkleProof,

    #[msg("Invalid tx identifier")]
    InvalidTxId,

    #[msg("Invalid block hash")]
    InvalidBlockHash,

    #[msg("Division by zero")]
    DivisionByZero,

    #[msg("Arithmetic error")]
    ArithmeticError,

    #[msg("Invalid counter")]
    InvalidCounter,

    #[msg("Invalid chain id")]
    InvalidChainId,

    #[msg("Fork not found")]
    ForkNotFound,
}