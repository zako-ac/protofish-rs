use crate::{
    prost_generated::common::{self},
    schema::common::schema::{ErrorType, IntegrityType, StreamCreateMeta, Version},
};

impl From<common::v1::Version> for Version {
    fn from(value: common::v1::Version) -> Self {
        Version {
            major: value.major,
            minor: value.minor,
            patch: value.patch,
        }
    }
}

impl From<Version> for common::v1::Version {
    fn from(value: Version) -> Self {
        common::v1::Version {
            major: value.major,
            minor: value.minor,
            patch: value.patch,
        }
    }
}

impl From<common::v1::StreamCreateMeta> for StreamCreateMeta {
    fn from(value: common::v1::StreamCreateMeta) -> Self {
        StreamCreateMeta {
            integrity_type: common::v1::IntegrityType::try_from(value.stream_integrity)
                .unwrap()
                .into(),
        }
    }
}

impl From<common::v1::IntegrityType> for IntegrityType {
    fn from(value: common::v1::IntegrityType) -> Self {
        match value {
            // default to reliable
            common::v1::IntegrityType::Unspecified => IntegrityType::Reliable,
            common::v1::IntegrityType::Reliable => IntegrityType::Reliable,
            common::v1::IntegrityType::Unreliable => IntegrityType::Unreliable,
        }
    }
}

impl From<IntegrityType> for common::v1::IntegrityType {
    fn from(value: IntegrityType) -> Self {
        match value {
            IntegrityType::Reliable => common::v1::IntegrityType::Reliable,
            IntegrityType::Unreliable => common::v1::IntegrityType::Unreliable,
        }
    }
}

impl From<common::v1::ErrorType> for ErrorType {
    fn from(value: common::v1::ErrorType) -> Self {
        match value {
            common::v1::ErrorType::Unspecified => ErrorType::Unspecified,
            common::v1::ErrorType::Timeout => ErrorType::Timeout,
        }
    }
}

impl From<ErrorType> for common::v1::ErrorType {
    fn from(value: ErrorType) -> Self {
        match value {
            ErrorType::Unspecified => common::v1::ErrorType::Unspecified,
            ErrorType::Timeout => common::v1::ErrorType::Timeout,
        }
    }
}
