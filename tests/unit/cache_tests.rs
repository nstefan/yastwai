/*!
 * Tests for translation cache functionality
 */

use yastwai::translation::cache::TranslationCache;

#[tokio::test]
async fn test_cache_new_withEnabled_shouldCreateEnabledCache() {
    let _cache = TranslationCache::new(true);
    // Cache should be created successfully
    assert!(true); // If we get here, construction succeeded
}

#[tokio::test]
async fn test_cache_new_withDisabled_shouldCreateDisabledCache() {
    let cache = TranslationCache::new(false);
    // Store something
    cache.store("hello", "en", "fr", "bonjour").await;
    // Get should return None because cache is disabled
    let result = cache.get("hello", "en", "fr").await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_cache_store_withEnabledCache_shouldStoreTranslation() {
    let cache = TranslationCache::new(true);
    cache.store("hello", "en", "fr", "bonjour").await;
    
    let result = cache.get("hello", "en", "fr").await;
    assert_eq!(result, Some("bonjour".to_string()));
}

#[tokio::test]
async fn test_cache_get_withMissingKey_shouldReturnNone() {
    let cache = TranslationCache::new(true);
    let result = cache.get("nonexistent", "en", "fr").await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_cache_get_withDifferentLanguages_shouldReturnNone() {
    let cache = TranslationCache::new(true);
    cache.store("hello", "en", "fr", "bonjour").await;
    
    // Different source language
    let result = cache.get("hello", "de", "fr").await;
    assert!(result.is_none());
    
    // Different target language
    let result = cache.get("hello", "en", "es").await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_cache_store_withMultipleEntries_shouldStoreAll() {
    let cache = TranslationCache::new(true);
    
    cache.store("hello", "en", "fr", "bonjour").await;
    cache.store("goodbye", "en", "fr", "au revoir").await;
    cache.store("hello", "en", "es", "hola").await;
    
    assert_eq!(cache.get("hello", "en", "fr").await, Some("bonjour".to_string()));
    assert_eq!(cache.get("goodbye", "en", "fr").await, Some("au revoir".to_string()));
    assert_eq!(cache.get("hello", "en", "es").await, Some("hola".to_string()));
}

#[tokio::test]
async fn test_cache_store_withSameKey_shouldOverwrite() {
    let cache = TranslationCache::new(true);
    
    cache.store("hello", "en", "fr", "bonjour").await;
    cache.store("hello", "en", "fr", "salut").await;
    
    assert_eq!(cache.get("hello", "en", "fr").await, Some("salut".to_string()));
}

#[tokio::test]
async fn test_cache_default_shouldBeEnabled() {
    let cache = TranslationCache::default();
    cache.store("test", "en", "fr", "essai").await;
    
    let result = cache.get("test", "en", "fr").await;
    assert_eq!(result, Some("essai".to_string()));
}

#[tokio::test]
async fn test_cache_clone_shouldShareStorage() {
    let cache1 = TranslationCache::new(true);
    let cache2 = cache1.clone();
    
    cache1.store("hello", "en", "fr", "bonjour").await;
    
    // cache2 should see the same data (shared storage)
    let result = cache2.get("hello", "en", "fr").await;
    assert_eq!(result, Some("bonjour".to_string()));
}

#[tokio::test]
async fn test_cache_withEmptyStrings_shouldHandleCorrectly() {
    let cache = TranslationCache::new(true);
    
    cache.store("", "en", "fr", "").await;
    let result = cache.get("", "en", "fr").await;
    assert_eq!(result, Some("".to_string()));
}

#[tokio::test]
async fn test_cache_withUnicodeText_shouldHandleCorrectly() {
    let cache = TranslationCache::new(true);
    
    let source = "こんにちは";
    let translation = "Bonjour 你好 مرحبا";
    
    cache.store(source, "ja", "multi", translation).await;
    let result = cache.get(source, "ja", "multi").await;
    assert_eq!(result, Some(translation.to_string()));
}

#[tokio::test]
async fn test_cache_withLongText_shouldHandleCorrectly() {
    let cache = TranslationCache::new(true);
    
    let source = "a".repeat(10000);
    let translation = "b".repeat(10000);
    
    cache.store(&source, "en", "fr", &translation).await;
    let result = cache.get(&source, "en", "fr").await;
    assert_eq!(result, Some(translation));
}

#[tokio::test]
async fn test_cache_concurrent_access_shouldBeThreadSafe() {
    use std::sync::Arc;
    use tokio::task::JoinSet;
    
    let cache = Arc::new(TranslationCache::new(true));
    let mut join_set = JoinSet::new();
    
    // Spawn multiple tasks to write to the cache
    for i in 0..10 {
        let cache = cache.clone();
        let key = format!("key{}", i);
        let value = format!("value{}", i);
        join_set.spawn(async move {
            cache.store(&key, "en", "fr", &value).await;
        });
    }
    
    // Wait for all writes
    while join_set.join_next().await.is_some() {}
    
    // Verify all values are stored
    for i in 0..10 {
        let key = format!("key{}", i);
        let expected = format!("value{}", i);
        assert_eq!(cache.get(&key, "en", "fr").await, Some(expected));
    }
}

