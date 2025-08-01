use std::{env::VarError, str::FromStr};

pub const ORB_BACKEND_ENV_VAR_NAME: &str = "ORB_BACKEND";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Backend {
    Prod,
    Staging,
    Analysis,
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildType {
    Prod,
    Staging,
    Analysis,
}

impl Backend {
    /// Choose the backend based on the environment variable.
    /// See also [`Self::from_env_or_build_type()`] for a more convenient constructor.
    pub fn from_env() -> Result<Self, BackendFromEnvError> {
        let v = std::env::var(ORB_BACKEND_ENV_VAR_NAME).map_err(|e| match e {
            VarError::NotPresent => BackendFromEnvError::NotSet,
            VarError::NotUnicode(_) => BackendFromEnvError::Invalid(BackendParseErr),
        })?;

        Self::from_str(&v).map_err(|e| e.into())
    }

    /// Choose the backend based on environment variable, using the build type
    /// to determine the fallback in the event the variable is missing.
    ///
    /// # Panics
    /// - If the env var was provided but could not parse.
    /// - If the build was staging but the env var was prod.
    /// - If the build was analysis but the env var was prod.
    ///
    /// # Example usage
    /// ```
    /// use orb_endpoints::{Backend, BuildType};
    /// Backend::from_env_or_build_type(BuildType::Stage);
    /// ```
    ///
    pub fn from_env_or_build_type(build_type: BuildType) -> Self {
        let b = match Backend::from_env() {
            Ok(b) => b,
            Err(BackendFromEnvError::NotSet) => match build_type {
                BuildType::Prod => Backend::Prod,
                BuildType::Staging => Backend::Staging,
                BuildType::Analysis => Backend::Analysis,
            },
            Err(err @ BackendFromEnvError::Invalid(_)) => {
                panic!("could not parse backend from env var: {err}")
            }
        };
        match (b, build_type) {
            (Backend::Prod, BuildType::Staging) => {
                panic!("tried to talk to prod backend but this is a staging build!");
            }
            (Backend::Prod, BuildType::Analysis) => {
                panic!("tried to talk to prod backend but this is an analysis build!");
            }
            _ => {}
        }
        b
    }
}

impl FromStr for Backend {
    type Err = BackendParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "prod" | "production" => Ok(Self::Prod),
            "stage" | "staging" | "dev" | "development" => Ok(Self::Staging),
            "analysis" | "analysis.ml" | "analysis-ml" => Ok(Self::Analysis),
            "local" | "localhost" | "127.0.0.1" => Ok(Self::Local),
            _ => Err(BackendParseErr),
        }
    }
}

// ---- Error types ----

/// Error from parsing a string into [`crate::Backend`].
#[derive(Debug, Eq, PartialEq)]
pub struct BackendParseErr;

impl std::fmt::Display for BackendParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to parse `Backend` from str")
    }
}

impl std::error::Error for BackendParseErr {}

/// Error from parsing env var into [`crate::Backend`].
#[derive(Debug, Eq, PartialEq)]
pub enum BackendFromEnvError {
    NotSet,
    Invalid(BackendParseErr),
}

impl std::fmt::Display for BackendFromEnvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendFromEnvError::NotSet => {
                write!(f, "env var {ORB_BACKEND_ENV_VAR_NAME} was not set")
            }
            BackendFromEnvError::Invalid(_e) => {
                write!(f, "env var {ORB_BACKEND_ENV_VAR_NAME} failed to parse")
            }
        }
    }
}

impl std::error::Error for BackendFromEnvError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BackendFromEnvError::NotSet => None,
            BackendFromEnvError::Invalid(e) => Some(e),
        }
    }
}

impl From<BackendParseErr> for BackendFromEnvError {
    fn from(value: BackendParseErr) -> Self {
        Self::Invalid(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_backend_parse() {
        assert_eq!(Backend::from_str("prod").unwrap(), Backend::Prod);
        assert_eq!(Backend::from_str("pRod").unwrap(), Backend::Prod);
        assert_eq!(Backend::from_str("stage").unwrap(), Backend::Staging);
        assert_eq!(Backend::from_str("staGe").unwrap(), Backend::Staging);
        assert_eq!(Backend::from_str("dev").unwrap(), Backend::Staging);
        assert_eq!(Backend::from_str("analysis").unwrap(), Backend::Analysis);
        assert_eq!(Backend::from_str("foobar"), Err(BackendParseErr));
    }
}
