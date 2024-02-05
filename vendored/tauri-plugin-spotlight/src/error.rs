#[derive(Debug)]
pub enum Error {
    FailedToLockMutex,
    FailedToGetExecutablePath,
    WindowNotFound,
    FailedToRegisterShortcut,
    FailedToUnregisterShortcut,
    FailedToGetNSWindow,
    FailedToGetNSWorkspaceClass,
    FailedToCheckWindowVisibility,
    FailedToHideWindow,
    FailedToShowWindow,
}
