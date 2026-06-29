use std::ffi::CString;
use tracing::{error, info};

/// Trigger a jemalloc heap dump.
/// This is only available when the `profiling` feature is enabled and
/// the application is running on a platform where jemalloc is supported.
#[cfg(feature = "profiling")]
pub fn dump_memory_stats(path: &str) -> Result<(), String> {
    info!("Triggering jemalloc heap dump to: {}", path);

    let path_cstring = CString::new(path).map_err(|e| format!("Invalid path: {}", e))?;

    // Safety: tikv-jemalloc-ctl provides a safe wrapper around jemalloc's control APIs
    match tikv_jemalloc_ctl::prof::dump::write(path_cstring.as_c_str()) {
        Ok(_) => {
            info!("Successfully wrote memory dump to {}", path);
            Ok(())
        }
        Err(e) => {
            error!("Failed to write memory dump: {}", e);
            Err(format!("Jemalloc error: {}", e))
        }
    }
}

/// Dummy implementation for when profiling is disabled.
#[cfg(not(feature = "profiling"))]
pub fn dump_memory_stats(_path: &str) -> Result<(), String> {
    error!("Memory profiling requested but the 'profiling' feature is not enabled");
    Err(
        "Profiling feature not enabled. Recompile with --features profiling to use this endpoint."
            .to_string(),
    )
}

/// Get jemalloc stats summary if available.
#[cfg(feature = "profiling")]
pub fn get_memory_stats() -> Result<String, String> {
    use tikv_jemalloc_ctl::{epoch, stats};

    // Advance the epoch to refresh stats
    epoch::advance().map_err(|e| format!("Failed to advance jemalloc epoch: {}", e))?;

    let allocated =
        stats::allocated::read().map_err(|e| format!("Failed to read allocated stats: {}", e))?;
    let active =
        stats::active::read().map_err(|e| format!("Failed to read active stats: {}", e))?;
    let metadata =
        stats::metadata::read().map_err(|e| format!("Failed to read metadata stats: {}", e))?;
    let resident =
        stats::resident::read().map_err(|e| format!("Failed to read resident stats: {}", e))?;
    let mapped =
        stats::mapped::read().map_err(|e| format!("Failed to read mapped stats: {}", e))?;
    let retained =
        stats::retained::read().map_err(|e| format!("Failed to read retained stats: {}", e))?;

    Ok(format!(
        "Jemalloc Stats:\n\
         - Allocated: {} bytes\n\
         - Active: {} bytes\n\
         - Metadata: {} bytes\n\
         - Resident: {} bytes\n\
         - Mapped: {} bytes\n\
         - Retained: {} bytes",
        allocated, active, metadata, resident, mapped, retained
    ))
}

#[cfg(not(feature = "profiling"))]
pub fn get_memory_stats() -> Result<String, String> {
    Err("Profiling feature not enabled.".to_string())
}
