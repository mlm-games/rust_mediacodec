#[repr(C)]
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum MediaStatus {
    Ok = 0,
    ErrorInsufficientResource = 1100,
    ErrorReclaimed = 1101,
    ErrorUnknown = -10000,
    ErrorMalformed = MediaStatus::ErrorUnknown as isize - 1,
    ErrorUnsupported = MediaStatus::ErrorUnknown as isize - 2,
    ErrorInvalidObject = MediaStatus::ErrorUnknown as isize - 3,
    ErrorInvalidParameter = MediaStatus::ErrorUnknown as isize - 4,
    ErrorInvalidOperation = MediaStatus::ErrorUnknown as isize - 5,
    ErrorEndOfStream = MediaStatus::ErrorUnknown as isize - 6,
    ErrorIO = MediaStatus::ErrorUnknown as isize - 7,
    ErrorWouldBlock = MediaStatus::ErrorUnknown as isize - 8,
    DRMErrorBase = -20000,
    DRMNotProvisioned = MediaStatus::DRMErrorBase as isize - 1,
    DRMResourceBusy = MediaStatus::DRMErrorBase as isize - 2,
    DRMDeviceRevoked = MediaStatus::DRMErrorBase as isize - 3,
    DRMShortBuffer = MediaStatus::DRMErrorBase as isize - 4,
    DRMSessionNotOpened = MediaStatus::DRMErrorBase as isize - 5,
    DRMTamperDetected = MediaStatus::DRMErrorBase as isize - 6,
    DRMVerifyFailed = MediaStatus::DRMErrorBase as isize - 7,
    DRMNeedKey = MediaStatus::DRMErrorBase as isize - 8,
    DRMLicenseExpired = MediaStatus::DRMErrorBase as isize - 9,
    ImgReaderErrorBase = -30000,
    ImgReaderNoBufferAvailable = MediaStatus::ImgReaderErrorBase as isize - 1,
    ImgReaderMaxImagesAcquired = MediaStatus::ImgReaderErrorBase as isize - 2,
    ImgReaderCannotLockImage = MediaStatus::ImgReaderErrorBase as isize - 3,
    ImgReaderCannotUnlockImage = MediaStatus::ImgReaderErrorBase as isize - 4,
    ImgReaderImageNotLocked = MediaStatus::ImgReaderErrorBase as isize - 5,
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

    fn is_ok(&self) -> bool {
        !Self::all().iter().skip(1).any(|&x| *self == x)
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
