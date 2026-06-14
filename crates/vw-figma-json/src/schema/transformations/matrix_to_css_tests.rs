use super::*;
use serde_json::json;

// 用于比较浮点数与公差的辅助函数
fn approx_eq(a: f64, b: f64, epsilon: f64) -> bool {
    (a - b).abs() < epsilon
}

#[test]
fn test_identity_matrix() {
    let mut tree = json!({
        "transform": {
            "m00": 1.0,
            "m01": 0.0,
            "m02": 0.0,
            "m10": 0.0,
            "m11": 1.0,
            "m12": 0.0
        }
    });

    transform_matrix_to_css(&mut tree).unwrap();

    let transform = tree.get("transform").unwrap();
    assert!(approx_eq(transform["x"].as_f64().unwrap(), 0.0, 1e-10));
    assert!(approx_eq(transform["y"].as_f64().unwrap(), 0.0, 1e-10));
    // 不应出现默认值
    assert!(transform.get("rotation").is_none());
    assert!(transform.get("scaleX").is_none());
    assert!(transform.get("scaleY").is_none());
    assert!(transform.get("skewX").is_none());
}

#[test]
fn test_pure_translation() {
    let mut tree = json!({
        "transform": {
            "m00": 1.0,
            "m01": 0.0,
            "m02": 100.0,
            "m10": 0.0,
            "m11": 1.0,
            "m12": 50.0
        }
    });

    transform_matrix_to_css(&mut tree).unwrap();

    let transform = tree.get("transform").unwrap();
    assert!(approx_eq(transform["x"].as_f64().unwrap(), 100.0, 1e-10));
    assert!(approx_eq(transform["y"].as_f64().unwrap(), 50.0, 1e-10));
    // 不应出现默认值
    assert!(transform.get("rotation").is_none());
    assert!(transform.get("scaleX").is_none());
    assert!(transform.get("scaleY").is_none());
    assert!(transform.get("skewX").is_none());
}

#[test]
fn test_pure_scale() {
    let mut tree = json!({
        "transform": {
            "m00": 2.0,
            "m01": 0.0,
            "m02": 0.0,
            "m10": 0.0,
            "m11": 3.0,
            "m12": 0.0
        }
    });

    transform_matrix_to_css(&mut tree).unwrap();

    let transform = tree.get("transform").unwrap();
    assert!(approx_eq(transform["x"].as_f64().unwrap(), 0.0, 1e-10));
    assert!(approx_eq(transform["y"].as_f64().unwrap(), 0.0, 1e-10));
    assert!(approx_eq(transform["scaleX"].as_f64().unwrap(), 2.0, 1e-10));
    assert!(approx_eq(transform["scaleY"].as_f64().unwrap(), 3.0, 1e-10));
    // 不应出现默认旋转和倾斜
    assert!(transform.get("rotation").is_none());
    assert!(transform.get("skewX").is_none());
}

#[test]
fn test_pure_rotation_45_degrees() {
    // 45度旋转：cos(45°) ≈ 0.7071，sin(45°) ≈ 0.7071
    let cos45 = std::f64::consts::FRAC_1_SQRT_2;
    let sin45 = std::f64::consts::FRAC_1_SQRT_2;

    let mut tree = json!({
        "transform": {
            "m00": cos45,
            "m01": -sin45,
            "m02": 0.0,
            "m10": sin45,
            "m11": cos45,
            "m12": 0.0
        }
    });

    transform_matrix_to_css(&mut tree).unwrap();

    let transform = tree.get("transform").unwrap();
    assert!(approx_eq(transform["x"].as_f64().unwrap(), 0.0, 1e-10));
    assert!(approx_eq(transform["y"].as_f64().unwrap(), 0.0, 1e-10));
    assert!(approx_eq(transform["rotation"].as_f64().unwrap(), 45.0, 1e-8));
    // 不应出现默认比例和倾斜
    assert!(transform.get("scaleX").is_none());
    assert!(transform.get("scaleY").is_none());
    assert!(transform.get("skewX").is_none());
}

#[test]
fn test_pure_rotation_90_degrees() {
    // 90 度旋转：cos(90°) = 0，sin(90°) = 1
    let mut tree = json!({
        "transform": {
            "m00": 0.0,
            "m01": -1.0,
            "m02": 0.0,
            "m10": 1.0,
            "m11": 0.0,
            "m12": 0.0
        }
    });

    transform_matrix_to_css(&mut tree).unwrap();

    let transform = tree.get("transform").unwrap();
    assert!(approx_eq(transform["x"].as_f64().unwrap(), 0.0, 1e-10));
    assert!(approx_eq(transform["y"].as_f64().unwrap(), 0.0, 1e-10));
    assert!(approx_eq(transform["rotation"].as_f64().unwrap(), 90.0, 1e-8));
    // 不应出现默认比例和倾斜
    assert!(transform.get("scaleX").is_none());
    assert!(transform.get("scaleY").is_none());
    assert!(transform.get("skewX").is_none());
}

#[test]
fn test_combined_translation_scale_rotation() {
    // 30 度旋转，比例为 2x, 3y，翻译为 (100, 50)
    let angle = 30.0 * PI / 180.0;
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    let sx = 2.0;
    let sy = 3.0;

    let mut tree = json!({
        "transform": {
            "m00": sx * cos_a,
            "m01": -sy * sin_a,
            "m02": 100.0,
            "m10": sx * sin_a,
            "m11": sy * cos_a,
            "m12": 50.0
        }
    });

    transform_matrix_to_css(&mut tree).unwrap();

    let transform = tree.get("transform").unwrap();
    assert!(approx_eq(transform["x"].as_f64().unwrap(), 100.0, 1e-10));
    assert!(approx_eq(transform["y"].as_f64().unwrap(), 50.0, 1e-10));
    assert!(approx_eq(transform["rotation"].as_f64().unwrap(), 30.0, 1e-8));
    assert!(approx_eq(transform["scaleX"].as_f64().unwrap(), 2.0, 1e-10));
    assert!(approx_eq(transform["scaleY"].as_f64().unwrap(), 3.0, 1e-10));
    // 不应出现默认倾斜
    assert!(transform.get("skewX").is_none());
}

#[test]
fn test_nested_objects() {
    let mut tree = json!({
        "name": "Root",
        "transform": {
            "m00": 1.0,
            "m01": 0.0,
            "m02": 10.0,
            "m10": 0.0,
            "m11": 1.0,
            "m12": 20.0
        },
        "children": [
            {
                "name": "Child1",
                "transform": {
                    "m00": 2.0,
                    "m01": 0.0,
                    "m02": 5.0,
                    "m10": 0.0,
                    "m11": 2.0,
                    "m12": 10.0
                }
            }
        ]
    });

    transform_matrix_to_css(&mut tree).unwrap();

    // 检查根变换(仅平移，应该只有 x 和 y)
    let root_transform = tree.get("transform").unwrap();
    assert!(approx_eq(root_transform["x"].as_f64().unwrap(), 10.0, 1e-10));
    assert!(approx_eq(root_transform["y"].as_f64().unwrap(), 20.0, 1e-10));
    assert!(root_transform.get("rotation").is_none());
    assert!(root_transform.get("scaleX").is_none());
    assert!(root_transform.get("scaleY").is_none());
    assert!(root_transform.get("skewX").is_none());

    // 检查子变换(有比例，应该有x，y，scaleX，scaleY)
    let child_transform = &tree["children"][0]["transform"];
    assert!(approx_eq(child_transform["x"].as_f64().unwrap(), 5.0, 1e-10));
    assert!(approx_eq(child_transform["y"].as_f64().unwrap(), 10.0, 1e-10));
    assert!(approx_eq(child_transform["scaleX"].as_f64().unwrap(), 2.0, 1e-10));
    assert!(approx_eq(child_transform["scaleY"].as_f64().unwrap(), 2.0, 1e-10));
    assert!(child_transform.get("rotation").is_none());
    assert!(child_transform.get("skewX").is_none());
}

#[test]
fn test_non_transform_object_unchanged() {
    let mut tree = json!({
        "name": "Rectangle",
        "position": {
            "x": 10,
            "y": 20
        }
    });

    let original = tree.clone();
    transform_matrix_to_css(&mut tree).unwrap();

    // 应保持不变
    assert_eq!(tree, original);
}

#[test]
fn test_transform_without_matrix_fields_unchanged() {
    let mut tree = json!({
        "transform": {
            "x": 10,
            "y": 20
        }
    });

    let original = tree.clone();
    transform_matrix_to_css(&mut tree).unwrap();

    // 应保持不变，因为它没有矩阵字段
    assert_eq!(tree, original);
}

#[test]
fn test_negative_scale() {
    // 负比例(反射)
    let mut tree = json!({
        "transform": {
            "m00": -1.0,
            "m01": 0.0,
            "m02": 0.0,
            "m10": 0.0,
            "m11": 1.0,
            "m12": 0.0
        }
    });

    transform_matrix_to_css(&mut tree).unwrap();

    let transform = tree.get("transform").unwrap();
    // x 和 y 应该存在
    assert!(approx_eq(transform["x"].as_f64().unwrap(), 0.0, 1e-10));
    assert!(approx_eq(transform["y"].as_f64().unwrap(), 0.0, 1e-10));
    // scaleX应为1.0(大小)，旋转应为180度
    assert!(approx_eq(transform["rotation"].as_f64().unwrap(), 180.0, 1e-8));
    assert!(approx_eq(transform["scaleY"].as_f64().unwrap(), -1.0, 1e-10));
    // scaleX 为 1.0(默认)，因此不应存在
    assert!(transform.get("scaleX").is_none());
    assert!(transform.get("skewX").is_none());
}

#[test]
fn test_real_world_example() {
    // 来自实际的 example.canvas.fig：translate(248, -7)
    let mut tree = json!({
        "transform": {
            "m00": 1.0,
            "m01": 0.0,
            "m02": 248.0,
            "m10": 0.0,
            "m11": 1.0,
            "m12": -7.0
        }
    });

    transform_matrix_to_css(&mut tree).unwrap();

    let transform = tree.get("transform").unwrap();
    assert!(approx_eq(transform["x"].as_f64().unwrap(), 248.0, 1e-10));
    assert!(approx_eq(transform["y"].as_f64().unwrap(), -7.0, 1e-10));
    // 不应出现默认值
    assert!(transform.get("rotation").is_none());
    assert!(transform.get("scaleX").is_none());
    assert!(transform.get("scaleY").is_none());
    assert!(transform.get("skewX").is_none());
}

#[test]
fn test_skew_x_matrix() {
    let mut tree = json!({
        "transform": {
            "m00": 1.0,
            "m01": 1.0,
            "m02": 0.0,
            "m10": 0.0,
            "m11": 1.0,
            "m12": 0.0
        }
    });

    transform_matrix_to_css(&mut tree).unwrap();

    let transform = tree.get("transform").unwrap();
    assert!(approx_eq(transform["skewX"].as_f64().unwrap(), 45.0, 1e-8));
    assert!(transform.get("rotation").is_none());
    assert!(transform.get("scaleX").is_none());
    assert!(transform.get("scaleY").is_none());
}

#[test]
fn test_zero_scale_x_uses_second_column_scale() {
    let mut tree = json!({
        "transform": {
            "m00": 0.0,
            "m01": 3.0,
            "m02": 7.0,
            "m10": 0.0,
            "m11": 4.0,
            "m12": 9.0
        }
    });

    transform_matrix_to_css(&mut tree).unwrap();

    let transform = tree.get("transform").unwrap();
    assert!(approx_eq(transform["x"].as_f64().unwrap(), 7.0, 1e-10));
    assert!(approx_eq(transform["y"].as_f64().unwrap(), 9.0, 1e-10));
    assert!(approx_eq(transform["scaleX"].as_f64().unwrap(), 0.0, 1e-10));
    assert!(approx_eq(transform["scaleY"].as_f64().unwrap(), 5.0, 1e-10));
    assert!(transform.get("rotation").is_none());
    assert!(transform.get("skewX").is_none());
}

#[test]
fn test_invalid_matrix_value_preserves_transform() {
    let mut tree = json!({
        "transform": {
            "m00": "1",
            "m01": 0.0,
            "m02": 0.0,
            "m10": 0.0,
            "m11": 1.0,
            "m12": 0.0
        }
    });
    let original = tree.clone();

    transform_matrix_to_css(&mut tree).unwrap();

    assert_eq!(tree, original);
}
