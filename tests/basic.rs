#[cfg(test)]
mod tests {
    use plbindgen_macros::*;

    #[allow(non_camel_case_types)]
    type array<T> = *const T;

    #[export]
    pub unsafe fn avg(nums: array<i32>, nums_len: usize) -> i32 {
        let nums = std::slice::from_raw_parts(nums, nums_len);
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


}
