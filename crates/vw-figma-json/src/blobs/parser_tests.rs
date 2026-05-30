use super::*;

#[test]
fn test_parse_commands_simple_path() {
    // M 10 20 L 30 40 Z
    let mut bytes = Vec::new();
    bytes.push(1); // M
    bytes.extend_from_slice(&10.0f32.to_le_bytes());
    bytes.extend_from_slice(&20.0f32.to_le_bytes());
    bytes.push(2); // L
    bytes.extend_from_slice(&30.0f32.to_le_bytes());
    bytes.extend_from_slice(&40.0f32.to_le_bytes());
    bytes.push(0); // Z

    let result = parse_commands(&bytes).unwrap();
    let arr = result.as_array().unwrap();

    assert_eq!(arr.len(), 7);
    assert_eq!(arr[0].as_str(), Some("M"));
    assert_eq!(arr[1].as_f64(), Some(10.0));
    assert_eq!(arr[2].as_f64(), Some(20.0));
    assert_eq!(arr[3].as_str(), Some("L"));
    assert_eq!(arr[4].as_f64(), Some(30.0));
    assert_eq!(arr[5].as_f64(), Some(40.0));
    assert_eq!(arr[6].as_str(), Some("Z"));
}

#[test]
fn test_parse_commands_quadratic() {
    // Q 1 2 3 4
    let mut bytes = Vec::new();
    bytes.push(3); // Q
    bytes.extend_from_slice(&1.0f32.to_le_bytes());
    bytes.extend_from_slice(&2.0f32.to_le_bytes());
    bytes.extend_from_slice(&3.0f32.to_le_bytes());
    bytes.extend_from_slice(&4.0f32.to_le_bytes());

    let result = parse_commands(&bytes).unwrap();
    let arr = result.as_array().unwrap();

    assert_eq!(arr.len(), 5);
    assert_eq!(arr[0].as_str(), Some("Q"));
    assert_eq!(arr[1].as_f64(), Some(1.0));
    assert_eq!(arr[2].as_f64(), Some(2.0));
    assert_eq!(arr[3].as_f64(), Some(3.0));
    assert_eq!(arr[4].as_f64(), Some(4.0));
}

#[test]
fn test_parse_commands_cubic() {
    // C 1 2 3 4 5 6
    let mut bytes = Vec::new();
    bytes.push(4); // C
    bytes.extend_from_slice(&1.0f32.to_le_bytes());
    bytes.extend_from_slice(&2.0f32.to_le_bytes());
    bytes.extend_from_slice(&3.0f32.to_le_bytes());
    bytes.extend_from_slice(&4.0f32.to_le_bytes());
    bytes.extend_from_slice(&5.0f32.to_le_bytes());
    bytes.extend_from_slice(&6.0f32.to_le_bytes());

    let result = parse_commands(&bytes).unwrap();
    let arr = result.as_array().unwrap();

    assert_eq!(arr.len(), 7);
    assert_eq!(arr[0].as_str(), Some("C"));
    assert_eq!(arr[1].as_f64(), Some(1.0));
    assert_eq!(arr[2].as_f64(), Some(2.0));
    assert_eq!(arr[3].as_f64(), Some(3.0));
    assert_eq!(arr[4].as_f64(), Some(4.0));
    assert_eq!(arr[5].as_f64(), Some(5.0));
    assert_eq!(arr[6].as_f64(), Some(6.0));
}

#[test]
fn test_parse_commands_invalid() {
    // 无效的命令类型
    let bytes = vec![99];
    assert!(parse_commands(&bytes).is_none());

    // 数据不完整
    let bytes = vec![1, 0]; // M with incomplete coordinates
    assert!(parse_commands(&bytes).is_none());
}

#[test]
fn test_parse_vector_network_simple() {
    let mut bytes = Vec::new();

    // 标头：2 个顶点，1 个线段，0 个区域
    bytes.extend_from_slice(&2u32.to_le_bytes());
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());

    // 顶点 0：styleID=0，x=10，y=20
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&10.0f32.to_le_bytes());
    bytes.extend_from_slice(&20.0f32.to_le_bytes());

    // 顶点 1：styleID=0，x=30，y=40
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&30.0f32.to_le_bytes());
    bytes.extend_from_slice(&40.0f32.to_le_bytes());

    // 段 0：styleID=0，开始=(顶点=0，dx=0，dy=0)，结束=(顶点=1，dx=0，dy=0)
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0.0f32.to_le_bytes());
    bytes.extend_from_slice(&0.0f32.to_le_bytes());
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&0.0f32.to_le_bytes());
    bytes.extend_from_slice(&0.0f32.to_le_bytes());

    let result = parse_vector_network(&bytes).unwrap();

    assert!(result.get("vertices").is_some());
    assert!(result.get("segments").is_some());
    assert!(result.get("regions").is_some());

    let vertices = result.get("vertices").unwrap().as_array().unwrap();
    assert_eq!(vertices.len(), 2);

    let segments = result.get("segments").unwrap().as_array().unwrap();
    assert_eq!(segments.len(), 1);

    let regions = result.get("regions").unwrap().as_array().unwrap();
    assert_eq!(regions.len(), 0);
}
