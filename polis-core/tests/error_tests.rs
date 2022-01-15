use polis_core::{PolisError, Result};

#[test]
fn test_polis_error_display() {
    let container_error = PolisError::Container("Container not found".to_string());
    assert_eq!(
        format!("{}", container_error),
        "Container error: Container not found"
    );

    let runtime_error = PolisError::Runtime("Runtime error".to_string());
    assert_eq!(format!("{}", runtime_error), "Runtime error: Runtime error");

    let network_error = PolisError::Network("Network error".to_string());
    assert_eq!(format!("{}", network_error), "Network error: Network error");

    let storage_error = PolisError::Storage("Storage error".to_string());
    assert_eq!(format!("{}", storage_error), "Storage error: Storage error");

    let security_error = PolisError::Security("Security error".to_string());
    assert_eq!(
        format!("{}", security_error),
        "Security error: Security error"
    );

    let image_error = PolisError::Image("Image error".to_string());
    assert_eq!(format!("{}", image_error), "Image error: Image error");

    let api_error = PolisError::Api("API error".to_string());
    assert_eq!(format!("{}", api_error), "API error: API error");
}

#[test]
fn test_polis_error_from_io() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let polis_error: PolisError = io_error.into();

    match polis_error {
        PolisError::Io(e) => {
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
            assert_eq!(e.to_string(), "File not found");
        }
        _ => panic!("Expected IO error"),
    }
}

#[test]
fn test_polis_error_from_serde_json() {
    let json_error = serde_json::Error::io(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "Invalid JSON",
    ));
    let polis_error: PolisError = json_error.into();

    match polis_error {
        PolisError::Serialization(e) => {
            assert_eq!(e.to_string(), "Invalid JSON");
        }
        _ => panic!("Expected Serialization error"),
    }
}

#[test]
fn test_result_type() {
    fn success_function() -> Result<String> {
        Ok("success".to_string())
    }

    fn error_function() -> Result<String> {
        Err(PolisError::Container("Container error".to_string()))
    }

    // Test success case
    match success_function() {
        Ok(value) => assert_eq!(value, "success"),
        Err(_) => panic!("Expected success"),
    }

    // Test error case
    match error_function() {
        Ok(_) => panic!("Expected error"),
        Err(e) => match e {
            PolisError::Container(msg) => assert_eq!(msg, "Container error"),
            _ => panic!("Expected Container error"),
        },
    }
}

#[test]
fn test_error_chaining() {
    fn io_operation() -> Result<String> {
        std::fs::read_to_string("nonexistent_file.txt")?;
        Ok("success".to_string())
    }

    match io_operation() {
        Ok(_) => panic!("Expected error"),
        Err(e) => match e {
            PolisError::Io(io_err) => {
                assert_eq!(io_err.kind(), std::io::ErrorKind::NotFound);
            }
            _ => panic!("Expected IO error"),
        },
    }
}

#[test]
fn test_error_debug() {
    let error = PolisError::Container("Test error".to_string());
    let debug_string = format!("{:?}", error);
    assert!(debug_string.contains("Container"));
    assert!(debug_string.contains("Test error"));
}

#[test]
fn test_error_pattern_matching() {
    let error1 = PolisError::Container("Same error".to_string());
    let error2 = PolisError::Container("Different error".to_string());
    let error3 = PolisError::Runtime("Same error".to_string());

    // Test pattern matching for error types
    match error1 {
        PolisError::Container(msg) => assert_eq!(msg, "Same error"),
        _ => panic!("Expected Container error"),
    }

    match error2 {
        PolisError::Container(msg) => assert_eq!(msg, "Different error"),
        _ => panic!("Expected Container error"),
    }

    match error3 {
        PolisError::Runtime(msg) => assert_eq!(msg, "Same error"),
        _ => panic!("Expected Runtime error"),
    }
}

#[test]
fn test_error_conversion() {
    // Test converting from different error types
    let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Permission denied");
    let polis_error: PolisError = io_error.into();

    assert!(matches!(polis_error, PolisError::Io(_)));

    let json_error = serde_json::Error::io(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "Invalid JSON format",
    ));
    let polis_error: PolisError = json_error.into();

    assert!(matches!(polis_error, PolisError::Serialization(_)));
}
