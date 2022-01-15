use anyhow::Result;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use sysinfo::{ProcessExt, System, SystemExt};

/// Memory allocator wrapper for tracking allocations
pub struct TrackingAllocator {
    inner: System,
    allocated: AtomicUsize,
    peak: AtomicUsize,
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc(layout);
        if !ptr.is_null() {
            let size = layout.size();
            let current = self.allocated.fetch_add(size, Ordering::Relaxed);
            let peak = self.peak.load(Ordering::Relaxed);
            if current + size > peak {
                self.peak.store(current + size, Ordering::Relaxed);
            }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner.dealloc(ptr, layout);
        let size = layout.size();
        self.allocated.fetch_sub(size, Ordering::Relaxed);
    }
}

impl TrackingAllocator {
    pub fn new() -> Self {
        Self {
            inner: System,
            allocated: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
        }
    }

    pub fn allocated_bytes(&self) -> usize {
        self.allocated.load(Ordering::Relaxed)
    }

    pub fn peak_bytes(&self) -> usize {
        self.peak.load(Ordering::Relaxed)
    }
}

/// Memory pool for efficient allocation
pub struct MemoryPool {
    blocks: Vec<Vec<u8>>,
    block_size: usize,
    max_blocks: usize,
    available: Vec<usize>,
}

impl MemoryPool {
    pub fn new(block_size: usize, max_blocks: usize) -> Self {
        Self {
            blocks: Vec::with_capacity(max_blocks),
            block_size,
            max_blocks,
            available: Vec::with_capacity(max_blocks),
        }
    }

    pub fn allocate(&mut self) -> Option<&mut [u8]> {
        if let Some(index) = self.available.pop() {
            Some(&mut self.blocks[index])
        } else if self.blocks.len() < self.max_blocks {
            let block = vec![0u8; self.block_size];
            let index = self.blocks.len();
            self.blocks.push(block);
            Some(&mut self.blocks[index])
        } else {
            None
        }
    }

    pub fn deallocate(&mut self, block: &mut [u8]) {
        for (i, b) in self.blocks.iter_mut().enumerate() {
            if b.as_mut_ptr() == block.as_mut_ptr() {
                self.available.push(i);
                break;
            }
        }
    }
}

/// Memory usage monitor
pub struct MemoryMonitor {
    system: System,
    process_id: u32,
    baseline_memory: usize,
}

impl MemoryMonitor {
    pub fn new() -> Result<Self> {
        let mut system = System::new_all();
        system.refresh_all();

        let process_id = std::process::id();

        Ok(Self {
            system,
            process_id,
            baseline_memory: 0,
        })
    }

    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }

    pub fn current_memory_usage(&mut self) -> usize {
        self.refresh();
        if let Some(process) = self.system.process(self.process_id) {
            process.memory() as usize
        } else {
            0
        }
    }

    pub fn memory_delta(&mut self) -> isize {
        let current = self.current_memory_usage();
        let delta = current as isize - self.baseline_memory as isize;
        self.baseline_memory = current;
        delta
    }

    pub fn memory_usage_percentage(&mut self) -> f64 {
        self.refresh();
        if let Some(process) = self.system.process(self.process_id) {
            process.cpu_usage() as f64
        } else {
            0.0
        }
    }

    pub fn system_memory_info(&mut self) -> (usize, usize) {
        self.refresh();
        (
            self.system.total_memory() as usize,
            self.system.used_memory() as usize,
        )
    }
}

/// Memory optimization strategies
pub struct MemoryOptimizer {
    monitor: MemoryMonitor,
    gc_threshold: usize,
    compression_enabled: bool,
}

impl MemoryOptimizer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            monitor: MemoryMonitor::new()?,
            gc_threshold: 100 * 1024 * 1024, // 100MB
            compression_enabled: true,
        })
    }

    pub fn should_garbage_collect(&mut self) -> bool {
        self.monitor.current_memory_usage() > self.gc_threshold
    }

    pub fn optimize_memory(&mut self) -> Result<()> {
        if self.should_garbage_collect() {
            self.force_garbage_collect();
        }

        if self.compression_enabled {
            self.compress_memory();
        }

        Ok(())
    }

    fn force_garbage_collect(&self) {
        // Force garbage collection by dropping unused allocations
        // This is a simplified version - in practice, you'd implement
        // proper garbage collection based on your data structures
        std::hint::black_box(());
    }

    fn compress_memory(&self) {
        // Implement memory compression for large data structures
        // This would compress unused or rarely accessed memory
    }

    pub fn set_gc_threshold(&mut self, threshold: usize) {
        self.gc_threshold = threshold;
    }

    pub fn enable_compression(&mut self, enabled: bool) {
        self.compression_enabled = enabled;
    }
}

/// Memory-efficient string interning
pub struct StringInterner {
    strings: std::collections::HashMap<String, u32>,
    reverse: std::collections::HashMap<u32, String>,
    next_id: u32,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            strings: std::collections::HashMap::new(),
            reverse: std::collections::HashMap::new(),
            next_id: 0,
        }
    }

    pub fn intern(&mut self, s: &str) -> u32 {
        if let Some(&id) = self.strings.get(s) {
            id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            self.strings.insert(s.to_string(), id);
            self.reverse.insert(id, s.to_string());
            id
        }
    }

    pub fn get(&self, id: u32) -> Option<&str> {
        self.reverse.get(&id).map(|s| s.as_str())
    }

    pub fn memory_usage(&self) -> usize {
        self.strings.len() * std::mem::size_of::<String>()
            + self.reverse.len() * std::mem::size_of::<String>()
    }
}

/// Memory-efficient container for small objects
pub struct SmallVec<T, const N: usize> {
    data: [T; N],
    len: usize,
}

impl<T: Default + Copy, const N: usize> SmallVec<T, N> {
    pub fn new() -> Self {
        Self {
            data: [T::default(); N],
            len: 0,
        }
    }

    pub fn push(&mut self, item: T) -> Result<()> {
        if self.len < N {
            self.data[self.len] = item;
            self.len += 1;
            Ok(())
        } else {
            Err(anyhow::anyhow!("SmallVec is full"))
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len > 0 {
            self.len -= 1;
            Some(self.data[self.len])
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data[..self.len].iter()
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub current_usage: usize,
    pub peak_usage: usize,
    pub allocated_blocks: usize,
    pub freed_blocks: usize,
    pub fragmentation: f64,
}

impl MemoryStats {
    pub fn new() -> Self {
        Self {
            current_usage: 0,
            peak_usage: 0,
            allocated_blocks: 0,
            freed_blocks: 0,
            fragmentation: 0.0,
        }
    }

    pub fn update(&mut self, current: usize, peak: usize) {
        self.current_usage = current;
        self.peak_usage = peak;
    }

    pub fn add_allocation(&mut self, size: usize) {
        self.allocated_blocks += 1;
        self.current_usage += size;
        if self.current_usage > self.peak_usage {
            self.peak_usage = self.current_usage;
        }
    }

    pub fn add_deallocation(&mut self, size: usize) {
        self.freed_blocks += 1;
        self.current_usage = self.current_usage.saturating_sub(size);
    }

    pub fn calculate_fragmentation(&mut self) {
        if self.allocated_blocks > 0 {
            self.fragmentation = (self.freed_blocks as f64) / (self.allocated_blocks as f64);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_pool() {
        let mut pool = MemoryPool::new(1024, 10);

        let block1 = pool.allocate().unwrap();
        assert_eq!(block1.len(), 1024);

        let block2 = pool.allocate().unwrap();
        assert_eq!(block2.len(), 1024);

        pool.deallocate(block1);
        let block3 = pool.allocate().unwrap();
        assert_eq!(block3.len(), 1024);
    }

    #[test]
    fn test_string_interner() {
        let mut interner = StringInterner::new();

        let id1 = interner.intern("hello");
        let id2 = interner.intern("world");
        let id3 = interner.intern("hello");

        assert_eq!(id1, id3);
        assert_ne!(id1, id2);
        assert_eq!(interner.get(id1), Some("hello"));
        assert_eq!(interner.get(id2), Some("world"));
    }

    #[test]
    fn test_small_vec() {
        let mut vec = SmallVec::<i32, 4>::new();

        assert!(vec.is_empty());
        assert_eq!(vec.len(), 0);

        vec.push(1).unwrap();
        vec.push(2).unwrap();

        assert_eq!(vec.len(), 2);
        assert!(!vec.is_empty());

        let items: Vec<i32> = vec.iter().copied().collect();
        assert_eq!(items, vec![1, 2]);

        assert_eq!(vec.pop(), Some(2));
        assert_eq!(vec.pop(), Some(1));
        assert_eq!(vec.pop(), None);
    }
}
