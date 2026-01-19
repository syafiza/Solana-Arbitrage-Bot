/// Object Pool for Memory Optimization
/// 
/// Provides object pooling to reduce allocations in hot paths.

use std::sync::{Arc, Mutex};

/// Generic object pool
pub struct ObjectPool<T> {
    objects: Arc<Mutex<Vec<T>>>,
    factory: Arc<dyn Fn() -> T + Send + Sync>,
}

impl<T> ObjectPool<T> {
    /// Create a new object pool
    pub fn new<F>(factory: F, initial_capacity: usize) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        let factory = Arc::new(factory);
        let mut objects = Vec::with_capacity(initial_capacity);
        
        // Pre-allocate objects
        for _ in 0..initial_capacity {
            objects.push(factory());
        }

        Self {
            objects: Arc::new(Mutex::new(objects)),
            factory,
        }
    }

    /// Acquire an object from the pool
    pub fn acquire(&self) -> PooledObject<T> {
        let obj = {
            let mut pool = self.objects.lock().unwrap();
            pool.pop().unwrap_or_else(|| (self.factory)())
        };

        PooledObject {
            obj: Some(obj),
            pool: self.objects.clone(),
        }
    }

    /// Get current pool size
    pub fn size(&self) -> usize {
        self.objects.lock().unwrap().len()
    }
}

/// RAII wrapper that returns object to pool when dropped
pub struct PooledObject<T> {
    obj: Option<T>,
    pool: Arc<Mutex<Vec<T>>>,
}

impl<T> PooledObject<T> {
    /// Get mutable reference to the pooled object
    pub fn as_mut(&mut self) -> &mut T {
        self.obj.as_mut().unwrap()
    }
}

impl<T> std::ops::Deref for PooledObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.obj.as_ref().unwrap()
    }
}

impl<T> std::ops::DerefMut for PooledObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.obj.as_mut().unwrap()
    }
}

impl<T> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(obj) = self.obj.take() {
            let mut pool = self.pool.lock().unwrap();
            pool.push(obj);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_pool_acquire_and_return() {
        let pool = ObjectPool::new(|| Vec::<u8>::with_capacity(1024), 2);
        
        assert_eq!(pool.size(), 2);
        
        {
            let obj1 = pool.acquire();
            assert_eq!(pool.size(), 1);
            
            let obj2 = pool.acquire();
            assert_eq!(pool.size(), 0);
        }
        
        // Objects returned to pool
        assert_eq!(pool.size(), 2);
    }

    #[test]
    fn test_object_pool_overflow() {
        let pool = ObjectPool::new(|| Vec::<u8>::new(), 1);
        
        let _obj1 = pool.acquire();
        let _obj2 = pool.acquire(); // Creates new object since pool empty
        
        assert_eq!(pool.size(), 0);
    }
}
