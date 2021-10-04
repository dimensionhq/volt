use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum VoltError {
    // #[error("failed to enable ansi support")]
    // #[diagnostic(code(volt::environment::enable_ansi_support))]
    // EnableAnsiSupport(),
    #[error("failed to detect `{env}`")]
    #[diagnostic(code(volt::environment::get))]
    EnvironmentError { source: std::io::Error, env: String },

    #[error("failed to detect your home directory")]
    #[diagnostic(code(volt::environment::home_dir))]
    GetHomeDirError,

    #[error("failed to initialize lz4 decoder")]
    #[diagnostic(code(volt::decode::lz4::initialize))]
    DecoderError(#[source] std::io::Error),

    #[error("failed to decode lz4 encoded data")]
    #[diagnostic(code(volt::decode::lz4::decode))]
    DecodeError(#[source] std::io::Error),

    #[error("failed to recieve response from the registry")]
    #[diagnostic(code(volt::network))]
    NetworkError(isahc::Error),

    #[error("failed to recieve byte response")]
    #[diagnostic(code(volt::network::rec))]
    NetworkRecError(#[source] std::io::Error),

    #[error("failed to create directory")]
    #[diagnostic(code(volt::io::create_dir))]
    CreateDirError(#[source] std::io::Error),

    #[error("GET {url} - 404 - {package_name} was not found in the volt registry, or you don't have the permission to request it.")]
    #[diagnostic(code(volt::registry::volt::package_not_found))]
    PackageNotFound { url: String, package_name: String },

    #[error("GET {url} - 429 - Too many requests has been sent to {url} on the volt registry. Please try again later.")]
    #[diagnostic(code(volt::registry::volt::too_many_requests))]
    TooManyRequests { url: String, package_name: String },

    #[error("GET {url} - 400 - Bad request. Please try again later.")]
    #[diagnostic(code(volt::registry::volt::bad_request))]
    BadRequest { url: String, package_name: String },

    #[error("GET {url} - {} - An unknown error occured. Please try again later.")]
    #[diagnostic(code(volt::registry::volt::unknown_error))]
    NetworkUnknownError {
        url: String,
        package_name: String,
        code: String,
    },

    #[error("failed to parse {hash} integrity hash.")]
    #[diagnostic(code(volt::integrity::parse))]
    HashParseError { hash: String },

    #[error("failed to copy bytes to hasher.")]
    #[diagnostic(code(volt::hasher::copy))]
    HasherCopyError(#[source] std::io::Error),

    #[error("failed to verify tarball checksum")]
    #[diagnostic(code(volt::integrity::verify))]
    ChecksumVerificationError,

    #[error("failed to convert integrity into hex")]
    #[diagnostic(code(volt::integrity::convert))]
    IntegrityConversionError,

    #[error("failed to deserialize slice to `JSONVoltResponse`")]
    #[diagnostic(code(volt::integrity::convert))]
    DeserializeError,

    #[error("failed to build request client")]
    #[diagnostic(code(volt::network::builder))]
    RequestBuilderError(#[source] isahc::http::Error),

    #[error("failed to build recieve response text")]
    #[diagnostic(code(volt::io::rec::text))]
    IoTextRecError(#[source] std::io::Error),

    #[error("failed to find a hash that matches the specified version requirement: {version}")]
    #[diagnostic(code(volt::io::rec::text))]
    HashLookupError { version: String },

    #[error("failed to find a version that matches the specified version requirement for {name}")]
    #[diagnostic(code(volt::io::rec::text))]
    VersionLookupError { name: String },

    #[error("failed to read `{name}`")]
    #[diagnostic(code(volt::io::file::read))]
    ReadFileError {
        source: std::io::Error,
        name: String,
    },

    #[error("failed to write to `{name}`")]
    #[diagnostic(code(volt::io::file::write))]
    WriteFileError {
        source: std::io::Error,
        name: String,
    },

    #[error("an unknown error occured.")]
    #[diagnostic(code(volt::unknown))]
    UnknownError,
}
