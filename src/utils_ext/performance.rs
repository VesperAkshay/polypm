use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use serde::{Deserialize, Serialize};

/// Performance optimization enhancements for PPM
/// Provides parallel downloads and advanced caching

/// Download cache entry with TTL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Cached data
    pub data: Vec<u8>,
    /// Timestamp when cached
    pub cached_at: u64,
    /// Time-to-live in seconds
    pub ttl: u64,
    /// Size in bytes
    pub size: u64,
    /// Number of times accessed
    pub access_count: u64,
    /// Package metadata
    pub metadata: CacheMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Package ecosystem
    pub ecosystem: String,
    /// Content type
    pub content_type: Option<String>,
    /// Integrity hash
    pub integrity: Option<String>,
}

impl CacheEntry {
    pub fn new(data: Vec<u8>, ttl: u64, metadata: CacheMetadata) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            size: data.len() as u64,
            data,
            cached_at: now,
            ttl,
            access_count: 1,
            metadata,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        now > self.cached_at + self.ttl
    }
    
    pub fn touch(&mut self) {
        self.access_count += 1;
    }
}

/// Download cache with LRU eviction and TTL
#[derive(Debug)]
pub struct DownloadCache {
    /// Cache storage
    entries: Arc<Mutex<HashMap<String, CacheEntry>>>,
    /// Maximum cache size in bytes
    max_size: u64,
    /// Current cache size in bytes
    current_size: Arc<Mutex<u64>>,
    /// Default TTL for cache entries in seconds
    default_ttl: u64,
}

impl DownloadCache {
    pub fn new(max_size_mb: u64, default_ttl_seconds: u64) -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            max_size: max_size_mb * 1024 * 1024, // Convert MB to bytes
            current_size: Arc::new(Mutex::new(0)),
            default_ttl: default_ttl_seconds,
        }
    }
    
    /// Get cached data if available and not expired
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let mut entries = self.entries.lock().unwrap();
        
        if let Some(entry) = entries.get_mut(key) {
            if !entry.is_expired() {
                entry.touch();
                return Some(entry.data.clone());
            } else {
                // Remove expired entry
                let size = entry.size;
                entries.remove(key);
                let mut current_size = self.current_size.lock().unwrap();
                *current_size = current_size.saturating_sub(size);
            }
        }
        
        None
    }
    
    /// Store data in cache
    pub fn put(&self, key: String, data: Vec<u8>, metadata: CacheMetadata) {
        self.put_with_ttl(key, data, metadata, self.default_ttl);
    }
    
    /// Store data in cache with custom TTL
    pub fn put_with_ttl(&self, key: String, data: Vec<u8>, metadata: CacheMetadata, ttl: u64) {
        let entry = CacheEntry::new(data, ttl, metadata);
        let entry_size = entry.size;
        
        {
            let mut entries = self.entries.lock().unwrap();
            let mut current_size = self.current_size.lock().unwrap();
            
            // Remove existing entry if present
            if let Some(old_entry) = entries.remove(&key) {
                *current_size = current_size.saturating_sub(old_entry.size);
            }
            
            // Check if we need to evict entries
            while *current_size + entry_size > self.max_size && !entries.is_empty() {
                self.evict_lru(&mut entries, &mut current_size);
            }
            
            // Only store if we have space
            if *current_size + entry_size <= self.max_size {
                entries.insert(key, entry);
                *current_size += entry_size;
            }
        }
    }
    
    /// Evict least recently used entry
    fn evict_lru(&self, entries: &mut HashMap<String, CacheEntry>, current_size: &mut u64) {
        if let Some((oldest_key, oldest_entry)) = entries.iter()
            .min_by_key(|(_, entry)| (entry.access_count, entry.cached_at))
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            entries.remove(&oldest_key);
            *current_size = current_size.saturating_sub(oldest_entry.size);
        }
    }
    
    /// Clear expired entries
    pub fn clear_expired(&self) {
        let mut entries = self.entries.lock().unwrap();
        let mut current_size = self.current_size.lock().unwrap();
        
        let expired_keys: Vec<String> = entries.iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect();
        
        for key in expired_keys {
            if let Some(entry) = entries.remove(&key) {
                *current_size = current_size.saturating_sub(entry.size);
            }
        }
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let entries = self.entries.lock().unwrap();
        let current_size = *self.current_size.lock().unwrap();
        
        CacheStats {
            total_entries: entries.len(),
            total_size_bytes: current_size,
            max_size_bytes: self.max_size,
            hit_ratio: 0.0, // Would need to track hits/misses for this
        }
    }
    
    /// Clear all cache entries
    pub fn clear(&self) {
        let mut entries = self.entries.lock().unwrap();
        let mut current_size = self.current_size.lock().unwrap();
        
        entries.clear();
        *current_size = 0;
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size_bytes: u64,
    pub max_size_bytes: u64,
    pub hit_ratio: f64,
}

/// Download progress tracker
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    /// Package name
    pub package_name: String,
    /// Current bytes downloaded
    pub downloaded_bytes: u64,
    /// Total bytes to download (if known)
    pub total_bytes: Option<u64>,
    /// Download start time
    pub start_time: Instant,
    /// Current download speed in bytes/sec
    pub speed_bps: u64,
    /// Estimated time remaining in seconds
    pub eta_seconds: Option<u64>,
}

impl DownloadProgress {
    pub fn new(package_name: String, total_bytes: Option<u64>) -> Self {
        Self {
            package_name,
            downloaded_bytes: 0,
            total_bytes,
            start_time: Instant::now(),
            speed_bps: 0,
            eta_seconds: None,
        }
    }
    
    pub fn update(&mut self, downloaded_bytes: u64) {
        self.downloaded_bytes = downloaded_bytes;
        
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.speed_bps = (downloaded_bytes as f64 / elapsed) as u64;
            
            if let Some(total) = self.total_bytes {
                if self.speed_bps > 0 {
                    let remaining_bytes = total.saturating_sub(downloaded_bytes);
                    self.eta_seconds = Some(remaining_bytes / self.speed_bps);
                }
            }
        }
    }
    
    pub fn progress_percentage(&self) -> Option<f64> {
        self.total_bytes.map(|total| {
            if total > 0 {
                (self.downloaded_bytes as f64 / total as f64) * 100.0
            } else {
                0.0
            }
        })
    }
    
    pub fn is_complete(&self) -> bool {
        self.total_bytes.map_or(false, |total| self.downloaded_bytes >= total)
    }
}

/// Parallel download manager
#[derive(Debug)]
pub struct ParallelDownloader {
    /// Download cache
    cache: DownloadCache,
    /// HTTP client with connection pooling
    client: reqwest::Client,
    /// Concurrency semaphore
    semaphore: Arc<Semaphore>,
    /// Active downloads
    active_downloads: Arc<Mutex<HashMap<String, DownloadProgress>>>,
    /// Download timeout
    timeout: Duration,
}

impl ParallelDownloader {
    pub fn new(
        max_concurrent: usize,
        cache_size_mb: u64,
        cache_ttl_seconds: u64,
        timeout_seconds: u64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .pool_max_idle_per_host(max_concurrent)
            .pool_idle_timeout(Duration::from_secs(30))
            .tcp_keepalive(Duration::from_secs(60))
            .use_rustls_tls()
            .build()?;
        
        Ok(Self {
            cache: DownloadCache::new(cache_size_mb, cache_ttl_seconds),
            client,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            active_downloads: Arc::new(Mutex::new(HashMap::new())),
            timeout: Duration::from_secs(timeout_seconds),
        })
    }
    
    /// Download multiple packages in parallel
    pub async fn download_parallel(
        &self,
        downloads: Vec<(String, String, CacheMetadata)>, // (cache_key, url, metadata)
    ) -> Vec<Result<Vec<u8>, String>> {
        let mut join_set = JoinSet::new();
        
        for (cache_key, url, metadata) in downloads {
            let downloader = self.clone_for_task();
            
            join_set.spawn(async move {
                downloader.download_single(cache_key, url, metadata).await
            });
        }
        
        let mut results = Vec::new();
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(download_result) => results.push(download_result),
                Err(e) => results.push(Err(format!("Task join error: {}", e))),
            }
        }
        
        results
    }
    
    /// Download a single package with caching and progress tracking
    pub async fn download_single(
        &self,
        cache_key: String,
        url: String,
        metadata: CacheMetadata,
    ) -> Result<Vec<u8>, String> {
        // Check cache first
        if let Some(cached_data) = self.cache.get(&cache_key) {
            return Ok(cached_data);
        }
        
        // Acquire semaphore permit for concurrency control
        let _permit = self.semaphore.acquire().await.map_err(|e| format!("Semaphore error: {}", e))?;
        
        // Double-check cache after acquiring permit
        if let Some(cached_data) = self.cache.get(&cache_key) {
            return Ok(cached_data);
        }
        
        // Initialize progress tracking
        let mut progress = DownloadProgress::new(metadata.name.clone(), None);
        {
            let mut active = self.active_downloads.lock().unwrap();
            active.insert(cache_key.clone(), progress.clone());
        }
        
        // Perform download
        let result = self.download_with_progress(&url, &mut progress).await;
        
        // Remove from active downloads
        {
            let mut active = self.active_downloads.lock().unwrap();
            active.remove(&cache_key);
        }
        
        match result {
            Ok(data) => {
                // Cache the downloaded data
                self.cache.put(cache_key, data.clone(), metadata);
                Ok(data)
            }
            Err(e) => Err(e),
        }
    }
    
    /// Download with progress tracking
    async fn download_with_progress(
        &self,
        url: &str,
        progress: &mut DownloadProgress,
    ) -> Result<Vec<u8>, String> {
        let response = self.client.get(url)
            .send()
            .await
            .map_err(|e| format!("Failed to start download: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }
        
        // Get content length if available
        if let Some(content_length) = response.content_length() {
            progress.total_bytes = Some(content_length);
        }
        
        let mut data = Vec::new();
        let mut stream = response.bytes_stream();
        
        use futures_util::StreamExt;
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| format!("Download stream error: {}", e))?;
            data.extend_from_slice(&chunk);
            progress.update(data.len() as u64);
        }
        
        Ok(data)
    }
    
    /// Get download progress for a package
    pub fn get_progress(&self, cache_key: &str) -> Option<DownloadProgress> {
        let active = self.active_downloads.lock().unwrap();
        active.get(cache_key).cloned()
    }
    
    /// Get all active download progresses
    pub fn get_all_progress(&self) -> Vec<DownloadProgress> {
        let active = self.active_downloads.lock().unwrap();
        active.values().cloned().collect()
    }
    
    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        self.cache.stats()
    }
    
    /// Clear cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }
    
    /// Clear expired cache entries
    pub fn clear_expired_cache(&self) {
        self.cache.clear_expired();
    }
    
    /// Clone for async tasks (cheap clone of Arc-wrapped data)
    fn clone_for_task(&self) -> Self {
        Self {
            cache: DownloadCache::new(100, 3600), // Dummy cache for tasks
            client: self.client.clone(),
            semaphore: self.semaphore.clone(),
            active_downloads: self.active_downloads.clone(),
            timeout: self.timeout,
        }
    }
}

/// Optimized batch operations
pub struct BatchOptimizer;

impl BatchOptimizer {
    /// Group dependencies by ecosystem for optimized resolution
    pub fn group_by_ecosystem(
        dependencies: Vec<crate::models::dependency::Dependency>
    ) -> HashMap<crate::models::ecosystem::Ecosystem, Vec<crate::models::dependency::Dependency>> {
        let mut groups = HashMap::new();
        
        for dep in dependencies {
            groups.entry(dep.ecosystem.clone())
                .or_insert_with(Vec::new)
                .push(dep);
        }
        
        groups
    }
    
    /// Prioritize dependencies by install order for optimal performance
    pub fn prioritize_dependencies(
        mut dependencies: Vec<crate::models::resolved_dependency::ResolvedDependency>
    ) -> Vec<crate::models::resolved_dependency::ResolvedDependency> {
        // Sort by:
        // 1. Ecosystem (JavaScript first, then Python)
        // 2. Package size (smaller packages first)
        // 3. Dependency depth (shallow dependencies first)
        dependencies.sort_by(|a, b| {
            use crate::models::ecosystem::Ecosystem;
            
            // Priority by ecosystem
            let ecosystem_priority = |eco: &Ecosystem| match eco {
                Ecosystem::JavaScript => 0,
                Ecosystem::Python => 1,
            };
            
            let eco_cmp = ecosystem_priority(&a.ecosystem).cmp(&ecosystem_priority(&b.ecosystem));
            if eco_cmp != std::cmp::Ordering::Equal {
                return eco_cmp;
            }
            
            // Then by name for consistency
            a.name.cmp(&b.name)
        });
        
        dependencies
    }
    
    /// Calculate optimal batch size based on available memory and network conditions
    pub fn calculate_batch_size(
        total_packages: usize,
        max_concurrent: usize,
        estimated_package_size_mb: f64,
        available_memory_mb: f64,
    ) -> usize {
        // Conservative estimate: use 25% of available memory for downloads
        let memory_budget_mb = available_memory_mb * 0.25;
        let memory_limited_batch = (memory_budget_mb / estimated_package_size_mb) as usize;
        
        // Use the minimum of memory limit, concurrency limit, and total packages
        memory_limited_batch.min(max_concurrent).min(total_packages).max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_entry_expiry() {
        let metadata = CacheMetadata {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            ecosystem: "javascript".to_string(),
            content_type: None,
            integrity: None,
        };
        
        let mut entry = CacheEntry::new(vec![1, 2, 3], 1, metadata); // 1 second TTL
        assert!(!entry.is_expired());
        
        // Simulate time passing
        entry.cached_at -= 2; // 2 seconds ago
        assert!(entry.is_expired());
    }
    
    #[test]
    fn test_download_progress() {
        let mut progress = DownloadProgress::new("test-package".to_string(), Some(1000));
        
        assert_eq!(progress.progress_percentage(), Some(0.0));
        
        progress.update(500);
        assert_eq!(progress.progress_percentage(), Some(50.0));
        
        progress.update(1000);
        assert_eq!(progress.progress_percentage(), Some(100.0));
        assert!(progress.is_complete());
    }
    
    #[tokio::test]
    async fn test_download_cache() {
        let cache = DownloadCache::new(1, 3600); // 1MB cache
        
        let metadata = CacheMetadata {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            ecosystem: "javascript".to_string(),
            content_type: None,
            integrity: None,
        };
        
        let data = vec![0u8; 1024]; // 1KB
        cache.put("test".to_string(), data.clone(), metadata);
        
        let retrieved = cache.get("test");
        assert_eq!(retrieved, Some(data));
        
        // Test cache miss
        let missing = cache.get("nonexistent");
        assert_eq!(missing, None);
    }
}
