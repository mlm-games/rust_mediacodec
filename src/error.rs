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
    fn all() -> &'static [Self] {
        use MediaStatus::*;
        &[
            Ok,
            ErrorInsufficientResource,
            ErrorReclaimed,
            ErrorUnknown,
            ErrorMalformed,
            ErrorUnsupported,
            ErrorInvalidObject,
            ErrorInvalidParameter,
            ErrorInvalidOperation,
            ErrorEndOfStream,
            ErrorIO,
            ErrorWouldBlock,
            DRMErrorBase,
            DRMNotProvisioned,
            DRMResourceBusy,
            DRMDeviceRevoked,
            DRMShortBuffer,
            DRMSessionNotOpened,
            DRMTamperDetected,
            DRMVerifyFailed,
            DRMNeedKey,
            DRMLicenseExpired,
            ImgReaderErrorBase,
            ImgReaderNoBufferAvailable,
            ImgReaderMaxImagesAcquired,
            ImgReaderCannotLockImage,
            ImgReaderCannotUnlockImage,
            ImgReaderImageNotLocked,
        ]
    }

    pub fn make_result(value: isize) -> Result<isize, MediaStatus> {
        match MediaStatus::try_from(value) {
            Ok(status) if status.is_ok() => Ok(status as isize),
            Ok(status) => Err(status),
            Err(_) => Ok(value),
        }
    }

    pub fn result(&self) -> Result<isize, Self> {
        if self.is_ok() {
            Ok(*self as isize)
        } else {
            Err(*self)
        }
    }

    pub fn is_ok(&self) -> bool {
        *self == Self::Ok
    }
}

impl TryFrom<isize> for MediaStatus {
    type Error = &'static str;

    fn try_from(value: isize) -> Result<Self, Self::Error> {
        Self::all()
            .iter()
            .find(|&&s| s as isize == value)
            .copied()
            .ok_or("Not Found")
    }
}
