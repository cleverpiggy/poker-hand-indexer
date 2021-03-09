// for now this is how I know how to do it until i figure out build.rs
// bindgen --whitelist-function '^rust.*' --whitelist-type '^rust.*' --whitelist-var '^rust.*'  src_c/rust.h


pub type IndexerPtr = *mut ::std::os::raw::c_void;

extern "C" {
    pub fn rust_init_indexer(
        rounds: u32,
        cards_per_round: *const u8,
        success: *mut bool,
    ) -> *mut ::std::os::raw::c_void;

    pub fn rust_free_indexer(indexer: *mut ::std::os::raw::c_void);

    pub fn rust_indexer_size(v_indexer: *mut ::std::os::raw::c_void, round: u32) -> u64;

    pub fn rust_index_all(
        v_indexer: *mut ::std::os::raw::c_void,
        cards: *const u8,
        indices: *mut u64,
    ) -> u64;

    pub fn rust_unindex(
        v_indexer: *mut ::std::os::raw::c_void,
        round: u32,
        index: u64,
        cards: *mut u8,
    ) -> bool;

    pub fn rust_index_round(
        v_indexer: *mut ::std::os::raw::c_void,
        cards: *const u8,
        ncards: u32,
    ) -> u64;
}
