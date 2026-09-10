/// Move `index` one slot toward the front. Returns whether the slice changed.
pub fn move_up<T>(items: &mut [T], index: usize) -> bool {
    if index == 0 || index >= items.len() {
        return false;
    }
    items.swap(index, index - 1);
    true
}

/// Move `index` one slot toward the back. Returns whether the slice changed.
pub fn move_down<T>(items: &mut [T], index: usize) -> bool {
    if items.len() < 2 || index + 1 >= items.len() {
        return false;
    }
    items.swap(index, index + 1);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_up_swaps_with_predecessor() {
        let mut items = vec!["a", "b", "c"];
        assert!(move_up(&mut items, 1));
        assert_eq!(items, ["b", "a", "c"]);
    }

    #[test]
    fn move_up_at_start_is_noop() {
        let mut items = vec!["a", "b"];
        assert!(!move_up(&mut items, 0));
        assert_eq!(items, ["a", "b"]);
    }

    #[test]
    fn move_down_swaps_with_successor() {
        let mut items = vec!["a", "b", "c"];
        assert!(move_down(&mut items, 1));
        assert_eq!(items, ["a", "c", "b"]);
    }

    #[test]
    fn move_down_at_end_is_noop() {
        let mut items = vec!["a", "b"];
        assert!(!move_down(&mut items, 1));
        assert_eq!(items, ["a", "b"]);
    }

    #[test]
    fn out_of_range_is_noop() {
        let mut items = vec![1, 2];
        assert!(!move_up(&mut items, 9));
        assert!(!move_down(&mut items, 9));
        assert_eq!(items, [1, 2]);
    }
}
