#[derive(Debug)]
pub enum Error {
    FailedToLockMutex,
    FailedToGetExecutablePath,
    FailedToRegisterShortcut,
    FailedToUnregisterShortcut,
    FailedToGetNSWindow,
    FailedToGetNSWorkspaceClass,
    FailedToCheckWindowVisibility,
    FailedToHideWindow,
    FailedToShowWindow,
}
