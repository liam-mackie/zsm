mod budget;
mod conflict;
mod generator;
mod normalizer;
mod truncator;

pub use budget::{
    max_generated_name_len, name_budget_for_socket_dir, DEFAULT_MAX_NAME_LENGTH,
    FALLBACK_NAME_BUDGET, SOCKET_DIR_PROBE,
};
pub use generator::SessionNameGenerator;
pub use normalizer::PathNormalizer;

#[cfg(test)]
mod tests;
