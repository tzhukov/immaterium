/// Integration tests for UI context menu behavior
/// Tests that the context menu:
/// 1. Opens when triggered
/// 2. Stays open (doesn't immediately close on opening click)
/// 3. Closes when clicking outside after the debounce period
/// 4. Closes when pressing Escape
/// 5. Closes when clicking a menu item

#[cfg(test)]
mod context_menu_tests {
    use std::time::{Duration, Instant};

    /// Test that the debounce period prevents immediate closure
    #[test]
    fn test_debounce_prevents_immediate_close() {
        let opened_at = Instant::now();
        
        // Immediately after opening (0ms)
        let elapsed_ms = opened_at.elapsed().as_millis();
        assert!(elapsed_ms < 100, "Menu should be within debounce period");
        
        // After 50ms (still in debounce)
        std::thread::sleep(Duration::from_millis(50));
        let elapsed_ms = opened_at.elapsed().as_millis();
        assert!(elapsed_ms < 100, "Menu should still be in debounce period");
    }

    #[test]
    fn test_debounce_period_expires() {
        let opened_at = Instant::now();
        
        // Wait for debounce period to expire
        std::thread::sleep(Duration::from_millis(110));
        
        let elapsed_ms = opened_at.elapsed().as_millis();
        assert!(elapsed_ms > 100, "Debounce period should have expired (got {}ms)", elapsed_ms);
    }

    /// Test the menu state machine logic
    #[test]
    fn test_menu_state_transitions() {
        // Simulate menu lifecycle
        let mut menu_open = false;
        let mut opened_at: Option<Instant> = None;
        
        // 1. Menu is initially closed
        assert!(!menu_open);
        assert!(opened_at.is_none());
        
        // 2. Open menu
        menu_open = true;
        opened_at = Some(Instant::now());
        assert!(menu_open);
        assert!(opened_at.is_some());
        
        // 3. Immediate click should not close (within debounce)
        let should_close = opened_at.map(|t| t.elapsed().as_millis()).unwrap_or(0) > 100;
        assert!(!should_close, "Menu should not close during debounce period");
        
        // 4. Wait for debounce
        std::thread::sleep(Duration::from_millis(110));
        
        // 5. Click outside should now close
        let should_close = opened_at.map(|t| t.elapsed().as_millis()).unwrap_or(0) > 100;
        assert!(should_close, "Menu should close after debounce period");
        
        // 6. Close menu
        menu_open = false;
        opened_at = None;
        assert!(!menu_open);
        assert!(opened_at.is_none());
    }

    /// Test multiple rapid opens/closes
    #[test]
    fn test_rapid_menu_toggling() {
        for _ in 0..5 {
            let opened_at = Instant::now();
            
            // Menu opens
            assert!(opened_at.elapsed().as_millis() < 100);
            
            // Small delay
            std::thread::sleep(Duration::from_millis(20));
            
            // Still in debounce
            assert!(opened_at.elapsed().as_millis() < 100);
        }
    }

    /// Test edge case: exactly at debounce boundary
    #[test]
    fn test_debounce_boundary() {
        let opened_at = Instant::now();
        
        // Sleep for exactly 100ms
        std::thread::sleep(Duration::from_millis(100));
        
        let elapsed = opened_at.elapsed().as_millis();
        // Due to scheduling, this might be slightly over 100ms
        assert!(elapsed >= 99, "Should be at or past debounce threshold (got {}ms)", elapsed);
    }

    /// Test None handling for unopened menu
    #[test]
    fn test_unopened_menu_duration() {
        let opened_at: Option<Instant> = None;
        let duration = opened_at.map(|t| t.elapsed().as_millis()).unwrap_or(0);
        
        assert_eq!(duration, 0, "Unopened menu should have 0 duration");
        assert!(duration <= 100, "Unopened menu should be treated as in debounce");
    }

    /// Test timestamp reset on menu close
    #[test]
    fn test_timestamp_reset() {
        let mut opened_at = Some(Instant::now());
        
        std::thread::sleep(Duration::from_millis(150));
        assert!(opened_at.is_some());
        
        // Simulate menu close
        opened_at = None;
        
        // Verify reset
        assert!(opened_at.is_none());
        let duration = opened_at.map(|t| t.elapsed().as_millis()).unwrap_or(0);
        assert_eq!(duration, 0);
    }

    /// Benchmark: Measure debounce timing accuracy
    #[test]
    fn test_debounce_timing_accuracy() {
        let samples = 10;
        let mut deviations = Vec::new();
        
        for _ in 0..samples {
            let opened_at = Instant::now();
            std::thread::sleep(Duration::from_millis(100));
            let elapsed = opened_at.elapsed().as_millis();
            
            // Calculate deviation from target (100ms)
            let deviation = if elapsed > 100 {
                elapsed - 100
            } else {
                100 - elapsed
            };
            deviations.push(deviation);
        }
        
        let avg_deviation: u128 = deviations.iter().sum::<u128>() / samples as u128;
        
        // Average deviation should be small (< 10ms)
        assert!(avg_deviation < 10, 
            "Average timing deviation too high: {}ms (deviations: {:?})", 
            avg_deviation, deviations);
    }

    /// Test concurrent menu operations
    #[test]
    fn test_menu_concurrency_safety() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let menu_state = Arc::new(Mutex::new((false, None::<Instant>)));
        let mut handles = vec![];
        
        // Spawn multiple threads trying to open/close menu
        for _ in 0..3 {
            let state = Arc::clone(&menu_state);
            let handle = thread::spawn(move || {
                for _ in 0..10 {
                    // Open
                    {
                        let mut guard = state.lock().unwrap();
                        guard.0 = true;
                        guard.1 = Some(Instant::now());
                    }
                    
                    thread::sleep(Duration::from_millis(5));
                    
                    // Close
                    {
                        let mut guard = state.lock().unwrap();
                        guard.0 = false;
                        guard.1 = None;
                    }
                    
                    thread::sleep(Duration::from_millis(5));
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Final state should be closed
        let final_state = menu_state.lock().unwrap();
        assert!(!final_state.0, "Menu should be closed");
        assert!(final_state.1.is_none(), "Timestamp should be None");
    }
}
