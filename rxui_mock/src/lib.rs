pub mod sync;
pub mod thread;
pub mod time;

#[macro_export]
macro_rules! rxui_mock_init {
    () => {
        use rxui_mock::sync;

        #[cfg(test)]
        use rxui_mock::thread;
        #[cfg(not(test))]
        use std::thread;

        #[cfg(test)]
        use rxui_mock::time;
        #[cfg(not(test))]
        use std::time;
    };
}
