#[repr(i32)]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum MediaStatus {
    Ok = 0,
    ErrorInsufficientResource = 1100,
    ErrorReclaimed = 1101,
    ErrorUnknown = -10000,
    ErrorMalformed = -10001,
    ErrorUnsupported = -10002,
    ErrorInvalidObject = -10003,
    ErrorInvalidParameter = -10004,
    ErrorInvalidOperation = -10005,
    ErrorEndOfStream = -10006,
    ErrorIO = -10007,
    ErrorWouldBlock = -10008,
    DRMErrorBase = -20000,
    DRMNotProvisioned = -20001,
    DRMResourceBusy = -20002,
    DRMDeviceRevoked = -20003,
    DRMShortBuffer = -20004,
    DRMSessionNotOpened = -20005,
    DRMTamperDetected = -20006,
    DRMVerifyFailed = -20007,
    DRMNeedKey = -20008,
    DRMLicenseExpired = -20009,
    ImgReaderErrorBase = -30000,
    ImgReaderNoBufferAvailable = -30001,
    ImgReaderMaxImagesAcquired = -30002,
    ImgReaderCannotLockImage = -30003,
    ImgReaderCannotUnlockImage = -30004,
    ImgReaderImageNotLocked = -30005,
}

impl MediaStatus {
    /// Convert a raw `i32` from FFI to a `MediaStatus`.
    /// This is safe because unknown values are mapped to `Ok` (if >= 0) or `ErrorUnknown` (if < 0).
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => Self::Ok,
            1100 => Self::ErrorInsufficientResource,
            1101 => Self::ErrorReclaimed,
            -10000 => Self::ErrorUnknown,
            -10001 => Self::ErrorMalformed,
            -10002 => Self::ErrorUnsupported,
            -10003 => Self::ErrorInvalidObject,
            -10004 => Self::ErrorInvalidParameter,
            -10005 => Self::ErrorInvalidOperation,
            -10006 => Self::ErrorEndOfStream,
            -10007 => Self::ErrorIO,
            -10008 => Self::ErrorWouldBlock,
            -20000 => Self::DRMErrorBase,
            -20001 => Self::DRMNotProvisioned,
            -20002 => Self::DRMResourceBusy,
            -20003 => Self::DRMDeviceRevoked,
            -20004 => Self::DRMShortBuffer,
            -20005 => Self::DRMSessionNotOpened,
            -20006 => Self::DRMTamperDetected,
            -20007 => Self::DRMVerifyFailed,
            -20008 => Self::DRMNeedKey,
            -20009 => Self::DRMLicenseExpired,
            -30000 => Self::ImgReaderErrorBase,
            -30001 => Self::ImgReaderNoBufferAvailable,
            -30002 => Self::ImgReaderMaxImagesAcquired,
            -30003 => Self::ImgReaderCannotLockImage,
            -30004 => Self::ImgReaderCannotUnlockImage,
            -30005 => Self::ImgReaderImageNotLocked,
            v if v >= 0 => Self::Ok,
            _ => Self::ErrorUnknown,
        }
    }

    /// Convert a raw `i32` FFI return value into `Result<(), MediaStatus>`.
    /// Unknown negative values become `ErrorUnknown`; unknown non-negative values succeed.
    pub fn make_result(value: i32) -> Result<(), MediaStatus> {
        let status = Self::from_i32(value);
        if status.is_ok() {
            Ok(())
        } else {
            Err(status)
        }
    }

    pub fn is_ok(&self) -> bool {
        *self == Self::Ok
    }
}


