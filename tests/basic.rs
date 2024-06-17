
#[cfg(test)]
mod tests {

    #[plbindgen_macros::opaque]
    struct Point {
        x: i32,
        y: i32,
    }

    #[plbindgen_macros::opaque]
    struct Window(i32, i32, i32, i32);

    #[plbindgen_macros::export]
    unsafe fn avg(nums: &[i32], nums_len: usize) -> i32 {
        let sum: i32 = nums.iter().sum();
        let avg = sum / nums.len() as i32;
        println!("Average: {}", avg);

        avg
    }

    #[test]
    fn test_avg() {
        let nums = [1, 2, 3, 4, 5];
        let avg = unsafe { avg(nums.as_ptr(), 5) };
        assert_eq!(avg, 3);
    }

    #[test]
    fn test_point() {
        let point = Point { x: 1, y: 2 };
        assert_eq!(point.x, 1);
        assert_eq!(point.y, 2);
    }

    #[test]
    fn test_window() {
        let window = Window(1, 2, 3, 4);
        assert_eq!(window.0, 1);
        assert_eq!(window.1, 2);
        assert_eq!(window.2, 3);
        assert_eq!(window.3, 4);
    }

}